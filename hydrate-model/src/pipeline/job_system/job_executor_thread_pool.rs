use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use crate::{ImportDataProvider, JobApi, JobApiImpl, JobEnumeratedDependencies, JobExecutor, JobId, JobProcessorRegistry, JobTypeId};

// Ask the thread to gather build data from the asset
pub struct JobExecutorThreadPoolRequestRunJob {
    pub job_id: JobId,
    pub job_type: JobTypeId,
    pub debug_name: Arc<String>,
    pub dependencies: Arc<JobEnumeratedDependencies>,
    pub input_data: Arc<Vec<u8>>,
    pub data_set: Arc<DataSet>,
}

pub enum JobExecutorThreadPoolRequest {
    RunJob(JobExecutorThreadPoolRequestRunJob),
}

// Results from successful build
pub struct JobExecutorThreadPoolOutcomeRunJobComplete {
    pub request: JobExecutorThreadPoolRequestRunJob,
    pub output_data: Arc<Vec<u8>>,
    //asset: SingleObject,
    //import_data: SingleObject,
}

// Results from failed build
pub struct JobExecutorThreadPoolOutcomeRunJobFailure {
    pub failure: String,
}

pub enum JobExecutorThreadPoolOutcome {
    RunJobComplete(JobExecutorThreadPoolOutcomeRunJobComplete),
    RunJobFailed(JobExecutorThreadPoolOutcomeRunJobFailure)
}

// Thread that tries to take jobs out of the request channel and ends when the finish channel is signalled
struct JobExecutorWorkerThread {
    finish_tx: Sender<()>,
    join_handle: JoinHandle<()>,
}

fn do_build(
    import_data_root_path: &Path,
    job_processor_registry: &JobProcessorRegistry,
    schema_set: &SchemaSet,
    job_api: &dyn JobApi,
    request: JobExecutorThreadPoolRequestRunJob,
) -> JobExecutorThreadPoolOutcome {
    profiling::scope!(&format!("Handle Job {}", request.debug_name));

    // Load the import data
    let mut required_import_data = HashMap::default();
    {
        for &import_data_id in &request.dependencies.import_data {
            profiling::scope!(&format!("Load Import Data {:?}", import_data_id));
            let import_data = super::super::import_jobs::load_import_data(import_data_root_path, schema_set, import_data_id);
            required_import_data.insert(import_data_id, import_data.import_data);
        }
    }

    // Load the upstream job result data

    // Execute the job
    let job_processor = job_processor_registry.get_processor(request.job_type).unwrap();
    let output_data = {
        profiling::scope!(&format!("JobProcessor::run_inner"));
        job_processor.run_inner(
            &request.input_data,
            &*request.data_set,
            schema_set,
            &required_import_data,
            job_api,
        )
    };

    JobExecutorThreadPoolOutcome::RunJobComplete(JobExecutorThreadPoolOutcomeRunJobComplete {
        request,
        output_data: Arc::new(output_data),
    })

    //TODO: Write to file
    //hydrate_base::uuid_path::uuid_to_path()
}

impl JobExecutorWorkerThread {
    fn new(
        job_processor_registry: JobProcessorRegistry,
        schema_set: SchemaSet,
        import_data_root_path: Arc<PathBuf>,
        job_data_root_path: Arc<PathBuf>,
        job_api: JobApiImpl,
        request_rx: Receiver<JobExecutorThreadPoolRequest>,
        outcome_tx: Sender<JobExecutorThreadPoolOutcome>,
        active_request_count: Arc<AtomicUsize>,
        thread_index: usize,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new().name("IO Thread".into()).spawn(move || {
            profiling::register_thread!(&format!("JobExecutorWorkerThread {}", thread_index));
            loop {
                crossbeam_channel::select! {
                    recv(request_rx) -> msg => {
                        match msg.unwrap() {
                            JobExecutorThreadPoolRequest::RunJob(msg) => {
                                profiling::scope!("JobExecutorThreadPoolRequest::RequestBuild");

                                let result = do_build(
                                    &*import_data_root_path,
                                    &job_processor_registry,
                                    &schema_set,
                                    &job_api,
                                    msg
                                );

                                outcome_tx.send(result).unwrap();
                                active_request_count.fetch_sub(1, Ordering::Release);
                            },
                        }
                    },
                    recv(finish_rx) -> _msg => {
                        return;
                    }
                }
            }
        }).unwrap();

        JobExecutorWorkerThread {
            finish_tx,
            join_handle,
        }
    }
}

// Spans N threads, proxies messages to/from them, and kills the threads when the pool is dropped
pub struct JobExecutorThreadPool {
    worker_threads: Vec<JobExecutorWorkerThread>,
    request_tx: Sender<JobExecutorThreadPoolRequest>,
    active_request_count: Arc<AtomicUsize>,
}

impl JobExecutorThreadPool {
    pub fn new(
        job_processor_registry: JobProcessorRegistry,
        schema_set: SchemaSet,
        import_data_root_path: &Path,
        job_data_root_path: &Path,
        job_api: JobApiImpl,
        max_requests_in_flight: usize,
        result_tx: Sender<JobExecutorThreadPoolOutcome>,
    ) -> Self {
        let import_data_root_path = Arc::new(import_data_root_path.to_path_buf());
        let job_data_root_path = Arc::new(job_data_root_path.to_path_buf());
        let (request_tx, request_rx) = crossbeam_channel::unbounded::<JobExecutorThreadPoolRequest>();
        let active_request_count = Arc::new(AtomicUsize::new(0));

        let mut worker_threads = Vec::with_capacity(max_requests_in_flight);
        for thread_index in 0..max_requests_in_flight {
            let worker = JobExecutorWorkerThread::new(
                job_processor_registry.clone(),
                schema_set.clone(),
                import_data_root_path.clone(),
                job_data_root_path.clone(),
                job_api.clone(),
                request_rx.clone(),
                result_tx.clone(),
                active_request_count.clone(),
                thread_index,
            );
            worker_threads.push(worker);
        }

        JobExecutorThreadPool {
            request_tx,
            worker_threads,
            active_request_count,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.active_request_count() == 0
    }

    pub fn active_request_count(&self) -> usize {
        self.active_request_count.load(Ordering::Relaxed)
    }

    pub fn add_request(
        &self,
        request: JobExecutorThreadPoolRequest,
    ) {
        self.active_request_count.fetch_add(1, Ordering::Release);
        self.request_tx.send(request).unwrap();
    }

    pub fn finish(self) {
        for worker_thread in &self.worker_threads {
            worker_thread.finish_tx.send(()).unwrap();
        }

        for worker_thread in self.worker_threads {
            worker_thread.join_handle.join().unwrap();
        }
    }
}
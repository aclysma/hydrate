use super::job_system_traits::{FetchedAssetData, FetchedImportData};
use super::{
    JobApi, JobApiImpl, JobEnumeratedDependencies, JobId, JobProcessorRegistry, JobTypeId,
};
use crate::{BuildLogEvent, PipelineResult};
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::AssetId;
use hydrate_data::{DataSet, SchemaSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

// Ask the thread to gather build data from the asset
pub(crate) struct JobExecutorThreadPoolRequestRunJob {
    pub job_id: JobId,
    pub job_type: JobTypeId,
    pub _debug_name: Arc<String>,
    pub _dependencies: Arc<JobEnumeratedDependencies>,
    pub input_data: Arc<Vec<u8>>,
    pub data_set: Arc<DataSet>,
}

pub(crate) enum JobExecutorThreadPoolRequest {
    RunJob(JobExecutorThreadPoolRequestRunJob),
}

pub struct JobExecutorThreadPoolOutcomeRunJobCompleteData {
    pub output_data: Arc<Vec<u8>>,
    pub fetched_asset_data: HashMap<AssetId, FetchedAssetData>,
    pub fetched_import_data: HashMap<AssetId, FetchedImportData>,
    pub log_events: Vec<BuildLogEvent>,
}

// Results from successful build
pub(crate) struct JobExecutorThreadPoolOutcomeRunJobComplete {
    pub request: JobExecutorThreadPoolRequestRunJob,
    pub result: PipelineResult<JobExecutorThreadPoolOutcomeRunJobCompleteData>,
    //asset: SingleObject,
    //import_data: SingleObject,
}

pub(crate) enum JobExecutorThreadPoolOutcome {
    RunJobComplete(JobExecutorThreadPoolOutcomeRunJobComplete),
}

// Thread that tries to take jobs out of the request channel and ends when the finish channel is signalled
struct JobExecutorWorkerThread {
    finish_tx: Sender<()>,
    join_handle: JoinHandle<()>,
}

fn do_build(
    job_processor_registry: &JobProcessorRegistry,
    schema_set: &SchemaSet,
    job_api: &dyn JobApi,
    request: &JobExecutorThreadPoolRequestRunJob,
) -> PipelineResult<JobExecutorThreadPoolOutcomeRunJobCompleteData> {
    profiling::scope!(&format!("Handle Job {}", request._debug_name));

    let mut fetched_asset_data = HashMap::<AssetId, FetchedAssetData>::default();
    let mut fetched_import_data = HashMap::<AssetId, FetchedImportData>::default();
    let mut log_events = Vec::default();

    // Execute the job
    let job_processor = job_processor_registry
        .get_processor(request.job_type)
        .unwrap();
    let output_data = {
        profiling::scope!(&format!("JobProcessor::run_inner"));
        job_processor.run_inner(
            request.job_id,
            &request.input_data,
            &*request.data_set,
            schema_set,
            job_api,
            &mut fetched_asset_data,
            &mut fetched_import_data,
            &mut log_events,
        )
    }?;

    Ok(JobExecutorThreadPoolOutcomeRunJobCompleteData {
        output_data,
        fetched_asset_data,
        fetched_import_data,
        log_events,
    })

    //TODO: Write to file
    //hydrate_base::uuid_path::uuid_to_path()
}

impl JobExecutorWorkerThread {
    fn new(
        job_processor_registry: JobProcessorRegistry,
        schema_set: SchemaSet,
        _job_data_root_path: Arc<PathBuf>,
        job_api: JobApiImpl,
        request_rx: Receiver<JobExecutorThreadPoolRequest>,
        outcome_tx: Sender<JobExecutorThreadPoolOutcome>,
        active_request_count: Arc<AtomicUsize>,
        _thread_index: usize,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new()
            .name("IO Thread".into())
            .spawn(move || {
                profiling::register_thread!(&format!("JobExecutorWorkerThread {}", _thread_index));
                loop {
                    crossbeam_channel::select! {
                        recv(request_rx) -> msg => {
                            match msg.unwrap() {
                                JobExecutorThreadPoolRequest::RunJob(msg) => {
                                    let result = std::panic::catch_unwind(|| {
                                        profiling::scope!("JobExecutorThreadPoolRequest::RequestBuild");
                                        do_build(
                                            &job_processor_registry,
                                            &schema_set,
                                            &job_api,
                                            &msg
                                        )
                                    });

                                    match result {
                                        Ok(result) => {
                                            outcome_tx.send(JobExecutorThreadPoolOutcome::RunJobComplete(JobExecutorThreadPoolOutcomeRunJobComplete {
                                                request: msg,
                                                result,
                                            })).unwrap();
                                        },
                                        Err(_) => {
                                            outcome_tx.send(JobExecutorThreadPoolOutcome::RunJobComplete(JobExecutorThreadPoolOutcomeRunJobComplete {
                                                request: msg,
                                                result: Err("Panic detected in build job.".into())
                                            })).unwrap();
                                        }
                                    }

                                    active_request_count.fetch_sub(1, Ordering::Release);
                                },
                            }
                        },
                        recv(finish_rx) -> _msg => {
                            return;
                        }
                    }
                }
            })
            .unwrap();

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
    pub(crate) fn new(
        job_processor_registry: JobProcessorRegistry,
        schema_set: SchemaSet,
        job_data_root_path: &Path,
        job_api: JobApiImpl,
        max_requests_in_flight: usize,
        result_tx: Sender<JobExecutorThreadPoolOutcome>,
    ) -> Self {
        let job_data_root_path = Arc::new(job_data_root_path.to_path_buf());
        let (request_tx, request_rx) =
            crossbeam_channel::unbounded::<JobExecutorThreadPoolRequest>();
        let active_request_count = Arc::new(AtomicUsize::new(0));

        let mut worker_threads = Vec::with_capacity(max_requests_in_flight);
        for thread_index in 0..max_requests_in_flight {
            let worker = JobExecutorWorkerThread::new(
                job_processor_registry.clone(),
                schema_set.clone(),
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

    pub(crate) fn add_request(
        &self,
        request: JobExecutorThreadPoolRequest,
    ) {
        self.active_request_count.fetch_add(1, Ordering::Release);
        self.request_tx.send(request).unwrap();
    }

    pub(crate) fn finish(self) {
        for worker_thread in &self.worker_threads {
            worker_thread.finish_tx.send(()).unwrap();
        }

        for worker_thread in self.worker_threads {
            worker_thread.join_handle.join().unwrap();
        }
    }
}

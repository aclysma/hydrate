use super::thumbnail_types::ThumbnailApi;
use super::ThumbnailEnumeratedDependencies;
use crate::{PipelineResult, ThumbnailImage, ThumbnailProviderRegistry};
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::AssetId;
use hydrate_data::SchemaSet;
use hydrate_schema::SchemaFingerprint;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

// Ask the thread to gather build data from the asset
pub(crate) struct ThumbnailThreadPoolRequestRunJob {
    pub asset_id: AssetId,
    //pub thumbnail_input_hash: ThumbnailInputHash,
    pub asset_type: SchemaFingerprint,
    pub dependencies: Arc<ThumbnailEnumeratedDependencies>,
}

pub(crate) enum ThumbnailThreadPoolRequest {
    RunJob(ThumbnailThreadPoolRequestRunJob),
}

// Results from successful build
pub(crate) struct ThumbnailThreadPoolOutcomeRunJobComplete {
    pub request: ThumbnailThreadPoolRequestRunJob,
    pub result: PipelineResult<ThumbnailImage>,
    //asset: SingleObject,
    //import_data: SingleObject,
}

pub(crate) enum ThumbnailThreadPoolOutcome {
    RunJobComplete(ThumbnailThreadPoolOutcomeRunJobComplete),
}

// Thread that tries to take jobs out of the request channel and ends when the finish channel is signalled
struct ThumbnailWorkerThread {
    finish_tx: Sender<()>,
    join_handle: JoinHandle<()>,
}

fn do_build(
    thumbnail_provider_registry: &ThumbnailProviderRegistry,
    schema_set: &SchemaSet,
    thumbnail_api: &ThumbnailApi,
    request: ThumbnailThreadPoolRequestRunJob,
) -> ThumbnailThreadPoolOutcome {
    profiling::scope!(&format!("Build Thumbnail {}", request.asset_id));

    // Execute the job
    let thumbnail_provider = thumbnail_provider_registry
        .provider_for_asset(request.asset_type)
        .unwrap();
    let result = {
        profiling::scope!(&format!("JobProcessor::run_inner"));
        thumbnail_provider.render_inner(
            request.asset_id,
            &*request.dependencies.gathered_data,
            schema_set,
            thumbnail_api,
        )
    };

    ThumbnailThreadPoolOutcome::RunJobComplete(ThumbnailThreadPoolOutcomeRunJobComplete {
        request,
        result,
    })

    //TODO: Write to file
    //hydrate_base::uuid_path::uuid_to_path()
}

impl ThumbnailWorkerThread {
    fn new(
        thumbnail_provider_registry: ThumbnailProviderRegistry,
        schema_set: SchemaSet,
        //job_data_root_path: Arc<PathBuf>,
        thumbnail_api: ThumbnailApi,
        request_rx: Receiver<ThumbnailThreadPoolRequest>,
        outcome_tx: Sender<ThumbnailThreadPoolOutcome>,
        active_request_count: Arc<AtomicUsize>,
        thread_index: usize,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new()
            .name("IO Thread".into())
            .spawn(move || {
                profiling::register_thread!(&format!("ThumbnailWorkerThread {}", thread_index));
                loop {
                    crossbeam_channel::select! {
                        recv(request_rx) -> msg => {
                            match msg.unwrap() {
                                ThumbnailThreadPoolRequest::RunJob(msg) => {
                                    profiling::scope!("ThumbnailThreadPoolRequest::RequestBuild");

                                    let result = do_build(
                                        &thumbnail_provider_registry,
                                        &schema_set,
                                        &thumbnail_api,
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
            })
            .unwrap();

        ThumbnailWorkerThread {
            finish_tx,
            join_handle,
        }
    }
}

// Spans N threads, proxies messages to/from them, and kills the threads when the pool is dropped
pub struct ThumbnailThreadPool {
    worker_threads: Vec<ThumbnailWorkerThread>,
    request_tx: Sender<ThumbnailThreadPoolRequest>,
    active_request_count: Arc<AtomicUsize>,
}

impl ThumbnailThreadPool {
    pub(crate) fn new(
        thumbnail_provider_registry: ThumbnailProviderRegistry,
        schema_set: SchemaSet,
        //job_data_root_path: &Path,
        thumbnail_api: ThumbnailApi,
        max_requests_in_flight: usize,
        result_tx: Sender<ThumbnailThreadPoolOutcome>,
    ) -> Self {
        //let job_data_root_path = Arc::new(job_data_root_path.to_path_buf());
        let (request_tx, request_rx) = crossbeam_channel::unbounded::<ThumbnailThreadPoolRequest>();
        let active_request_count = Arc::new(AtomicUsize::new(0));

        let mut worker_threads = Vec::with_capacity(max_requests_in_flight);
        for thread_index in 0..max_requests_in_flight {
            let worker = ThumbnailWorkerThread::new(
                thumbnail_provider_registry.clone(),
                schema_set.clone(),
                //job_data_root_path.clone(),
                thumbnail_api.clone(),
                request_rx.clone(),
                result_tx.clone(),
                active_request_count.clone(),
                thread_index,
            );
            worker_threads.push(worker);
        }

        ThumbnailThreadPool {
            request_tx,
            worker_threads,
            active_request_count,
        }
    }

    pub(crate) fn add_request(
        &self,
        request: ThumbnailThreadPoolRequest,
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

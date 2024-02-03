use crate::build::{BuiltArtifact, WrittenArtifact};
use crate::import::ImportData;
use crate::{BuildLogData, BuildLogEvent, LogEventLevel, PipelineError, PipelineResult};
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::uuid_path::uuid_and_hash_to_path;
use hydrate_base::{ArtifactId, AssetId};
use hydrate_data::{DataSet, SchemaSet};
use log::Log;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::hash::Hasher;
use std::io::{BufWriter, Write};
use std::panic::RefUnwindSafe;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use super::*;

struct JobWrapper<T: JobProcessor>(T);

impl<T: JobProcessor + Send + Sync + RefUnwindSafe> JobProcessorAbstract for JobWrapper<T>
where
    <T as JobProcessor>::InputT: for<'a> Deserialize<'a> + 'static,
    <T as JobProcessor>::OutputT: Serialize + 'static,
{
    fn version_inner(&self) -> u32 {
        self.0.version()
    }

    fn enumerate_dependencies_inner(
        &self,
        job_id: JobId,
        job_requestor: JobRequestor,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        log_events: &mut Vec<BuildLogEvent>,
    ) -> PipelineResult<JobEnumeratedDependencies> {
        let data: <T as JobProcessor>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        self.0.enumerate_dependencies(EnumerateDependenciesContext {
            job_id,
            job_requestor,
            input: &data,
            data_set,
            schema_set,
            log_events: &Rc::new(RefCell::new(log_events)),
        })
    }

    fn run_inner(
        &self,
        job_id: JobId,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
        fetched_asset_data: &mut HashMap<AssetId, FetchedAssetData>,
        fetched_import_data: &mut HashMap<AssetId, FetchedImportData>,
        log_events: &mut Vec<BuildLogEvent>,
    ) -> PipelineResult<Arc<Vec<u8>>> {
        let data: <T as JobProcessor>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        let output = {
            profiling::scope!(&format!("{:?}::run", std::any::type_name::<T>()));
            self.0.run(&RunContext {
                job_id,
                input: &data,
                data_set,
                schema_set,
                fetched_asset_data: &Rc::new(RefCell::new(fetched_asset_data)),
                fetched_import_data: &Rc::new(RefCell::new(fetched_import_data)),
                job_api,
                log_events: &Rc::new(RefCell::new(log_events)),
            })
        }?;
        Ok(Arc::new(bincode::serialize(&output)?))
    }
}

// struct JobHistory {
//     // version() returned from the processor, if it bumps we invalidate the job
//     job_version: u32,
//
//     // The dependencies that existed when we ran this job last time (may not need this?)
//     dependencies: JobEnumeratedDependencies,
//     // Hash of import data used to run the job. If our import data changed, the job results can't be
//     // reused
//     import_data_hashes: HashMap<AssetId, u128>,
//     // All the jobs this job produced. Even if we can reuse the results of this job, we will have
//     // to check downstream jobs do not detect an input data change.
//     downstream_jobs: Vec<QueuedJob>,
// }

struct JobState {
    job_type: JobTypeId,
    dependencies: Arc<JobEnumeratedDependencies>,
    input_data: Arc<Vec<u8>>,
    debug_name: Arc<String>,

    // When we send the job to the thread pool, this is set to true
    has_been_scheduled: bool,
    // This would eventually be stored on file system
    output_data: Option<JobStateOutput>,
}

struct JobStateOutput {
    output_data: PipelineResult<Arc<Vec<u8>>>,
    fetched_asset_data: HashMap<AssetId, FetchedAssetData>,
    fetched_import_data: HashMap<AssetId, FetchedImportData>,
}

//TODO: Future optimization, we clone this and it could be big, especially when we re-run jobs. We
// could just enqueue the ID of the job if we have the job history
#[derive(Clone)]
struct QueuedJob {
    job_id: JobId,
    job_requestor: JobRequestor,
    job_type: JobTypeId,
    input_data: Arc<Vec<u8>>,
    dependencies: PipelineResult<JobEnumeratedDependencies>,
    debug_name: Arc<String>,
}

#[derive(Default)]
pub struct JobProcessorRegistryBuilder {
    job_processors: HashMap<JobTypeId, Arc<dyn JobProcessorAbstract>>,
}

impl JobProcessorRegistryBuilder {
    pub fn register_job_processor<
        T: JobProcessor + Send + Sync + RefUnwindSafe + Default + 'static,
    >(
        &mut self
    ) where
        <T as JobProcessor>::InputT: for<'a> Deserialize<'a>,
        <T as JobProcessor>::OutputT: Serialize,
    {
        let old = self.job_processors.insert(
            JobTypeId::from_bytes(T::UUID),
            Arc::new(JobWrapper(T::default())),
        );
        if old.is_some() {
            panic!("Multiple job processors registered with the same UUID");
        }
    }

    pub fn register_job_processor_instance<
        T: JobProcessor + Send + Sync + RefUnwindSafe + 'static,
    >(
        &mut self,
        job_processor: T,
    ) where
        <T as JobProcessor>::InputT: for<'a> Deserialize<'a>,
        <T as JobProcessor>::OutputT: Serialize,
    {
        let old = self.job_processors.insert(
            JobTypeId::from_bytes(T::UUID),
            Arc::new(JobWrapper(job_processor)),
        );
        if old.is_some() {
            panic!("Multiple job processors registered with the same UUID");
        }
    }

    pub fn build(self) -> JobProcessorRegistry {
        let inner = JobProcessorRegistryInner {
            job_processors: self.job_processors,
        };

        JobProcessorRegistry {
            inner: Arc::new(inner),
        }
    }
}

pub struct JobProcessorRegistryInner {
    job_processors: HashMap<JobTypeId, Arc<dyn JobProcessorAbstract>>,
}

#[derive(Clone)]
pub struct JobProcessorRegistry {
    inner: Arc<JobProcessorRegistryInner>,
}

impl JobProcessorRegistry {
    fn get(
        &self,
        job_type_id: JobTypeId,
    ) -> Option<&dyn JobProcessorAbstract> {
        self.inner.job_processors.get(&job_type_id).map(|x| &**x)
    }

    fn contains_key(
        &self,
        job_type_id: JobTypeId,
    ) -> bool {
        self.inner.job_processors.contains_key(&job_type_id)
    }

    pub(crate) fn get_processor(
        &self,
        job_type: JobTypeId,
    ) -> Option<Arc<dyn JobProcessorAbstract>> {
        self.inner.job_processors.get(&job_type).cloned()
    }
}

struct JobApiImplInner {
    schema_set: SchemaSet,
    import_data_root_path: PathBuf,
    build_data_root_path: PathBuf,
    job_processor_registry: JobProcessorRegistry,
    job_create_queue_tx: Sender<QueuedJob>,
    artifact_handle_created_tx: Sender<AssetArtifactIdPair>,
    written_artifact_queue_tx: Sender<WrittenArtifact>,
}

#[derive(Clone)]
pub struct JobApiImpl {
    inner: Arc<JobApiImplInner>,
}

impl JobApi for JobApiImpl {
    fn enqueue_job(
        &self,
        job_requestor: JobRequestor,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        new_job: NewJob,
        debug_name: String,
        log_events: &mut Vec<BuildLogEvent>,
    ) -> PipelineResult<JobId> {
        // Dependencies:
        // - Job Versioning - so if logic changes we can bump version of the processor and kick jobs to rerun
        // - Asset (we need to know hash of data in it)
        // - Import Data (we need to know hash of data in it)
        // - Intermediate data (we need the job's input hash, which takes into account the parameters of the job including
        //   hashes of above stuff
        // - Build Data (we need the build hash, which takes into account the asset/import data
        let job_id = JobId::from_u128(new_job.input_hash);
        let processor = self
            .inner
            .job_processor_registry
            .get(new_job.job_type)
            .unwrap();

        let dependencies = processor.enumerate_dependencies_inner(
            job_id,
            job_requestor,
            &new_job.input_data,
            data_set,
            schema_set,
            log_events,
        );

        self.inner
            .job_create_queue_tx
            .send(QueuedJob {
                job_id,
                job_requestor,
                job_type: new_job.job_type,
                input_data: Arc::new(new_job.input_data),
                dependencies,
                debug_name: Arc::new(debug_name),
            })
            .unwrap();

        Ok(job_id)
    }

    fn artifact_handle_created(
        &self,
        asset_id: AssetId,
        artifact_id: ArtifactId,
    ) {
        //TODO: Is this necessary, can we handle it when the job result is returned?
        self.inner
            .artifact_handle_created_tx
            .send(AssetArtifactIdPair {
                asset_id,
                artifact_id,
            })
            .unwrap();
    }

    fn produce_artifact(
        &self,
        artifact: BuiltArtifact,
    ) {
        profiling::scope!("Write Asset to Disk");
        //
        // Hash the artifact
        //
        let mut hasher = siphasher::sip::SipHasher::default();
        artifact.data.hash(&mut hasher);
        artifact.metadata.hash(&mut hasher);
        let build_hash = hasher.finish();

        //
        // Determine where we will store the asset and ensure the directory exists
        //
        let path = uuid_and_hash_to_path(
            &self.inner.build_data_root_path,
            artifact.artifact_id.as_uuid(),
            build_hash,
            "bf",
        );

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }

        //
        // Serialize the artifacts to disk
        //
        let file = std::fs::File::create(&path).unwrap();
        let mut buf_writer = BufWriter::new(file);
        artifact.metadata.write_header(&mut buf_writer).unwrap();
        buf_writer.write(&artifact.data).unwrap();

        //
        // Send info about the written asset back to main thread for inclusion in the manifest
        //
        //TODO: Is this necessary, can we handle it when the job result is returned?
        self.inner
            .written_artifact_queue_tx
            .send(WrittenArtifact {
                asset_id: artifact.asset_id,
                artifact_id: artifact.artifact_id,
                metadata: artifact.metadata,
                build_hash,
                artifact_key_debug_name: artifact.artifact_key_debug_name,
            })
            .unwrap();
    }

    fn fetch_import_data(
        &self,
        asset_id: AssetId,
    ) -> PipelineResult<ImportData> {
        crate::import::load_import_data(
            &self.inner.import_data_root_path,
            &self.inner.schema_set,
            asset_id,
        )
    }
}

#[derive(Clone, Debug)]
pub struct AssetArtifactIdPair {
    pub asset_id: AssetId,
    pub artifact_id: ArtifactId,
}

pub struct JobExecutor {
    // Will be needed when we start doing job caching
    _root_path: PathBuf,
    job_api_impl: JobApiImpl,

    job_processor_registry: JobProcessorRegistry,

    // All the jobs that we have run or will run in this job batch
    current_jobs: HashMap<JobId, JobState>,

    // Queue for jobs to request additional jobs to run
    job_create_queue_rx: Receiver<QueuedJob>,

    //TODO: We will have additional deques for jobs that are in a ready state to avoid O(n) iteration
    artifact_handle_created_rx: Receiver<AssetArtifactIdPair>,

    written_artifact_queue_rx: Receiver<WrittenArtifact>,

    thread_pool_result_rx: Receiver<JobExecutorThreadPoolOutcome>,
    thread_pool: Option<JobExecutorThreadPool>,

    completed_job_count: usize,
    last_job_print_time: Option<std::time::Instant>,
}

impl Drop for JobExecutor {
    fn drop(&mut self) {
        let thread_pool = self.thread_pool.take().unwrap();
        thread_pool.finish();
    }
}

impl JobExecutor {
    // pub fn all_build_event_logs(&self, ) {
    //     for (job_id, job_state) in &self.current_jobs {
    //         if let Some(output_data) = job_state.output_data {
    //             for log_event in &output_data.log_events {
    //
    //             }
    //         }
    //     }
    // }

    pub fn reset(&mut self) {
        assert!(self.is_idle());
        self.current_jobs.clear();
        self.completed_job_count = 0;
    }

    pub fn new(
        schema_set: &SchemaSet,
        job_processor_registry: &JobProcessorRegistry,
        import_data_root_path: PathBuf,
        job_data_root_path: PathBuf,
        build_data_root_path: PathBuf,
    ) -> Self {
        let (job_create_queue_tx, job_create_queue_rx) = crossbeam_channel::unbounded();
        //let (job_completed_queue_tx, job_completed_queue_rx) = crossbeam_channel::unbounded();
        //let (built_asset_queue_tx, built_asset_queue_rx) = crossbeam_channel::unbounded();

        let (artifact_handle_created_tx, artifact_handle_created_rx) =
            crossbeam_channel::unbounded();
        let (written_artifact_queue_tx, written_artifact_queue_rx) = crossbeam_channel::unbounded();

        let job_api_impl = JobApiImpl {
            inner: Arc::new(JobApiImplInner {
                schema_set: schema_set.clone(),
                import_data_root_path: import_data_root_path.clone(),
                build_data_root_path,
                job_processor_registry: job_processor_registry.clone(),
                job_create_queue_tx,
                artifact_handle_created_tx,
                written_artifact_queue_tx,
            }),
        };

        let thread_count = num_cpus::get();
        //let thread_count = 1;

        let (thread_pool_result_tx, thread_pool_result_rx) = crossbeam_channel::unbounded();
        let thread_pool = JobExecutorThreadPool::new(
            job_processor_registry.clone(),
            schema_set.clone(),
            &job_data_root_path,
            job_api_impl.clone(),
            thread_count,
            thread_pool_result_tx,
        );

        JobExecutor {
            _root_path: job_data_root_path,
            job_api_impl,
            job_processor_registry: job_processor_registry.clone(),
            //job_history: Default::default(),
            current_jobs: Default::default(),
            //job_create_queue_tx,
            job_create_queue_rx,
            //job_completed_queue_tx,
            //job_completed_queue_rx,
            // built_asset_queue_tx,
            // built_asset_queue_rx,
            //artifact_handle_created_tx,
            artifact_handle_created_rx,
            //built_artifact_queue_tx,
            written_artifact_queue_rx,
            thread_pool_result_rx,
            thread_pool: Some(thread_pool),
            completed_job_count: 0,
            last_job_print_time: None,
        }
    }

    pub fn job_api(&self) -> &dyn JobApi {
        &self.job_api_impl
    }

    // pub fn take_built_assets(&self) -> Vec<BuiltAsset> {
    //     let mut built_assets = Vec::default();
    //     while let Ok(built_asset) = self.built_asset_queue_rx.try_recv() {
    //         built_assets.push(built_asset);
    //     }
    //
    //     built_assets
    // }

    pub fn take_written_artifacts(
        &self,
        artifact_asset_lookup: &mut HashMap<ArtifactId, AssetId>,
    ) -> Vec<WrittenArtifact> {
        let mut written_artifacts = Vec::default();
        while let Ok(written_artifact) = self.written_artifact_queue_rx.try_recv() {
            let old = artifact_asset_lookup
                .insert(written_artifact.artifact_id, written_artifact.asset_id);
            //assert!(old.is_none());

            if old.is_some() {
                // Is it possible a job has already created a handle to this asset, even if the asset hasn't been built yet
                assert_eq!(old, Some(written_artifact.asset_id));
            }

            written_artifacts.push(written_artifact);
        }

        while let Ok(asset_artifact_pair) = self.artifact_handle_created_rx.try_recv() {
            let old = artifact_asset_lookup.insert(
                asset_artifact_pair.artifact_id,
                asset_artifact_pair.asset_id,
            );
            if old.is_some() {
                // This happens after taking built artifacts because the built artifacts might have handles
                // to artifacts and we need to know the asset ID associated with them.
                assert_eq!(old, Some(asset_artifact_pair.asset_id));
            }
        }

        written_artifacts
    }

    fn clear_create_queue(&mut self) {
        while let Ok(queued_job) = self.job_create_queue_rx.try_recv() {
            // do nothing with it
            drop(queued_job);
        }
    }

    fn handle_create_queue(
        &mut self,
        log_data: &mut BuildLogData,
    ) {
        while let Ok(queued_job) = self.job_create_queue_rx.try_recv() {
            log_data
                .requestors
                .entry(queued_job.job_id)
                .or_default()
                .push(queued_job.job_requestor);
            // If key exists, we already queued a job with these exact inputs and we can reuse the outputs
            if !self.current_jobs.contains_key(&queued_job.job_id) {
                assert!(self
                    .job_processor_registry
                    .contains_key(queued_job.job_type));

                let job_state = match queued_job.dependencies {
                    Ok(dependencies) => JobState {
                        job_type: queued_job.job_type,
                        dependencies: Arc::new(dependencies),
                        input_data: queued_job.input_data,
                        debug_name: queued_job.debug_name,
                        has_been_scheduled: false,
                        output_data: None,
                    },
                    Err(e) => {
                        log_data.log_events.push(BuildLogEvent {
                            job_id: Some(queued_job.job_id),
                            asset_id: None,
                            level: LogEventLevel::FatalError,
                            message: format!(
                                "enumerate_dependencies returned error: {}",
                                e.to_string()
                            ),
                        });

                        JobState {
                            job_type: queued_job.job_type,
                            dependencies: Arc::new(JobEnumeratedDependencies::default()),
                            input_data: queued_job.input_data,
                            debug_name: queued_job.debug_name,
                            has_been_scheduled: true,
                            output_data: Some(JobStateOutput {
                                output_data: Err(e),
                                fetched_asset_data: Default::default(),
                                fetched_import_data: Default::default(),
                            }),
                        }
                    }
                };

                self.current_jobs.insert(queued_job.job_id, job_state);
            }
        }
    }

    fn handle_completed_queue(
        &mut self,
        log_events: &mut Vec<BuildLogEvent>,
    ) {
        while let Ok(result) = self.thread_pool_result_rx.try_recv() {
            match result {
                JobExecutorThreadPoolOutcome::RunJobComplete(msg) => {
                    let job = self.current_jobs.get_mut(&msg.request.job_id).unwrap();
                    match msg.result {
                        Ok(data) => {
                            job.output_data = Some(JobStateOutput {
                                output_data: Ok(data.output_data),
                                fetched_asset_data: data.fetched_asset_data,
                                fetched_import_data: data.fetched_import_data,
                            });

                            for log_event in data.log_events {
                                log_events.push(log_event);
                            }
                        }
                        Err(e) => {
                            log_events.push(BuildLogEvent {
                                job_id: Some(msg.request.job_id),
                                asset_id: None,
                                level: LogEventLevel::FatalError,
                                message: format!("Build job returned error: {}", e.to_string()),
                            });

                            job.output_data = Some(JobStateOutput {
                                output_data: Err(e),
                                fetched_asset_data: Default::default(),
                                fetched_import_data: Default::default(),
                            });
                        }
                    }
                    self.completed_job_count += 1;

                    // When we a
                    //TODO: do we hash stuff here and do anything with it?
                }
            }
        }
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        data_set: &Arc<DataSet>,
        log_data: &mut BuildLogData,
    ) {
        //
        // Pull jobs off the create queue. Determine their dependencies and prepare them to run.
        //
        self.handle_create_queue(log_data);

        let mut started_jobs = Vec::default();

        //TODO: Don't iterate every job in existence
        for (&job_id, job_state) in &self.current_jobs {
            //
            // See if we already did this job during the current execution cycle
            //
            if job_state.has_been_scheduled {
                continue;
            }

            assert!(job_state.output_data.is_none());

            //
            // See if the job we need to wait for has completed
            //
            let mut waiting_on_upstream_job = false;
            for upstream_job in &job_state.dependencies.upstream_jobs {
                let dependency = self.current_jobs.get(upstream_job);
                let Some(dependency_job_state) = dependency else {
                    panic!("Job has a dependency on another job that has not been created");
                    //TODO: We would not terminate if we remove the panic
                    //break;
                };

                if dependency_job_state.output_data.is_none() {
                    waiting_on_upstream_job = true;
                    break;
                }
            }

            if waiting_on_upstream_job {
                continue;
            }

            //
            // If we've run this job in the past and have a cached result, we can reuse the result.
            // But we still need to schedule downstream jobs in case their dependencies changed and
            // they need to be reprocessed
            //
            //let mut _has_run_job_before = false;
            //let mut _can_reuse_result = true;
            // let job_history = self.job_history.get(&job_id);
            // if let Some(job_history) = job_history {
            //     _has_run_job_before = true;
            //     _can_reuse_result = true;
            //     //TODO: Check if input data not represented in the job hash changed
            //     // job_history.import_data_hashed
            //     // job_history.dependencies
            //
            //     // can_reuse_result may be set to false here
            //
            //
            //     if _has_run_job_before && _can_reuse_result {
            //         // Kick off child jobs
            //         for downstream_job in &job_history.downstream_jobs {
            //             self.enqueue_job_internal(downstream_job.clone());
            //         }
            //
            //         // Bail, we will reuse the output from the previous run
            //         break;
            //     }
            // }

            //
            // At this point we have either never run the job before, or we know the job inputs have changed
            // Go ahead and run it.
            //
            self.thread_pool
                .as_ref()
                .unwrap()
                .add_request(JobExecutorThreadPoolRequest::RunJob(
                    JobExecutorThreadPoolRequestRunJob {
                        job_id,
                        job_type: job_state.job_type,
                        data_set: data_set.clone(),
                        debug_name: job_state.debug_name.clone(),
                        dependencies: job_state.dependencies.clone(),
                        input_data: job_state.input_data.clone(),
                    },
                ));

            started_jobs.push(job_id);
        }

        for job_id in started_jobs {
            self.current_jobs
                .get_mut(&job_id)
                .unwrap()
                .has_been_scheduled = true;
        }

        self.handle_completed_queue(&mut log_data.log_events);

        let now = std::time::Instant::now();
        let mut print_progress = true;
        if let Some(last_job_print_time) = self.last_job_print_time {
            if (now - last_job_print_time) < std::time::Duration::from_millis(500) {
                print_progress = false;
            }
        }

        if print_progress {
            log::info!(
                "Jobs: {}/{}",
                self.completed_job_count,
                self.current_jobs.len()
            );
            self.last_job_print_time = Some(now);
        }
    }

    pub fn completed_job_count(&self) -> usize {
        self.completed_job_count
    }

    pub fn current_job_count(&self) -> usize {
        self.current_jobs.len()
    }

    pub fn stop(
        &mut self,
        log_events: &mut Vec<BuildLogEvent>,
    ) {
        //TODO: If we have a thread pool do we need to notify them to stop?
        self.clear_create_queue();
        self.handle_completed_queue(log_events);

        self.current_jobs.clear();
    }

    pub fn is_idle(&self) -> bool {
        if !self.job_create_queue_rx.is_empty() {
            return false;
        }

        // if !self.job_completed_queue_rx.is_empty() {
        //     return false;
        // }

        if !self.thread_pool.as_ref().unwrap().is_idle() {
            return false;
        }

        // if !self.built_asset_queue_rx.is_empty() {
        //     return false;
        // }

        if !self.written_artifact_queue_rx.is_empty() {
            return false;
        }

        //TODO: Don't iterate, keep a count
        for (_id, job) in &self.current_jobs {
            if job.output_data.is_none() {
                return false;
            }
        }

        true
    }
}

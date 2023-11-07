use crate::BuiltArtifact;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::{ArtifactId, ObjectId};
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use super::*;

struct JobWrapper<T: JobProcessor>(T);

impl<T: JobProcessor> JobProcessorAbstract for JobWrapper<T>
where
    <T as JobProcessor>::InputT: for<'a> Deserialize<'a> + 'static,
    <T as JobProcessor>::OutputT: Serialize + 'static,
{
    fn version_inner(&self) -> u32 {
        self.0.version()
    }

    fn enumerate_dependencies_inner(
        &self,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies {
        let data: <T as JobProcessor>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        self.0.enumerate_dependencies(&data, data_set, schema_set)
    }

    fn run_inner(
        &self,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        job_api: &dyn JobApi,
    ) -> Vec<u8> {
        let data: <T as JobProcessor>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        let output = self
            .0
            .run(&data, data_set, schema_set, dependency_data, job_api);
        bincode::serialize(&output).unwrap()
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
//     import_data_hashes: HashMap<ObjectId, u128>,
//     // All the jobs this job produced. Even if we can reuse the results of this job, we will have
//     // to check downstream jobs do not detect an input data change.
//     downstream_jobs: Vec<QueuedJob>,
// }

struct JobState {
    job_type: JobTypeId,
    dependencies: JobEnumeratedDependencies,
    input_data: Vec<u8>,
    // This would eventually be stored on file system
    output_data: Option<Vec<u8>>,
}

//TODO: Future optimization, we clone this and it could be big, especially when we re-run jobs. We
// could just enqueue the ID of the job if we have the job history
#[derive(Clone)]
struct QueuedJob {
    job_id: JobId,
    job_type: JobTypeId,
    input_data: Vec<u8>,
    dependencies: JobEnumeratedDependencies,
}

struct CompletedJob {
    job_id: JobId,
    output_data: Vec<u8>,
}

#[derive(Default)]
pub struct JobProcessorRegistryBuilder {
    job_processors: HashMap<JobTypeId, Box<dyn JobProcessorAbstract>>,
}

impl JobProcessorRegistryBuilder {
    pub fn register_job_processor<T: JobProcessor + Default + 'static>(&mut self)
    where
        <T as JobProcessor>::InputT: for<'a> Deserialize<'a>,
        <T as JobProcessor>::OutputT: Serialize,
    {
        let old = self.job_processors.insert(
            JobTypeId::from_bytes(T::UUID),
            Box::new(JobWrapper(T::default())),
        );
        if old.is_some() {
            panic!("Multiple job processors registered with the same UUID");
        }
    }

    pub fn register_job_processor_instance<T: JobProcessor + 'static>(
        &mut self,
        job_processor: T,
    ) where
        <T as JobProcessor>::InputT: for<'a> Deserialize<'a>,
        <T as JobProcessor>::OutputT: Serialize,
    {
        let old = self.job_processors.insert(
            JobTypeId::from_bytes(T::UUID),
            Box::new(JobWrapper(job_processor)),
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
    job_processors: HashMap<JobTypeId, Box<dyn JobProcessorAbstract>>,
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
}

#[derive(Clone)]
pub struct JobProcessorRegistry {
    inner: Arc<JobProcessorRegistryInner>,
}

#[derive(Clone, Debug)]
pub struct AssetArtifactIdPair {
    pub asset_id: ObjectId,
    pub artifact_id: ArtifactId,
}

pub struct JobExecutor {
    // Will be needed when we start doing job caching
    _root_path: PathBuf,
    job_processor_registry: JobProcessorRegistry,

    // Represents all known previous executions of a job
    //job_history: HashMap<JobId, JobHistory>,
    // All the jobs that we have run or will run in this job batch
    current_jobs: HashMap<JobId, JobState>,

    // Queue for jobs to request additional jobs to run
    job_create_queue_tx: Sender<QueuedJob>,
    job_create_queue_rx: Receiver<QueuedJob>,

    //TODO: We will have additional deques for jobs that are in a ready state to avoid O(n) iteration

    // Queue for jobs to notify that they have completed
    job_completed_queue_tx: Sender<CompletedJob>,
    job_completed_queue_rx: Receiver<CompletedJob>,

    artifact_handle_created_tx: Sender<AssetArtifactIdPair>,
    artifact_handle_created_rx: Receiver<AssetArtifactIdPair>,

    // built_asset_queue_tx: Sender<BuiltAsset>,
    // built_asset_queue_rx: Receiver<BuiltAsset>,
    built_artifact_queue_tx: Sender<BuiltArtifact>,
    built_artifact_queue_rx: Receiver<BuiltArtifact>,
}

impl JobApi for JobExecutor {
    fn enqueue_job(
        &self,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        new_job: NewJob,
    ) -> JobId {
        // Dependencies:
        // - Job Versioning - so if logic changes we can bump version of the processor and kick jobs to rerun
        // - Asset (we need to know hash of data in it)
        // - Import Data (we need to know hash of data in it)
        // - Intermediate data (we need the job's input hash, which takes into account the parameters of the job including
        //   hashes of above stuff
        // - Build Data (we need the build hash, which takes into account the asset/import data
        let job_id = JobId::from_u128(new_job.input_hash);
        let processor = self.job_processor_registry.get(new_job.job_type).unwrap();
        let dependencies =
            processor.enumerate_dependencies_inner(&new_job.input_data, data_set, schema_set);
        self.enqueue_job_internal(QueuedJob {
            job_id,
            job_type: new_job.job_type,
            input_data: new_job.input_data,
            dependencies,
        });
        job_id
    }

    // fn produce_asset(&self, asset: BuiltAsset) {
    //     self.built_asset_queue_tx.send(asset).unwrap();
    // }

    fn artifact_handle_created(
        &self,
        asset_id: ObjectId,
        artifact_id: ArtifactId,
    ) {
        self.artifact_handle_created_tx
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
        self.built_artifact_queue_tx.send(artifact).unwrap();
    }
}

impl JobExecutor {
    pub fn new(
        root_path: PathBuf,
        job_processor_registry: &JobProcessorRegistry,
    ) -> Self {
        let (job_create_queue_tx, job_create_queue_rx) = crossbeam_channel::unbounded();
        let (job_completed_queue_tx, job_completed_queue_rx) = crossbeam_channel::unbounded();
        //let (built_asset_queue_tx, built_asset_queue_rx) = crossbeam_channel::unbounded();

        let (artifact_handle_created_tx, artifact_handle_created_rx) =
            crossbeam_channel::unbounded();
        let (built_artifact_queue_tx, built_artifact_queue_rx) = crossbeam_channel::unbounded();

        JobExecutor {
            _root_path: root_path,
            job_processor_registry: job_processor_registry.clone(),
            //job_history: Default::default(),
            current_jobs: Default::default(),
            job_create_queue_tx,
            job_create_queue_rx,
            job_completed_queue_tx,
            job_completed_queue_rx,
            // built_asset_queue_tx,
            // built_asset_queue_rx,
            artifact_handle_created_tx,
            artifact_handle_created_rx,
            built_artifact_queue_tx,
            built_artifact_queue_rx,
        }
    }

    // pub fn take_built_assets(&self) -> Vec<BuiltAsset> {
    //     let mut built_assets = Vec::default();
    //     while let Ok(built_asset) = self.built_asset_queue_rx.try_recv() {
    //         built_assets.push(built_asset);
    //     }
    //
    //     built_assets
    // }

    pub fn take_built_artifacts(
        &self,
        artifact_asset_lookup: &mut HashMap<ArtifactId, ObjectId>,
    ) -> Vec<BuiltArtifact> {
        let mut built_artifacts = Vec::default();
        while let Ok(built_artifact) = self.built_artifact_queue_rx.try_recv() {
            let old =
                artifact_asset_lookup.insert(built_artifact.artifact_id, built_artifact.asset_id);
            if old.is_some() {
                println!(
                    "{:?} {:?} {:?}",
                    built_artifact.artifact_id,
                    built_artifact.asset_id,
                    built_artifact.metadata.asset_type
                );
                panic!("produced same asset multiple times?");
            }
            // We produced the same asset multiple times?
            assert!(old.is_none());
            // if old.is_some() {
            //     assert_eq!(old, Some(built_artifact.asset_id));
            // }

            built_artifacts.push(built_artifact);
        }

        // This happens after taking built artifacts because the built artifacts might have handles
        // to artifacts and we need to know the asset ID associated with them.
        while let Ok(asset_artifact_pair) = self.artifact_handle_created_rx.try_recv() {
            println!(
                "pair {:?} {:?}",
                asset_artifact_pair.artifact_id, asset_artifact_pair.asset_id
            );
            let old = artifact_asset_lookup.insert(
                asset_artifact_pair.artifact_id,
                asset_artifact_pair.asset_id,
            );
            if old.is_some() {
                assert_eq!(old, Some(asset_artifact_pair.asset_id));
            }
        }

        built_artifacts
    }

    fn enqueue_job_internal(
        &self,
        job: QueuedJob,
    ) {
        self.job_create_queue_tx.send(job).unwrap();
    }

    fn clear_create_queue(&mut self) {
        while let Ok(queued_job) = self.job_create_queue_rx.try_recv() {
            // do nothing with it
            drop(queued_job);
        }
    }

    fn handle_create_queue(&mut self) {
        while let Ok(queued_job) = self.job_create_queue_rx.try_recv() {
            // If key exists, we already queued a job with these exact inputs and we can reuse the outputs
            if !self.current_jobs.contains_key(&queued_job.job_id) {
                assert!(self
                    .job_processor_registry
                    .contains_key(queued_job.job_type));

                self.current_jobs.insert(
                    queued_job.job_id,
                    JobState {
                        job_type: queued_job.job_type,
                        dependencies: queued_job.dependencies,
                        input_data: queued_job.input_data,
                        output_data: None,
                    },
                );
            }
        }
    }

    fn handle_completed_queue(&mut self) {
        while let Ok(completed_job) = self.job_completed_queue_rx.try_recv() {
            self.current_jobs
                .get_mut(&completed_job.job_id)
                .unwrap()
                .output_data = Some(completed_job.output_data);
        }
    }

    pub fn update(
        &mut self,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        import_data_provider: &dyn ImportDataProvider,
    ) {
        //
        // Pull jobs off the create queue. Determine their dependencies and prepare them to run.
        //
        self.handle_create_queue();

        //TODO: Don't iterate every job in existence
        for (&job_id, job_state) in &self.current_jobs {
            //
            // See if we already did this job during the current execution cycle
            //
            if job_state.output_data.is_some() {
                continue;
            }

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

            //TODO: Read from files

            // Load the import data
            let mut required_import_data = HashMap::default();
            for &import_data_id in &job_state.dependencies.import_data {
                let import_data = import_data_provider.load_import_data(schema_set, import_data_id);
                required_import_data.insert(import_data_id, import_data.import_data);
            }

            // Load the upstream job result data

            // Execute the job
            let job_processor = self.job_processor_registry.get(job_state.job_type).unwrap();
            let output_data = job_processor.run_inner(
                &job_state.input_data,
                data_set,
                schema_set,
                &required_import_data,
                self,
            );

            //TODO: Write to file
            //hydrate_base::uuid_path::uuid_to_path()

            // Send via crossbeam, this will eventually be on a thread pool
            self.job_completed_queue_tx
                .send(CompletedJob {
                    job_id,
                    output_data,
                })
                .unwrap();
        }

        self.handle_completed_queue();
    }

    pub fn stop(&mut self) {
        //TODO: If we have a thread pool do we need to notify them to stop?
        self.clear_create_queue();
        self.handle_completed_queue();

        self.current_jobs.clear();
    }

    pub fn is_idle(&self) -> bool {
        if !self.job_create_queue_rx.is_empty() {
            return false;
        }

        if !self.job_completed_queue_rx.is_empty() {
            return false;
        }

        // if !self.built_asset_queue_rx.is_empty() {
        //     return false;
        // }

        if !self.built_artifact_queue_rx.is_empty() {
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

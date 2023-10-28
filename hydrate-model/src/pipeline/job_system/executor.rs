use std::hash::Hash;
use std::path::PathBuf;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::{Bytes, TypeUuid};
use uuid::Uuid;
use hydrate_base::hashing::HashMap;
use hydrate_base::ObjectId;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use crate::BuiltAsset;

use super::*;
use super::traits::*;

struct BuildJobWrapper<T: BuildJobWithInput>(T);

impl<T: BuildJobWithInput> BuildJobAbstract for BuildJobWrapper<T>
    where
        <T as BuildJobWithInput>::InputT: for<'a> Deserialize<'a> + 'static,
        <T as BuildJobWithInput>::OutputT: Serialize + 'static {
    fn version_inner(&self) -> u32 {
        self.0.version()
    }
    
    fn enumerate_dependencies_inner(&self, input: &Vec<u8>, data_set: &DataSet, schema_set: &SchemaSet) -> JobEnumeratedDependencies {
        let data: <T as BuildJobWithInput>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        self.0.enumerate_dependencies(&data, data_set, schema_set)
    }

    fn run_inner(&self, input: &Vec<u8>, data_set: &DataSet, schema_set: &SchemaSet, dependency_data: &HashMap<ObjectId, SingleObject>, build_job_api: &dyn BuildJobApi) -> Vec<u8> {
        let data: <T as BuildJobWithInput>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        let output = self.0.run(&data, data_set, schema_set, dependency_data, build_job_api);
        bincode::serialize(&output).unwrap()
    }
}

struct JobOutput {
    output_data: Vec<u8>,
    downstream_jobs: Vec<JobId>,
}

struct JobHistory {
    // version() returned from the builder, if it bumps we invalidate the job
    job_version: u32,

    // The dependencies that existed when we ran this job last time (may not need this?)
    dependencies: JobEnumeratedDependencies,
    // Hash of import data used to run the job. If our import data changed, the job results can't be
    // reused
    import_data_hashes: HashMap<ObjectId, u128>,
    // All the jobs this job produced. Even if we can reuse the results of this job, we will have
    // to check downstream jobs do not detect an input data change.
    downstream_jobs: Vec<QueuedJob>,
}

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

pub struct BuildJobExecutor {
    root_path: PathBuf,

    builders: HashMap<JobTypeId, Box<dyn BuildJobAbstract>>,

    // Represents all known previous executions of a job
    job_history: HashMap<JobId, JobHistory>,
    // All the jobs that we have run or will run in this build cycle
    jobs: HashMap<JobId, JobState>,

    // Queue for jobs to request additional jobs to run
    job_create_queue_tx: Sender<QueuedJob>,
    job_create_queue_rx: Receiver<QueuedJob>,

    //TODO: We will have additional deques for jobs that are in a ready state to avoid O(n) iteration

    // Queue for jobs to notify that they have completed
    job_completed_queue_tx: Sender<CompletedJob>,
    job_completed_queue_rx: Receiver<CompletedJob>,
}

impl BuildJobApi for BuildJobExecutor {
    fn enqueue_build_task(&self, data_set: &DataSet, schema_set: &SchemaSet, new_job: NewJob) -> JobId {
        // Dependencies:
        // - Builder/Job Versioning - so if logic changes we can bump version of the builder and kick jobs to rerun
        // - Asset (we need to know hash of data in it)
        // - Import Data (we need to know hash of data in it)
        // - Build Data (we need the build hash, which takes into account the asset/import data
        // - Intermediate data (we need the job's input hash, which takes into account the parameters of the job including
        //   hashes of above stuff
        let job_id = JobId::from_u128(new_job.input_hash);
        let builder = self.builders.get(&new_job.job_type).unwrap();
        let dependencies = builder.enumerate_dependencies_inner(&new_job.input_data, data_set, schema_set);
        self.enqueue_build_task_internal(data_set, schema_set, QueuedJob {
            job_id,
            job_type: new_job.job_type,
            input_data: new_job.input_data,
            dependencies,
        });
        job_id
    }
}

impl BuildJobExecutor {
    pub fn new(root_path: PathBuf) -> Self {
        let (job_create_queue_tx, job_create_queue_rx) = crossbeam_channel::unbounded();
        let (job_completed_queue_tx, job_completed_queue_rx) = crossbeam_channel::unbounded();

        BuildJobExecutor {
            root_path,
            builders: Default::default(),
            job_history: Default::default(),
            jobs: Default::default(),
            job_create_queue_tx,
            job_create_queue_rx,
            job_completed_queue_tx,
            job_completed_queue_rx,
        }
    }

    fn enqueue_build_task_internal(&self, data_set: &DataSet, schema_set: &SchemaSet, job: QueuedJob) {
        self.job_create_queue_tx.send(job).unwrap();
    }

    pub fn register_job_type<T: BuildJobWithInput + 'static>(&mut self, builder: T)
        where
            <T as BuildJobWithInput>::InputT: for<'a> Deserialize<'a>,
            <T as BuildJobWithInput>::OutputT: Serialize,
    {
        self.builders.insert(JobTypeId::from_bytes(T::UUID), Box::new(BuildJobWrapper(builder)));
    }

    fn clear_create_queue(&mut self) {
        while let Ok(queued_job) = self.job_create_queue_rx.try_recv() {
            // do nothing with it
            drop(queued_job);
        }
    }

    fn handle_create_queue(&mut self, data_set: &DataSet, schema_set: &SchemaSet) {
        while let Ok(queued_job) = self.job_create_queue_rx.try_recv() {
            // If key exists, we already queued a job with these exact inputs and we can reuse the outputs
            if !self.jobs.contains_key(&queued_job.job_id) {
                assert!(self.builders.contains_key(&queued_job.job_type));

                self.jobs.insert(queued_job.job_id, JobState {
                    job_type: queued_job.job_type,
                    dependencies: queued_job.dependencies,
                    input_data: queued_job.input_data,
                    output_data: None
                });
            }
        }
    }

    fn handle_completed_queue(&mut self) {
        while let Ok(completed_job) = self.job_completed_queue_rx.try_recv() {
            self.jobs.get_mut(&completed_job.job_id).unwrap().output_data = Some(completed_job.output_data);
        }
    }

    pub fn update(&mut self, data_set: &DataSet, schema_set: &SchemaSet, import_data_provider: &dyn ImportDataProvider) {
        //
        // Pull jobs off the create queue. Determine their dependencies and prepare them to run.
        //
        self.handle_create_queue(data_set, schema_set);

        //TODO: Don't iterate every job in existence
        for (&job_id, job_state) in &self.jobs {
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
                let dependency = self.jobs.get(upstream_job);
                let Some(dependency_job_state) = dependency else {
                    panic!("Job has a dependency on another job that has not been created");
                    //TODO: We would not terminate if we remove the panic
                    break;
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
            let mut has_run_job_before = false;
            let mut can_reuse_result = true;
            let job_history = self.job_history.get(&job_id);
            if let Some(job_history) = job_history {
                has_run_job_before = true;
                can_reuse_result = true;
                //TODO: Check if input data not represented in the job hash changed
                // job_history.import_data_hashed
                // job_history.dependencies

                // can_reuse_result may be set to false here


                if has_run_job_before && can_reuse_result {
                    // Kick off child jobs
                    for downstream_job in &job_history.downstream_jobs {
                        self.enqueue_build_task_internal(data_set, schema_set, downstream_job.clone());
                    }

                    // Bail, we will reuse the output from the previous run
                    break;
                }
            }

            //
            // At this point we have either never run the job before, or we know the job inputs have changed
            // Go ahead and run it.
            //

            //TODO: Read from files

            // Load the import data
            let mut required_import_data = HashMap::default();
            for import_data_id in &job_state.dependencies.import_data {
                let import_data = import_data_provider.load_import_data(schema_set, *import_data_id);
                required_import_data.insert(import_data_id, import_data);
            }

            // Load the upstream job result data


            // Execute the job
            let builder = self.builders.get(&job_state.job_type).unwrap();
            let dependency_data = HashMap::default();
            let output_data = builder.run_inner(&job_state.input_data, data_set, schema_set, &dependency_data, self);

            //TODO: Write to file
            //hydrate_base::uuid_path::uuid_to_path()

            // Send via crossbeam, this will eventually be on a thread pool
            self.job_completed_queue_tx.send(CompletedJob {
                job_id,
                output_data
            }).unwrap();
        }

        self.handle_completed_queue();
    }

    pub fn stop(&mut self) {
        //TODO: If we have a thread pool do we need to notify them to stop?
        self.clear_create_queue();
        self.handle_completed_queue();

        self.jobs.clear();
    }

    pub fn is_idle(&self) -> bool {
        if !self.job_create_queue_rx.is_empty() {
            return false;
        }

        if !self.job_completed_queue_rx.is_empty() {
            return false;
        }

        //TODO: Don't iterate, keep a count
        for (id, job) in &self.jobs {
            if job.output_data.is_none() {
                return false;
            }
        }

        true
    }
}

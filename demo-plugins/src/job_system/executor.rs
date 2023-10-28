use std::hash::Hash;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::{Bytes, TypeUuid};
use uuid::Uuid;
use hydrate_base::hashing::HashMap;
use hydrate_base::ObjectId;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use hydrate_model::BuiltAsset;

use super::traits::*;

struct BuildJobWrapper<T: BuildJobWithInput>(T);

impl<T: BuildJobWithInput> BuildJobAbstract for BuildJobWrapper<T>
    where
        <T as BuildJobWithInput>::InputT: for<'a> Deserialize<'a> + 'static,
        <T as BuildJobWithInput>::OutputT: Serialize + 'static {
    fn enumerate_dependencies_inner(&self, input: &Vec<u8>, data_set: &DataSet, schema_set: &SchemaSet) -> BuildJobRunDependencies {
        let data: <T as BuildJobWithInput>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        self.0.enumerate_dependencies(&data, data_set, schema_set)
    }

    fn run_inner(&self, input: &Vec<u8>, data_set: &DataSet, schema_set: &SchemaSet, dependency_data: &HashMap<ObjectId, SingleObject>, build_job_api: &dyn BuildJobApi) -> Vec<u8> {
        let data: <T as BuildJobWithInput>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        let output = self.0.run(&data, data_set, schema_set, dependency_data, build_job_api);
        bincode::serialize(&output).unwrap()
    }
}



struct JobState {
    job_type: Uuid,
    dependencies: BuildJobRunDependencies,
    input_data: Vec<u8>,
    // This would eventually be stored on file system
    output_data: Option<Vec<u8>>,
}

struct QueuedJob {
    job_type: Uuid,
    job_id: Uuid,
    input_data: Vec<u8>,
    dependencies: BuildJobRunDependencies,
}

struct CompletedJob {
    job_id: Uuid,
    output_data: Vec<u8>,
}

pub struct BuildJobExecutor {
    builders: HashMap<Uuid, Box<dyn BuildJobAbstract>>,
    jobs: HashMap<Uuid, JobState>,

    job_create_queue_tx: Sender<QueuedJob>,
    job_create_queue_rx: Receiver<QueuedJob>,

    // job_ready_queue_tx: Sender<QueuedJob>,
    // job_ready_queue_rx: Receiver<QueuedJob>,
    //
    job_completed_queue_tx: Sender<CompletedJob>,
    job_completed_queue_rx: Receiver<CompletedJob>,
}

impl Default for BuildJobExecutor {
    fn default() -> Self {
        let (job_create_queue_tx, job_create_queue_rx) = crossbeam_channel::unbounded();
        let (job_completed_queue_tx, job_completed_queue_rx) = crossbeam_channel::unbounded();

        BuildJobExecutor {
            builders: Default::default(),
            jobs: Default::default(),
            job_create_queue_tx,
            job_create_queue_rx,
            job_completed_queue_tx,
            job_completed_queue_rx,
        }
    }
}

impl BuildJobApi for BuildJobExecutor {
    fn enqueue_build_task(&self, new_job: NewJob, data_set: &DataSet, schema_set: &SchemaSet) -> Uuid {
        // Dependencies:
        // - Builder/Job Versioning - so if logic changes we can bump version of the builder and kick jobs to rerun
        // - Asset (we need to know hash of data in it)
        // - Import Data (we need to know hash of data in it)
        // - Build Data (we need the build hash, which takes into account the asset/import data
        // - Intermediate data (we need the job's input hash, which takes into account the parameters of the job including
        //   hashes of above stuff
        let job_id = Uuid::from_u128(new_job.input_hash);
        let builder = self.builders.get(&new_job.job_type).unwrap();
        let dependencies = builder.enumerate_dependencies_inner(&new_job.input_data, data_set, schema_set);
        self.job_create_queue_tx.send(QueuedJob {
            job_id,
            job_type: new_job.job_type,
            input_data: new_job.input_data,
            dependencies,
        }).unwrap();
        job_id
    }
}

impl BuildJobExecutor {
    pub fn register_job_type<T: BuildJobWithInput + 'static>(&mut self, builder: T)
        where
            <T as BuildJobWithInput>::InputT: for<'a> Deserialize<'a>,
            <T as BuildJobWithInput>::OutputT: Serialize,
    {
        self.builders.insert(Uuid::from_bytes(T::UUID), Box::new(BuildJobWrapper(builder)));
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

    pub fn update(&mut self, data_set: &DataSet, schema_set: &SchemaSet) {
        //
        // Pull jobs off the create queue. Determine their dependencies and prepare them to run.
        //
        self.handle_create_queue(data_set, schema_set);

        //TODO: Don't iterate every job in existence
        for (&job_id, job_state) in &self.jobs {
            if job_state.output_data.is_some() {
                continue;
            }

            // TODO: Check dependencies
            let mut dependencies_met = true;

            for build_job_dependency in &job_state.dependencies.build_jobs {
                let dependency = self.jobs.get(build_job_dependency);
                let Some(dependency_job_state) = dependency else {
                    panic!("Build job has a dependency on a job that has not been created");
                    dependencies_met = false;
                    break;
                };

                if dependency_job_state.output_data.is_none() {
                    dependencies_met = false;
                    break;
                }
            }

            if !dependencies_met {
                continue;
            }

            // Load the import data
            // Load the dependency data

            let builder = self.builders.get(&job_state.job_type).unwrap();
            let dependency_data = HashMap::default();
            let output_data = builder.run_inner(&job_state.input_data, data_set, schema_set, &dependency_data, self);

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
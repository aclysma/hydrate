
// A build job should have a single output associated with the build job's ID (the ID may be the sha256 hash of job inputs)
// - For example, a job should be able to start job A and job B, and make B a dependent of A
// - job A might produce multiple things, start multiple other jobs, etc. But we don't know that until the job runs,
//   and B needs to be scheduled to run and able to access that later
// Another flow example:
//   - Some prerequisite work needed (optimizing vertex buffers)
//   - Multiple jobs take the work and format it different ways (a job to make position-only data, a job to make index data, a job to make full vertex data, etc.)
//   - A mesh object that needs all of these for the material it plans to use might be kicking it off
//
// Feel like a two-phase structure could work:
// - Enumerate Dependencies (allows requesting arbitrary data to be ready to use when running)
// - Run the job (allowed to fire off subjobs, they get memo-ized)
// - Finalize the job (allowed to read results of created jobs)
// - So if JobA kicks of JobB, JobB kicks of JobC:
//   - JobA: enumerate, run
//   - JobB: enumerate, run
//   - JobC: enumerate, run
//   - JobC: finalize
//   - JobB: finalize
//   - JobA: finalize
//
// One issue is we need a job that hasn't started yet to affect our job's output. Options:
// - We have to return a reference/handle/other form of indirection. We generate handle and child job uses it
// - We have to create an empty object that the child job populates.
//   - We can mark these "promises" as failed
//   - Maps well to the async mindset
//   - How do we handle memo-izing? The child job might be triggered with same input by multiple jobs that all
//     want to create the object to be filled by the job.
// - We have a second pass after the child jobs runs that can take the results of the child job and use them
//   to write the current job's output
//   - How to handle memo-ization. Could we end up with many copies of something?
//     - Yes if we have no way of optionally referencing rather than copying
//   - Might still benefit by passing a promise for an ID that points at some intermediate data, just so
//     we have fine-grained control of dependenies and can get good parallelization
//
// So subjobs should probably be able to create blobs of data referenced by UUID
//
// Could we omit having both run/finalize for simple jobs? Conceptually the run/finalize pair are two
// separate jobs.
//
// We could treat this like having signals/semaphores/promises?
//
// Jobs can create unfulfilled promises and pass them to other jobs
// - If the ID is deterministic based on inputs, we avoid memo-ization challanges
// The child jobs can signal the promises (which also means the data in it has been produced and is available)
// Jobs waiting for promises end up being unblocked



// trait BuildJobContext {
//     fn produce_intermediate_data(&mut self, )
//     fn produce_built_asset(&mut self, built_asset: BuiltAsset);
// }






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


//
// API Design
//
struct BuildJobApi {

}

impl BuildJobApi {
    pub fn enqueue_build_task<'a, T: BuildJobWithInput<'a>>(&self, input: T::InputT) -> Uuid {
        unimplemented!()
    }
}


//
// Job Traits
//
trait BuildJobInput: Hash {

}

trait BuildJobOutput {

}

#[derive(Default)]
struct BuildJobRunDependencies {
    import_data: Vec<ObjectId>,
    build_jobs: Vec<Uuid>,
}

trait BuildJobAbstract {
    fn enumerate_dependencies_inner(
        &self,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies;

    fn run_inner(
        &self,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &mut BuildJobApi
    ) -> Vec<u8>;
}

trait BuildJobWithInput<'a>: TypeUuid {
    type InputT: BuildJobInput + 'static;
    type OutputT: BuildJobOutput + 'static;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies;

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &mut BuildJobApi
    ) -> Self::OutputT;
}

struct BuildJobWrapper<T: for<'a> BuildJobWithInput<'a>>(T);

impl<T: for<'a> BuildJobWithInput<'a>> BuildJobAbstract for BuildJobWrapper<T>
    where
        for<'a> <T as BuildJobWithInput<'a>>::InputT: Deserialize<'a> + 'static,
        for<'a> <T as BuildJobWithInput<'a>>::OutputT: Serialize + 'static {
    fn enumerate_dependencies_inner(&self, input: &Vec<u8>, data_set: &DataSet, schema_set: &SchemaSet) -> BuildJobRunDependencies {
        let data: <T as BuildJobWithInput>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        self.0.enumerate_dependencies(&data, data_set, schema_set)
    }

    fn run_inner(&self, input: &Vec<u8>, data_set: &DataSet, schema_set: &SchemaSet, dependency_data: &HashMap<ObjectId, SingleObject>, build_job_api: &mut BuildJobApi) -> Vec<u8> {
        let data: <T as BuildJobWithInput>::InputT = bincode::deserialize(input.as_slice()).unwrap();
        let output = self.0.run(&data, data_set, schema_set, dependency_data, build_job_api);
        bincode::serialize(&output).unwrap()
    }
}

//
// Example Job Impl - Imagine this kicking off scatter job(s), and then a gather job that produces the final output
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "512f3024-95a8-4b2e-8b4a-cb1111bac30b"]
struct ExampleBuildJobTopLevelInput {
    asset_id: ObjectId,
}
impl BuildJobInput for ExampleBuildJobTopLevelInput {}

struct ExampleBuildJobTopLevelOutput;
impl BuildJobOutput for ExampleBuildJobTopLevelOutput {}

#[derive(TypeUuid)]
#[uuid = "2e2c39f2-e672-4d2f-9d22-9e9ff84adf09"]
struct ExampleBuildJobTopLevel;

impl<'a> BuildJobWithInput<'a> for ExampleBuildJobTopLevel {
    type InputT = ExampleBuildJobTopLevelInput;
    type OutputT = ExampleBuildJobTopLevelOutput;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies {
        unimplemented!()
    }

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &mut BuildJobApi
    ) -> Self::OutputT {
        let task_id1 = build_job_api.enqueue_build_task::<ExampleBuildJobScatter>(ExampleBuildJobScatterInput {
            asset_id: input.asset_id,
            some_other_parameter: "Test1".to_string()
        });
        let task_id2 = build_job_api.enqueue_build_task::<ExampleBuildJobScatter>(ExampleBuildJobScatterInput {
            asset_id: input.asset_id,
            some_other_parameter: "Test2".to_string()
        });
        let task_id3 = build_job_api.enqueue_build_task::<ExampleBuildJobScatter>(ExampleBuildJobScatterInput {
            asset_id: input.asset_id,
            some_other_parameter: "Test3".to_string()
        });

        let final_task = build_job_api.enqueue_build_task::<ExampleBuildJobGather>(ExampleBuildJobGatherInput {
            asset_id: input.asset_id,
            scatter_tasks: vec![task_id1, task_id2, task_id3]
        });

        ExampleBuildJobTopLevelOutput {

        }
    }
}

//
// Example Scatter Job Impl
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "122248a9-9350-4ad7-8ef9-ac3795c08511"]
struct ExampleBuildJobScatterInput {
    asset_id: ObjectId,
    some_other_parameter: String,
}
impl BuildJobInput for ExampleBuildJobScatterInput {}

struct ExampleBuildJobScatterOutput;
impl BuildJobOutput for ExampleBuildJobScatterOutput {}

#[derive(TypeUuid)]
#[uuid = "29755562-5298-4908-8384-7b13b2cedf26"]
struct ExampleBuildJobScatter;

impl<'a> BuildJobWithInput<'a> for ExampleBuildJobScatter {
    type InputT = ExampleBuildJobScatterInput;
    type OutputT = ExampleBuildJobScatterOutput;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies {
        unimplemented!()
    }

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &mut BuildJobApi
    ) -> Self::OutputT {
        //Do stuff
        // We could return the result
        // build_job_api.publish_intermediate_data(...);
        unimplemented!();
    }
}


//
// Example Gather Job Impl
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "f9b45d02-93ba-44df-8252-555f8e01d0b7"]
struct ExampleBuildJobGatherInput {
    asset_id: ObjectId,
    scatter_tasks: Vec<Uuid>,
}
impl BuildJobInput for ExampleBuildJobGatherInput {}

struct ExampleBuildJobGatherOutput;
impl BuildJobOutput for ExampleBuildJobGatherOutput {}

#[derive(TypeUuid)]
#[uuid = "e5f3de94-2bb6-43a9-bea0-cc91467cdcc3"]
struct ExampleBuildJobGather;

impl<'a> BuildJobWithInput<'a> for ExampleBuildJobGather {
    type InputT = ExampleBuildJobGatherInput;
    type OutputT = ExampleBuildJobGatherOutput;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies {
        unimplemented!()
    }

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &mut BuildJobApi
    ) -> Self::OutputT {
        // Now use inputs from other jobs to produce an output
        //build_job_api.publish_built_asset(...);

        unimplemented!();
    }
}

struct JobState {
    input_data: Vec<u8>,
    // This would eventually be stored on file system
    output_data: Option<Vec<u8>>,
}

struct QueuedJob {
    job_type: Uuid,
    job_id: Uuid,
    input_data: Vec<u8>,
}

struct BuildJobExecutor {
    builders: HashMap<Uuid, Box<dyn BuildJobAbstract>>,
    jobs: HashMap<Uuid, JobState>,
    job_create_queue_tx: Sender<QueuedJob>,
    job_create_queue_rx: Receiver<QueuedJob>
}

impl Default for BuildJobExecutor {
    fn default() -> Self {
        let (job_create_queue_tx, job_create_queue_rx) = crossbeam_channel::unbounded();

        BuildJobExecutor {
            builders: Default::default(),
            jobs: Default::default(),
            job_create_queue_tx,
            job_create_queue_rx
        }
    }
}

impl BuildJobExecutor {
    pub fn register_job_type<T: for<'a> BuildJobWithInput<'a> + 'static>(&mut self, builder: T)
        where
            for<'a> <T as BuildJobWithInput<'a>>::InputT: Deserialize<'a>,
            for<'a> <T as BuildJobWithInput<'a>>::OutputT: Serialize,
    {
        self.builders.insert(Uuid::from_bytes(T::UUID), Box::new(BuildJobWrapper(builder)));
    }

    pub fn enqueue_build_task<T: for<'a> BuildJobWithInput<'a>>(&mut self, input: <T as BuildJobWithInput>::InputT) -> Uuid
        where for<'a> <T as BuildJobWithInput<'a>>::InputT: Serialize,
    {
        let mut hasher = siphasher::sip128::SipHasher::default();
        input.hash(&mut hasher);
        let input_hash = hasher.finish128().as_u128();

        let job_id = Uuid::from_u128(input_hash);
        let input_data = bincode::serialize(&input).unwrap();
        self.job_create_queue_tx.send(QueuedJob {
            job_type: Uuid::from_bytes(T::UUID),
            job_id,
            input_data,
        }).unwrap();

        job_id
    }

    pub fn update(&mut self, data_set: &DataSet, schema_set: &SchemaSet) {
        for queued_job in self.job_create_queue_rx.iter() {
            if !self.jobs.contains_key(&queued_job.job_id) {
                let builder = self.builders.get(&queued_job.job_type).unwrap();
                let dependencies = builder.enumerate_dependencies_inner(&queued_job.input_data, data_set, schema_set);

                self.jobs.insert(queued_job.job_id, JobState {
                    input_data: queued_job.input_data,
                    output_data: None
                });
            }
        }
    }

    pub fn is_idle(&self) -> bool {
        if !self.job_create_queue_rx.is_empty() {
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

    pub fn test() {
        let mut executor = BuildJobExecutor::default();
        executor.enqueue_build_task::<ExampleBuildJobTopLevel>(ExampleBuildJobTopLevelInput {
            asset_id: ObjectId::null(),
        });

        let data_set = DataSet::default();
        let schema_set = SchemaSet::default();

        while !executor.is_idle() {
            executor.update(&data_set, &schema_set);
        }

    }
}


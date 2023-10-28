use std::hash::Hash;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::{Bytes, TypeUuid};
use uuid::Uuid;
use hydrate_base::hashing::HashMap;
use hydrate_base::ObjectId;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use crate::{BuiltAsset, ImportData, ImportJobs};
use super::{JobId, JobTypeId};

pub trait ImportDataProvider {
    fn clone_import_data_metadata_hashes(&self) -> HashMap<ObjectId, u64>;

    fn load_import_data(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
    ) -> ImportData;
}

impl ImportDataProvider for ImportJobs {
    fn clone_import_data_metadata_hashes(&self) -> HashMap<ObjectId, u64> {
        self.clone_import_data_metadata_hashes()
    }

    fn load_import_data(&self, schema_set: &SchemaSet, object_id: ObjectId) -> ImportData {
        self.load_import_data(schema_set, object_id)
    }
}

pub struct NewJob {
    pub job_type: JobTypeId,
    pub input_hash: u128,
    pub input_data: Vec<u8>,
}

//
// API Design
//
pub trait BuildJobApi {
    fn enqueue_build_task(&self, data_set: &DataSet, schema_set: &SchemaSet, job: NewJob) -> JobId;
}


//
// Job Traits
//
pub trait BuildJobInput: Hash + Serialize + for<'a> Deserialize<'a> {

}

pub trait BuildJobOutput: Serialize + for<'a> Deserialize<'a> {

}

#[derive(Default, Clone)]
pub struct JobEnumeratedDependencies {
    // The contents of assets can affect the output so we need to include a hash of the contents of
    // the asset. But assets can ref other assets, task needs to list all objects that are touched
    // (including prototypes of those objects).
    //
    // We could do it at asset type granularity? (i.e. if you change an asset of type X all jobs that
    // read an asset of type X have to rerun.
    //
    // What if we provide a data_set reader that keeps track of what was read? When we run the task
    // the first time we don't know what we will touch or how to hash it but we can store it. Second
    // build we can check if anything that was read last time was modified.
    //
    // Alternatively, jobs that read assets must always copy data out of the data set into a hashable
    // form and pass it as input to a job.
    pub import_data: Vec<ObjectId>,
    //pub built_data: Vec<ObjectId>,
    pub upstream_jobs: Vec<JobId>,
}

pub trait BuildJobAbstract {
    fn version_inner(&self) -> u32;

    fn enumerate_dependencies_inner(
        &self,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies;

    fn run_inner(
        &self,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &dyn BuildJobApi
    ) -> Vec<u8>;
}

pub trait BuildJobWithInput: TypeUuid {
    type InputT: BuildJobInput + 'static;
    type OutputT: BuildJobOutput + 'static;

    fn version(&self) -> u32;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies;

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &dyn BuildJobApi
    ) -> Self::OutputT;
}


pub fn enqueue_build_task<T: BuildJobWithInput>(job_api: &dyn BuildJobApi, data_set: &DataSet, schema_set: &SchemaSet, input: <T as BuildJobWithInput>::InputT) -> JobId {
    let mut hasher = siphasher::sip128::SipHasher::default();
    input.hash(&mut hasher);
    let input_hash = hasher.finish128().as_u128();

    let input_data = bincode::serialize(&input).unwrap();

    let queued_job = NewJob {
        job_type: JobTypeId::from_bytes(T::UUID),
        input_hash,
        input_data,
    };

    job_api.enqueue_build_task(data_set, schema_set, queued_job)
}

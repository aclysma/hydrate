use std::hash::Hash;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::{Bytes, TypeUuid, TypeUuidDynamic};
use uuid::Uuid;
use hydrate_base::hashing::HashMap;
use hydrate_base::{AssetUuid, BuiltObjectMetadata, ObjectId};
use hydrate_base::handle::DummySerdeContextHandle;
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
pub trait JobApi {
    fn enqueue_job(&self, data_set: &DataSet, schema_set: &SchemaSet, job: NewJob) -> JobId;

    fn produce_asset(&self, asset: BuiltAsset);
}


//
// Job Traits
//
pub trait JobInput: Hash + Serialize + for<'a> Deserialize<'a> {

}

pub trait JobOutput: Serialize + for<'a> Deserialize<'a> {

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

pub trait JobProcessorAbstract {
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
        job_api: &dyn JobApi
    ) -> Vec<u8>;
}

pub trait JobProcessor: TypeUuid {
    type InputT: JobInput + 'static;
    type OutputT: JobOutput + 'static;

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
        job_api: &dyn JobApi
    ) -> Self::OutputT;
}


pub fn enqueue_job<T: JobProcessor>(
    data_set: &DataSet,
    schema_set: &SchemaSet,
    job_api: &dyn JobApi,
    input: <T as JobProcessor>::InputT
) -> JobId {
    let mut hasher = siphasher::sip128::SipHasher::default();
    input.hash(&mut hasher);
    let input_hash = hasher.finish128().as_u128();

    let input_data = bincode::serialize(&input).unwrap();

    let queued_job = NewJob {
        job_type: JobTypeId::from_bytes(T::UUID),
        input_hash,
        input_data,
    };

    job_api.enqueue_job(data_set, schema_set, queued_job)
}

pub fn produce_asset<T: TypeUuid + Serialize>(
    job_api: &dyn JobApi,
    asset_id: ObjectId,
    asset: T
) {
    produce_asset_with_handles(job_api, asset_id, || asset);
}

pub fn produce_asset_with_handles<T: TypeUuid + Serialize, F: FnOnce() -> T>(
    job_api: &dyn JobApi,
    asset_id: ObjectId,
    asset_fn: F
) {
    let mut ctx = DummySerdeContextHandle::default();
    ctx.begin_serialize_asset(AssetUuid(*asset_id.as_uuid().as_bytes()));

    let (built_data, asset_type) = ctx.scope(|| {
        let asset = (asset_fn)();
        let built_data = bincode::serialize(&asset).unwrap();
        (built_data, asset.uuid())
    });

    let referenced_assets = ctx.end_serialize_asset(AssetUuid(*asset_id.as_uuid().as_bytes()));

    job_api.produce_asset(BuiltAsset {
        asset_id,
        metadata: BuiltObjectMetadata {
            dependencies: referenced_assets.into_iter().map(|x| ObjectId::from_uuid(Uuid::from_bytes(x.0.0))).collect(),
            subresource_count: 0,
            asset_type: uuid::Uuid::from_bytes(asset_type)
        },
        data: built_data
    });
}

use hydrate_model::pipeline::{
    Builder
};
use hydrate_model::{
    job_system, DataContainer, DataSet, HashMap,
    JobApi, JobEnumeratedDependencies, JobInput, JobOutput, JobProcessor,
    AssetId, SchemaSet, SingleObject,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use type_uuid::{Bytes, TypeUuid};

use super::SimpleData;

#[derive(Hash, Serialize, Deserialize)]
pub struct SimpleBincodeDataJobInput {
    pub asset_id: AssetId,
}
impl JobInput for SimpleBincodeDataJobInput {}

#[derive(Serialize, Deserialize)]
pub struct SimpleBincodeDataJobOutput {}
impl JobOutput for SimpleBincodeDataJobOutput {}

pub struct SimpleBincodeDataJobProcessor<
    T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid,
>(PhantomData<T>);
impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> Default
    for SimpleBincodeDataJobProcessor<T>
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> TypeUuid
    for SimpleBincodeDataJobProcessor<T>
{
    const UUID: Bytes = T::UUID;
}

impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> JobProcessor
    for SimpleBincodeDataJobProcessor<T>
{
    type InputT = SimpleBincodeDataJobInput;
    type OutputT = SimpleBincodeDataJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn enumerate_dependencies(
        &self,
        _input: &SimpleBincodeDataJobInput,
        _data_set: &DataSet,
        _schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies::default()
    }

    fn run(
        &self,
        input: &SimpleBincodeDataJobInput,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        _dependency_data: &HashMap<AssetId, SingleObject>,
        job_api: &dyn JobApi,
    ) -> SimpleBincodeDataJobOutput {
        let mut data_set_view = DataContainer::from_dataset(&data_set, schema_set, input.asset_id);

        //
        // Serialize and return
        //
        job_system::produce_asset_with_handles(job_api, input.asset_id, || {
            T::from_data_container(&mut data_set_view, job_api)
        });
        SimpleBincodeDataJobOutput {}
    }
}

//
// Implement SimpleBincodeDataBuilder for all SimpleData
//
pub struct SimpleBincodeDataBuilder<
    T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid,
> {
    asset_type: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid>
    SimpleBincodeDataBuilder<T>
{
    pub fn new(asset_type: &'static str) -> Self {
        SimpleBincodeDataBuilder {
            asset_type,
            phantom_data: PhantomData::default(),
        }
    }
}

impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> Builder
    for SimpleBincodeDataBuilder<T>
{
    fn asset_type(&self) -> &'static str {
        self.asset_type
    }

    fn start_jobs(
        &self,
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
    ) {
        job_system::enqueue_job::<SimpleBincodeDataJobProcessor<T>>(
            data_set,
            schema_set,
            job_api,
            SimpleBincodeDataJobInput { asset_id },
        );
    }
}

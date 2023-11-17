use hydrate_model::pipeline::Builder;
use hydrate_pipeline::{job_system, AssetId, DataContainer, DataSet, HashMap, JobApi, JobEnumeratedDependencies, JobInput, JobOutput, JobProcessor, SchemaSet, SingleObject, BuilderContext, EnumerateDependenciesContext, RunContext};
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
        context: EnumerateDependenciesContext<Self::InputT>,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies::default()
    }

    fn run(
        &self,
        context: RunContext<Self::InputT>,
    ) -> SimpleBincodeDataJobOutput {
        let mut data_set_view = DataContainer::from_dataset(&context.data_set, context.schema_set, context.input.asset_id);

        //
        // Serialize and return
        //
        context.produce_default_artifact_with_handles(context.input.asset_id, |handle_factory| {
            T::from_data_container(&mut data_set_view, handle_factory)
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
        context: BuilderContext
    ) {
        context.enqueue_job::<SimpleBincodeDataJobProcessor<T>>(
            context.data_set,
            context.schema_set,
            context.job_api,
            SimpleBincodeDataJobInput { asset_id: context.asset_id },
        );
    }
}

pub use super::*;
use std::path::{Path};

use demo_types::mesh_adv::*;
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::{BuilderRegistryBuilder, DataContainer, DataContainerMut, DataSet, Enum, HashMap, ImporterRegistryBuilder, job_system, JobApi, JobEnumeratedDependencies, JobInput, JobOutput, JobProcessor, JobProcessorRegistryBuilder, ObjectId, Record, SchemaLinker, SchemaSet, SingleObject};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuiltAsset};
use hydrate_model::pipeline::{ImportedImportable, ScannedImportable, Importer};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use demo_types::gpu_buffer::GpuBufferBuiltData;
use crate::generated::GpuBufferAssetRecord;

use super::generated::{MeshAdvMaterialImportedDataRecord, MeshAdvMaterialAssetRecord, MeshAdvBlendMethodEnum, MeshAdvShadowMethodEnum};

#[derive(Hash, Serialize, Deserialize)]
pub struct GpuBufferJobInput {
    pub asset_id: ObjectId,
}
impl JobInput for GpuBufferJobInput {}

#[derive(Serialize, Deserialize)]
pub struct GpuBufferJobOutput {

}
impl JobOutput for GpuBufferJobOutput {}

#[derive(Default, TypeUuid)]
#[uuid = "91d7931c-7d9a-42f4-a1ed-09cd14fe5210"]
pub struct GpuBufferJobProcessor;

impl JobProcessor for GpuBufferJobProcessor {
    type InputT = GpuBufferJobInput;
    type OutputT = GpuBufferJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn enumerate_dependencies(
        &self,
        input: &GpuBufferJobInput,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies {
            import_data: vec![input.asset_id],
            upstream_jobs: Vec::default()
        }
    }

    fn run(
        &self,
        input: &GpuBufferJobInput,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        job_api: &dyn JobApi
    ) -> GpuBufferJobOutput {
        //
        // Read asset data
        //
        let data_container = DataContainer::new_dataset(data_set, schema_set, input.asset_id);
        let x = GpuBufferAssetRecord::default();

        //let base_color_factor = x.base_color_factor().get_vec4(&data_container).unwrap();

        //
        // Create the processed data
        //
        let processed_data = GpuBufferBuiltData {
            //base_color_factor,
            resource_type: 0,
            alignment: 0,
            data: vec![]
        };

        //
        // Serialize and return
        //
        job_system::produce_asset(job_api, input.asset_id, processed_data);

        GpuBufferJobOutput {

        }
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "3165e557-d356-4191-aa94-83a345ed4b6d"]
pub struct GpuBufferBuilder {}

impl Builder for GpuBufferBuilder {
    fn asset_type(&self) -> &'static str {
        GpuBufferAssetRecord::schema_name()
    }

    fn start_jobs(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi
    ) {
        job_system::enqueue_job::<GpuBufferJobProcessor>(data_set, schema_set, job_api, GpuBufferJobInput {
            asset_id,
        });
    }
}

pub struct GpuBufferAssetPlugin;

impl AssetPlugin for GpuBufferAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        builder_registry.register_handler::<GpuBufferBuilder>(schema_linker);
        job_processor_registry.register_job_processor::<GpuBufferJobProcessor>();
    }
}
pub use super::*;
use std::path::{Path};

use demo_types::mesh_adv::*;
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::{BuilderRegistryBuilder, DataContainer, DataContainerMut, DataSet, Enum, HashMap, ImporterRegistryBuilder, JobApi, JobProcessorRegistryBuilder, ObjectId, Record, SchemaLinker, SchemaSet, SingleObject};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuiltAsset};
use hydrate_model::pipeline::{ImportedImportable, ScannedImportable, Importer};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use demo_types::gpu_buffer::GpuBufferBuiltData;
use crate::generated::GpuBufferAssetRecord;

use super::generated::{MeshAdvMaterialImportedDataRecord, MeshAdvMaterialAssetRecord, MeshAdvBlendMethodEnum, MeshAdvShadowMethodEnum};



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

    }

    fn enumerate_dependencies(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> Vec<ObjectId> {
        vec![]
    }

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
    ) -> BuiltAsset {
        //
        // Read asset data
        //
        let data_container = DataContainer::new_dataset(data_set, schema_set, asset_id);
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
        let serialized = bincode::serialize(&processed_data).unwrap();
        BuiltAsset {
            asset_id,
            metadata: BuiltObjectMetadata {
                dependencies: vec![],
                subresource_count: 0,
                asset_type: uuid::Uuid::from_bytes(processed_data.uuid())
            },
            data: serialized
        }
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
    }
}

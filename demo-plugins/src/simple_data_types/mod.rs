use demo_types::simple_data::*;
use hydrate_base::AssetUuid;
use hydrate_model::{BuilderRegistryBuilder, DataContainer, ImporterRegistryBuilder, JobProcessorRegistryBuilder, SchemaLinker};
use hydrate_model::pipeline::{AssetPlugin, Builder};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use super::generated::{AllFieldsRecord, TransformRecord, TransformRefRecord};

mod simple_data_trait;
pub use simple_data_trait::SimpleData;

mod bincode_data_builder;
use bincode_data_builder::{SimpleBincodeDataJobProcessor, SimpleBincodeDataBuilder};

impl SimpleData for TransformRef {
    fn from_data_container(
        data_set_view: &DataContainer,
    ) -> Self {
        let x = TransformRefRecord::default();
        let transform = x.transform().get(data_set_view).unwrap();

        //TODO: Verify type?
        let asset_id = AssetUuid(*transform.as_uuid().as_bytes());
        let handle = hydrate_base::handle::make_handle::<Transform>(asset_id);

        TransformRef {
            transform: handle
        }
    }
}

impl SimpleData for Transform {
    fn from_data_container(
        data_container: &DataContainer,
    ) -> Self {
        let x = TransformRecord::default();
        let position = x.position().get_vec3(data_container).unwrap();
        let rotation = x.rotation().get_vec4(data_container).unwrap();
        let scale = x.scale().get_vec3(data_container).unwrap();

        Transform {
            position,
            rotation,
            scale,
        }
    }
}

impl SimpleData for AllFields {
    fn from_data_container(
        data_container: &DataContainer,
    ) -> Self {
        let x = AllFieldsRecord::default();
        let boolean = x.boolean().get(data_container).unwrap();
        let int32 = x.i32().get(data_container).unwrap();
        let int64 = x.i64().get(data_container).unwrap();

        AllFields {
            boolean,
            int32,
            int64,
        }
    }
}

pub struct SimpleDataAssetPlugin;

impl AssetPlugin for SimpleDataAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        macro_rules! register {
            ($data_type:ty, $name:literal) => {
                builder_registry.register_handler_instance(
                    schema_linker,
                    SimpleBincodeDataBuilder::<$data_type>::new($name),
                );
                job_processor_registry.register_job_processor::<SimpleBincodeDataJobProcessor::<$data_type>>()
            }
        }

        register!(AllFields, "AllFields");
        register!(Transform, "Transform");
        register!(TransformRef, "TransformRef");
    }
}

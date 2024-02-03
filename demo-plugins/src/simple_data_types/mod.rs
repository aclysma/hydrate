use super::generated::{AllFieldsAccessor, TransformAccessor, TransformRefAccessor};
use demo_types::simple_data::*;
use hydrate_model::pipeline::AssetPlugin;
use hydrate_pipeline::{
    AssetPluginSetupContext, BuilderRegistryBuilder, DataContainerRef, HandleFactory,
    ImporterRegistryBuilder, JobProcessorRegistryBuilder, PipelineResult,
};

mod simple_data_trait;
pub use simple_data_trait::SimpleData;

mod bincode_data_builder;
use bincode_data_builder::{SimpleBincodeDataBuilder, SimpleBincodeDataJobProcessor};

impl SimpleData for TransformRef {
    fn from_data_container(
        data_set_view: DataContainerRef,
        handle_context: HandleFactory,
    ) -> PipelineResult<Self> {
        let x = TransformRefAccessor::default();
        let transform = x.transform().get(data_set_view)?;

        //TODO: Verify type?
        let handle = handle_context.make_handle_to_default_artifact(transform);

        Ok(TransformRef { transform: handle })
    }
}

impl SimpleData for Transform {
    fn from_data_container(
        data_container: DataContainerRef,
        _handle_context: HandleFactory,
    ) -> PipelineResult<Self> {
        let x = TransformAccessor::default();
        let position = x.position().get_vec3(data_container.clone())?;
        let rotation = x.rotation().get_vec4(data_container.clone())?;
        let scale = x.scale().get_vec3(data_container.clone())?;

        Ok(Transform {
            position,
            rotation,
            scale,
        })
    }
}

impl SimpleData for AllFields {
    fn from_data_container(
        data_container: DataContainerRef,
        _handle_context: HandleFactory,
    ) -> PipelineResult<Self> {
        let x = AllFieldsAccessor::default();
        let boolean = x.boolean().get(data_container.clone())?;
        let int32 = x.i32().get(data_container.clone())?;
        let int64 = x.i64().get(data_container.clone())?;

        Ok(AllFields {
            boolean,
            int32,
            int64,
        })
    }
}

pub struct SimpleDataAssetPlugin;

impl AssetPlugin for SimpleDataAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        macro_rules! register {
            ($data_type:ty, $name:literal) => {
                context
                    .builder_registry
                    .register_handler_instance(SimpleBincodeDataBuilder::<$data_type>::new($name));
                context
                    .job_processor_registry
                    .register_job_processor::<SimpleBincodeDataJobProcessor<$data_type>>()
            };
        }

        register!(AllFields, "AllFields");
        register!(Transform, "Transform");
        register!(TransformRef, "TransformRef");
    }
}

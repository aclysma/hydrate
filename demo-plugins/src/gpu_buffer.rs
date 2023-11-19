pub use super::*;

use crate::generated::GpuBufferAssetAccessor;
use demo_types::gpu_buffer::GpuBufferBuiltData;
use hydrate_model::pipeline::{AssetPlugin, Builder};
use hydrate_pipeline::{
    AssetId, BuilderContext, BuilderRegistryBuilder, EnumerateDependenciesContext,
    ImporterRegistryBuilder, JobEnumeratedDependencies, JobInput, JobOutput, JobProcessor,
    JobProcessorRegistryBuilder, PipelineResult, RecordAccessor, RunContext, SchemaLinker,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Hash, Serialize, Deserialize)]
pub struct GpuBufferJobInput {
    pub asset_id: AssetId,
}
impl JobInput for GpuBufferJobInput {}

#[derive(Serialize, Deserialize)]
pub struct GpuBufferJobOutput {}
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
        context: EnumerateDependenciesContext<Self::InputT>,
    ) -> PipelineResult<JobEnumeratedDependencies> {
        // No dependencies
        Ok(JobEnumeratedDependencies {
            import_data: vec![context.input.asset_id],
            upstream_jobs: Vec::default(),
        })
    }

    fn run(
        &self,
        context: RunContext<Self::InputT>,
    ) -> PipelineResult<GpuBufferJobOutput> {
        //
        // Create the processed data
        //
        let processed_data = GpuBufferBuiltData {
            resource_type: 0,
            alignment: 0,
            data: vec![],
        };

        //
        // Serialize and return
        //
        context.produce_default_artifact(context.input.asset_id, processed_data)?;

        Ok(GpuBufferJobOutput {})
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "3165e557-d356-4191-aa94-83a345ed4b6d"]
pub struct GpuBufferBuilder {}

impl Builder for GpuBufferBuilder {
    fn asset_type(&self) -> &'static str {
        GpuBufferAssetAccessor::schema_name()
    }

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) -> PipelineResult<()> {
        context.enqueue_job::<GpuBufferJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            GpuBufferJobInput {
                asset_id: context.asset_id,
            },
        )?;
        Ok(())
    }
}

pub struct GpuBufferAssetPlugin;

impl AssetPlugin for GpuBufferAssetPlugin {
    fn setup(
        _schema_linker: &mut SchemaLinker,
        _importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        builder_registry.register_handler::<GpuBufferBuilder>();
        job_processor_registry.register_job_processor::<GpuBufferJobProcessor>();
    }
}

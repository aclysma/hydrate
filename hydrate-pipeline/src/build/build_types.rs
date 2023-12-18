use super::{JobApi, JobId, JobProcessor, JobRequestor};
use hydrate_base::{ArtifactId, BuiltArtifactHeaderData};
use hydrate_data::{AssetId, DataSet, SchemaSet};
use crate::PipelineResult;

pub struct BuiltAsset {
    pub asset_id: AssetId,
    pub metadata: BuiltArtifactHeaderData,
    pub data: Vec<u8>,
}

pub struct BuiltArtifact {
    pub asset_id: AssetId,
    pub artifact_id: ArtifactId,
    pub metadata: BuiltArtifactHeaderData,
    pub data: Vec<u8>,
    pub artifact_key_debug_name: Option<String>,
}

pub struct WrittenArtifact {
    pub asset_id: AssetId,
    pub artifact_id: ArtifactId,
    pub metadata: BuiltArtifactHeaderData,
    pub build_hash: u64,
    pub artifact_key_debug_name: Option<String>,
}

pub struct BuilderContext<'a> {
    pub asset_id: AssetId,
    pub data_set: &'a DataSet,
    pub schema_set: &'a SchemaSet,
    pub job_api: &'a dyn JobApi,
}

impl<'a> BuilderContext<'a> {
    pub fn enqueue_job<JobProcessorT: JobProcessor>(
        &self,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
        input: <JobProcessorT as JobProcessor>::InputT,
    ) -> PipelineResult<JobId> {
        super::job_system::enqueue_job::<JobProcessorT>(JobRequestor::Builder(self.asset_id), data_set, schema_set, job_api, input)
    }
}

// Interface all builders must implement
pub trait Builder {
    // The type of asset that this builder handles
    fn asset_type(&self) -> &'static str;

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) -> PipelineResult<()>;
}

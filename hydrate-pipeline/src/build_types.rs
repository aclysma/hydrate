use crate::JobApi;
use hydrate_base::{ArtifactId, BuiltArtifactMetadata};
use hydrate_data::{AssetId, DataSet, SchemaSet};

pub struct BuiltAsset {
    pub asset_id: AssetId,
    pub metadata: BuiltArtifactMetadata,
    pub data: Vec<u8>,
}

pub struct BuiltArtifact {
    pub asset_id: AssetId,
    pub artifact_id: ArtifactId,
    pub metadata: BuiltArtifactMetadata,
    pub data: Vec<u8>,
    pub artifact_key_debug_name: Option<String>,
}

pub struct WrittenArtifact {
    pub asset_id: AssetId,
    pub artifact_id: ArtifactId,
    pub metadata: BuiltArtifactMetadata,
    pub build_hash: u64,
    pub artifact_key_debug_name: Option<String>,
}

pub struct BuilderContext<'a> {
    pub asset_id: AssetId,
    pub data_set: &'a DataSet,
    pub schema_set: &'a SchemaSet,
    pub job_api: &'a dyn JobApi,
}

// Interface all builders must implement
pub trait Builder {
    // The type of asset that this builder handles
    fn asset_type(&self) -> &'static str;

    fn start_jobs(
        &self,
        context: BuilderContext
    );

    // Returns the assets that this build job needs to be available to complete
    // fn enumerate_dependencies(
    //     &self,
    //     asset_id: AssetId,
    //     data_set: &DataSet,
    //     schema_set: &SchemaSet,
    // ) -> Vec<AssetId>;

    // fn build_asset(
    //     &self,
    //     asset_id: AssetId,
    //     data_set: &DataSet,
    //     schema_set: &SchemaSet,
    //     dependency_data: &HashMap<AssetId, SingleObject>,
    // ) -> BuiltAsset;
}
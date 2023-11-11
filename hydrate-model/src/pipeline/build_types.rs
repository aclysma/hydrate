use crate::{DataSet, JobApi, AssetId, SchemaSet};
use hydrate_base::{ArtifactId, BuiltArtifactMetadata};

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
}

// Interface all builders must implement
pub trait Builder {
    // The type of asset that this builder handles
    fn asset_type(&self) -> &'static str;

    fn start_jobs(
        &self,
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
    );

    fn is_job_based(&self) -> bool {
        true
    }

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

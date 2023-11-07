use crate::{DataSet, JobApi, ObjectId, SchemaSet};
use hydrate_base::{ArtifactId, BuiltObjectMetadata};

pub struct BuiltAsset {
    pub asset_id: ObjectId,
    pub metadata: BuiltObjectMetadata,
    pub data: Vec<u8>,
}

pub struct BuiltArtifact {
    pub asset_id: ObjectId,
    pub artifact_id: ArtifactId,
    pub metadata: BuiltObjectMetadata,
    pub data: Vec<u8>,
}

// Interface all builders must implement
pub trait Builder {
    // The type of asset that this builder handles
    fn asset_type(&self) -> &'static str;

    fn start_jobs(
        &self,
        asset_id: ObjectId,
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
    //     asset_id: ObjectId,
    //     data_set: &DataSet,
    //     schema_set: &SchemaSet,
    // ) -> Vec<ObjectId>;

    // fn build_asset(
    //     &self,
    //     asset_id: ObjectId,
    //     data_set: &DataSet,
    //     schema_set: &SchemaSet,
    //     dependency_data: &HashMap<ObjectId, SingleObject>,
    // ) -> BuiltAsset;
}

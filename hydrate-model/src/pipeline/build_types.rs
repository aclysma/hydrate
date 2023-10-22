use crate::{BuildInfo, BuilderId, DataSet, DataSource, EditorModel, HashMap, HashMapKeys, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint, SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Write};
use std::path::{Path, PathBuf};
use hydrate_base::{BuiltObjectMetadata, AssetUuid};
use hydrate_base::handle::DummySerdeContextHandle;

use super::ImportJobs;

use hydrate_base::uuid_path::{path_to_uuid, uuid_and_hash_to_path, uuid_to_path};

pub struct BuiltAsset {
    pub metadata: BuiltObjectMetadata,
    pub data: Vec<u8>
}

// Interface all builders must implement
pub trait Builder {
    // The type of asset that this builder handles
    fn asset_type(&self) -> &'static str;

    // Returns the assets that this build job needs to be available to complete
    fn enumerate_dependencies(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> Vec<ObjectId>;

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
    ) -> BuiltAsset;
}

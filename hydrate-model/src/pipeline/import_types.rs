use crate::{
    DataSet, DataSource, EditorModel, HashMap, HashMapKeys, ImportInfo,
    ImporterId, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint,
    SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use type_uuid::{TypeUuid, TypeUuidDynamic};
use uuid::Uuid;

use crate::edit_context::EditContext;
use crate::SingleObjectJson;
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};


// Represents a path to another file encountered in a file that will need to be resolved to an asset
// at build time
#[derive(Debug)]
pub struct ReferencedSourceFile {
    pub importer_id: ImporterId,
    pub path: PathBuf,
}

// Metadata for all importable data from a file. For example, a GLTF could contain textures, meshes,
// materials, etc.
#[derive(Debug)]
pub struct ScannedImportable {
    pub name: Option<String>,
    pub asset_type: SchemaRecord,
    pub file_references: Vec<ReferencedSourceFile>,
}

pub struct ImportedImportable {
    pub file_references: Vec<ReferencedSourceFile>,
    pub default_asset: SingleObject,
    pub import_data: SingleObject,
}


// Interface all importers must implement
pub trait Importer: TypeUuidDynamic {
    fn importer_id(&self) -> ImporterId {
        ImporterId(Uuid::from_bytes(self.uuid()))
    }

    // Used to allow the importer registry to return all importers compatible with a given filename extension
    fn supported_file_extensions(&self) -> &[&'static str];

    // Open the file and determine what assets exist in it that can be imported
    fn scan_file(
        &self,
        path: &Path,
        schema_set: &SchemaSet,
    ) -> Vec<ScannedImportable>;

    // Open the file and extract all the data from it required for the build step, or for build
    // steps for assets referencing this asset
    fn import_file(
        &self,
        path: &Path,
        object_ids: &HashMap<Option<String>, ObjectId>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
        //referenced_source_file_paths: &mut Vec<PathBuf>,
    ) -> HashMap<Option<String>, ImportedImportable>;
}

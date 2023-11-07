use crate::{
    HashMap, ImporterId, ImporterRegistry, ObjectId, SchemaRecord, SchemaSet, SingleObject,
};
use std::path::{Path, PathBuf};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use uuid::Uuid;

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
    pub default_asset: Option<SingleObject>,
    pub import_data: Option<SingleObject>,
}

pub trait ImporterStatic: TypeUuid {
    fn importer_id() -> ImporterId {
        ImporterId(Uuid::from_bytes(Self::UUID))
    }
}

// #[derive(PartialEq, Eq, Hash, Debug, Clone)]
// pub struct FileReferenceKey {
//     // The object ID that is referencing something by path
//     pub importable_name: String,
//     // The path used to reference some other object
//     pub referenced_path: PathBuf,
// }

pub struct ImportableObject {
    pub id: ObjectId,
    pub referenced_paths: HashMap<PathBuf, ObjectId>,
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
        importer_registry: &ImporterRegistry,
    ) -> Vec<ScannedImportable>;

    // Open the file and extract all the data from it required for the build step, or for build
    // steps for assets referencing this asset
    fn import_file(
        &self,
        path: &Path,
        importable_objects: &HashMap<Option<String>, ImportableObject>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
        //referenced_source_file_paths: &mut Vec<PathBuf>,
    ) -> HashMap<Option<String>, ImportedImportable>;
}

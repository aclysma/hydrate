use crate::{ImporterRegistry, PipelineResult};
use hydrate_data::{AssetId, HashMap, ImporterId, SchemaRecord, SchemaSet, SingleObject};
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
//     // The asset ID that is referencing something by path
//     pub importable_name: String,
//     // The path used to reference some other asset
//     pub referenced_path: PathBuf,
// }

pub struct ImportableAsset {
    pub id: AssetId,
    pub referenced_paths: HashMap<PathBuf, AssetId>,
}

pub struct ScanContext<'a> {
    pub path: &'a Path,
    pub schema_set: &'a SchemaSet,
    pub importer_registry: &'a ImporterRegistry,
}

pub struct ImportContext<'a> {
    pub path: &'a Path,
    pub importable_assets: &'a HashMap<Option<String>, ImportableAsset>,
    pub schema_set: &'a SchemaSet,
}

// Interface all importers must implement
pub trait Importer: TypeUuidDynamic + Sync + Send + 'static {
    fn importer_id(&self) -> ImporterId {
        ImporterId(Uuid::from_bytes(self.uuid()))
    }

    // Used to allow the importer registry to return all importers compatible with a given filename extension
    fn supported_file_extensions(&self) -> &[&'static str];

    // Open the file and determine what assets exist in it that can be imported
    fn scan_file(
        &self,
        context: ScanContext,
    ) -> PipelineResult<Vec<ScannedImportable>>;

    // Open the file and extract all the data from it required for the build step, or for build
    // steps for assets referencing this asset
    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<HashMap<Option<String>, ImportedImportable>>;
}

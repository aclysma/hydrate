use crate::{ImporterRegistry, PipelineResult};
use hydrate_data::{
    AssetId, HashMap, ImporterId, RecordOwned, SchemaRecord, SchemaSet, SingleObject,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
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
    pub default_asset: SingleObject,
    pub import_data: Option<SingleObject>,
}

pub trait ImporterStatic: TypeUuid {
    fn importer_id() -> ImporterId {
        ImporterId(Uuid::from_bytes(Self::UUID))
    }
}

pub struct ImportableAsset {
    pub id: AssetId,
    pub referenced_paths: HashMap<PathBuf, AssetId>,
}

#[derive(Clone)]
pub struct ScanContext<'a> {
    pub path: &'a Path,
    pub schema_set: &'a SchemaSet,
    pub importer_registry: &'a ImporterRegistry,
    pub(crate) scanned_importables: Rc<RefCell<&'a mut HashMap<Option<String>, ScannedImportable>>>,
}

pub struct ScanContextImportable<'a> {
    context: ScanContext<'a>,
    importable_name: Option<String>,
}

impl<'a> ScanContext<'a> {
    pub fn new(
        path: &'a Path,
        schema_set: &'a SchemaSet,
        importer_registry: &'a ImporterRegistry,
        scanned_importables: &'a mut HashMap<Option<String>, ScannedImportable>,
    ) -> ScanContext<'a> {
        ScanContext {
            path,
            schema_set,
            importer_registry,
            scanned_importables: Rc::new(RefCell::new(scanned_importables)),
        }
    }

    pub fn add_importable<T: RecordOwned>(
        &self,
        name: Option<String>,
    ) -> PipelineResult<ScanContextImportable<'a>> {
        let asset_type = self
            .schema_set
            .find_named_type(T::schema_name())?
            .as_record()?
            .clone();

        self.add_importable_with_record(name, asset_type)
    }

    pub fn add_importable_with_record(
        &self,
        name: Option<String>,
        schema_record: SchemaRecord,
    ) -> PipelineResult<ScanContextImportable<'a>> {
        let scanned_importable = ScannedImportable {
            name: name.clone(),
            asset_type: schema_record,
            file_references: Default::default(),
        };
        if self.scanned_importables.borrow().contains_key(&name) {
            Err(format!("The importable {:?} was added twice", name))?;
        }
        let old = self
            .scanned_importables
            .borrow_mut()
            .insert(name.clone(), scanned_importable);
        assert!(old.is_none());
        Ok(ScanContextImportable {
            context: self.clone(),
            importable_name: name.clone(),
        })
    }

    pub fn add_file_reference_with_importer_id<PathT: Into<PathBuf>>(
        &self,
        name: Option<String>,
        path: PathT,
        importer_id: ImporterId,
    ) -> PipelineResult<()> {
        let file_reference = ReferencedSourceFile {
            importer_id,
            path: path.into(),
        };

        self.scanned_importables
            .borrow_mut()
            .get_mut(&name)
            .ok_or_else(|| format!("Trying to add file reference for importable named '{:?}'. The importable must be added before adding path references", name))?
            .file_references.push(file_reference);

        Ok(())
    }

    pub fn add_file_reference<PathT: Into<PathBuf>>(
        &self,
        name: Option<String>,
        path: PathT,
    ) -> PipelineResult<()> {
        let path = path.into();
        let importer = self.importer_registry.importers_for_file_extension(
            path.extension()
                .ok_or("File has no extension, cannot determine importer to use")?
                .to_str()
                .ok_or("File extension cannot be converted to string")?,
        );

        if importer.len() == 0 {
            Err(format!(
                "No importer found for file extension {:?} in path {:?}",
                path.extension(),
                path
            ))?;
        }

        if importer.len() > 1 {
            Err(format!(
                "Multiple importers found for file extension {:?} in path {:?}",
                path.extension(),
                path
            ))?;
        }

        self.add_file_reference_with_importer_id(name, path, importer[0])
    }

    pub fn add_file_reference_with_importer<ImporterT: TypeUuid, PathT: Into<PathBuf>>(
        &self,
        name: Option<String>,
        path: PathT,
    ) -> PipelineResult<()> {
        self.add_file_reference_with_importer_id(
            name,
            path,
            ImporterId(Uuid::from_bytes(ImporterT::UUID)),
        )
    }
}

impl<'a> ScanContextImportable<'a> {
    pub fn add_file_reference_with_importer_id<PathT: Into<PathBuf>>(
        &self,
        path: PathT,
        importer_id: ImporterId,
    ) -> PipelineResult<&Self> {
        self.context.add_file_reference_with_importer_id(
            self.importable_name.clone(),
            path,
            importer_id,
        )?;
        Ok(self)
    }

    pub fn add_file_reference<PathT: Into<PathBuf>>(
        &self,
        path: PathT,
    ) -> PipelineResult<&Self> {
        self.context
            .add_file_reference(self.importable_name.clone(), path)?;
        Ok(self)
    }

    pub fn add_file_reference_with_importer<ImporterT: TypeUuid, PathT: Into<PathBuf>>(
        &self,
        path: PathT,
    ) -> PipelineResult<&Self> {
        self.context
            .add_file_reference_with_importer::<ImporterT, _>(self.importable_name.clone(), path)?;
        Ok(self)
    }
}

#[derive(Clone)]
pub struct ImportContext<'a> {
    pub path: &'a Path,
    importable_assets: &'a HashMap<Option<String>, ImportableAsset>,
    pub schema_set: &'a SchemaSet,
    imported_importables: Rc<RefCell<&'a mut HashMap<Option<String>, ImportedImportable>>>,
}

impl<'a> ImportContext<'a> {
    pub fn new(
        path: &'a Path,
        importable_assets: &'a HashMap<Option<String>, ImportableAsset>,
        schema_set: &'a SchemaSet,
        imported_importables: &'a mut HashMap<Option<String>, ImportedImportable>,
    ) -> ImportContext<'a> {
        ImportContext {
            path,
            importable_assets,
            schema_set,
            imported_importables: Rc::new(RefCell::new(imported_importables)),
        }
    }

    pub fn add_importable(
        &self,
        name: Option<String>,
        asset: SingleObject,
        import_data: Option<SingleObject>,
    ) {
        let old = self.imported_importables.borrow_mut().insert(
            name,
            ImportedImportable {
                import_data,
                default_asset: asset,
            },
        );
        assert!(old.is_none());
    }

    pub fn should_import(
        &self,
        name: &Option<String>,
    ) -> bool {
        self.importable_assets.contains_key(name)
    }

    // This is for assets by this import job
    pub fn asset_id_for_importable(
        &self,
        name: &Option<String>,
    ) -> Option<AssetId> {
        self.importable_assets.get(name).map(|x| x.id)
    }

    // This is for assets produced by importing other files
    pub fn asset_id_for_referenced_file_path(
        &self,
        name: Option<String>,
        path: &Path,
    ) -> PipelineResult<AssetId> {
        Ok(*self.importable_assets
            .get(&name)
            .ok_or_else(|| format!("Default importable not found when trying to resolve path {:?} referenced by importable {:?}", path, name))?
            .referenced_paths
            .get(path)
            .ok_or_else(|| format!("No asset ID found for default importable when trying to resolve path {:?} referenced by importable {:?}", path, name))?)
    }
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
    ) -> PipelineResult<()>;

    // Open the file and extract all the data from it required for the build step, or for build
    // steps for assets referencing this asset
    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<()>;
}

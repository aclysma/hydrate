use crate::{HydrateProjectConfiguration, ImporterRegistry, PipelineResult};
use hydrate_data::{AssetId, CanonicalPathReference, HashMap, ImportableName, ImporterId, PathReference, Record, SchemaRecord, SchemaSet, SingleObject};
use std::cell::RefCell;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use siphasher::sip128::Hasher128;
use type_uuid::{TypeUuid, TypeUuidDynamic};
use uuid::Uuid;

// Represents a path to another file encountered in a file that will need to be resolved to an asset
// at build time
#[derive(Debug)]
pub struct SourceFileWithImporter {
    pub path_reference: CanonicalPathReference,
    pub importer_id: ImporterId,
}

// Metadata for all importable data from a file. For example, a GLTF could contain textures, meshes,
// materials, etc.
#[derive(Debug)]
pub struct ScannedImportable {
    pub name: ImportableName,
    pub asset_type: SchemaRecord,
    pub referenced_source_files: HashMap<Uuid, CanonicalPathReference>,
    pub referenced_source_file_info: HashMap<CanonicalPathReference, ImporterId>,
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

#[derive(Debug)]
pub struct ImportableAsset {
    pub id: AssetId,
    pub canonical_path_references: HashMap<CanonicalPathReference, AssetId>,
    pub path_references: HashMap<Uuid, CanonicalPathReference>,
}

#[derive(Clone)]
pub struct ScanContext<'a> {
    pub path: &'a Path,
    pub schema_set: &'a SchemaSet,
    pub importer_registry: &'a ImporterRegistry,
    project_config: &'a HydrateProjectConfiguration,
    pub(crate) scanned_importables: Rc<RefCell<&'a mut HashMap<ImportableName, ScannedImportable>>>,
}

pub struct ScanContextImportable<'a> {
    context: ScanContext<'a>,
    importable_name: ImportableName,
}

impl<'a> ScanContext<'a> {
    pub fn new(
        path: &'a Path,
        schema_set: &'a SchemaSet,
        importer_registry: &'a ImporterRegistry,
        project_config: &'a HydrateProjectConfiguration,
        scanned_importables: &'a mut HashMap<ImportableName, ScannedImportable>,
    ) -> ScanContext<'a> {
        ScanContext {
            path,
            schema_set,
            importer_registry,
            project_config,
            scanned_importables: Rc::new(RefCell::new(scanned_importables)),
        }
    }

    pub fn add_importable<T: Record>(
        &self,
        name: ImportableName,
    ) -> PipelineResult<ScanContextImportable<'a>> {
        let asset_type = self
            .schema_set
            .find_named_type(T::schema_name())?
            .as_record()?
            .clone();

        self.add_importable_with_record(name, asset_type)
    }

    pub fn add_default_importable<T: Record>(&self) -> PipelineResult<ScanContextImportable<'a>> {
        self.add_importable::<T>(ImportableName::default())
    }

    pub fn add_importable_with_record(
        &self,
        name: ImportableName,
        schema_record: SchemaRecord,
    ) -> PipelineResult<ScanContextImportable<'a>> {
        let scanned_importable = ScannedImportable {
            name: name.clone(),
            asset_type: schema_record,
            referenced_source_files: Default::default(),
            referenced_source_file_info: Default::default(),
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

    pub fn add_file_reference_with_importer_id<PathT: Into<PathReference>>(
        &self,
        name: ImportableName,
        path_reference: PathT,
        importer_id: ImporterId,
    ) -> PipelineResult<()> {
        let path_reference: PathReference = path_reference.into();
        let canonical_path_reference = path_reference.clone().simplify(self.project_config);
        // let file_reference = SourceFileWithImporter {
        //     importer_id,
        //     path_reference: path_reference.into().simplify(self.project_config),
        // };

        let mut scanned_importables = self.scanned_importables
            .borrow_mut();

        let importable = scanned_importables
            .get_mut(&name)
            .ok_or_else(|| format!("Trying to add file reference for importable named '{:?}'. The importable must be added before adding path references", name))?;

        let old_importer_id = importable.referenced_source_file_info.insert(canonical_path_reference.clone(), importer_id);
        if let Some(old_importer_id) = old_importer_id {
            if old_importer_id != importer_id {
                Err(format!("The referenced file {:?} has been requested with different importers", canonical_path_reference.to_string()))?;
            }
        }

        let mut hasher = siphasher::sip128::SipHasher::default();
        path_reference.to_string().hash(&mut hasher);
        let path_reference_hash = Uuid::from_u128(hasher.finish128().as_u128());
        importable.referenced_source_files.insert(path_reference_hash, canonical_path_reference);

        Ok(())
    }

    pub fn add_file_reference<PathT: Into<PathReference>>(
        &self,
        name: ImportableName,
        path: PathT,
    ) -> PipelineResult<()> {
        let path = path.into();
        let extension = PathBuf::from(path.path())
            .extension()
            .ok_or("File has no extension, cannot determine importer to use")?
            .to_str()
            .ok_or("File extension cannot be converted to string")?
            .to_string();

        let importer = self
            .importer_registry
            .importers_for_file_extension(&extension);

        if importer.len() == 0 {
            Err(format!(
                "No importer found for file extension {:?} in path {:?}",
                extension, path
            ))?;
        }

        if importer.len() > 1 {
            Err(format!(
                "Multiple importers found for file extension {:?} in path {:?}",
                extension, path
            ))?;
        }

        self.add_file_reference_with_importer_id(name, path, importer[0])
    }

    pub fn add_file_reference_with_importer<ImporterT: TypeUuid, PathT: Into<PathReference>>(
        &self,
        name: ImportableName,
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
    pub fn add_file_reference_with_importer_id<PathT: Into<PathReference>>(
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

    pub fn add_file_reference<PathT: Into<PathReference>>(
        &self,
        path: PathT,
    ) -> PipelineResult<&Self> {
        self.context
            .add_file_reference(self.importable_name.clone(), path)?;
        Ok(self)
    }

    pub fn add_file_reference_with_importer<ImporterT: TypeUuid, PathT: Into<PathReference>>(
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
    importable_assets: &'a HashMap<ImportableName, ImportableAsset>,
    pub schema_set: &'a SchemaSet,
    project_config: &'a HydrateProjectConfiguration,
    imported_importables: Rc<RefCell<&'a mut HashMap<ImportableName, ImportedImportable>>>,
}

impl<'a> ImportContext<'a> {
    pub fn new(
        path: &'a Path,
        importable_assets: &'a HashMap<ImportableName, ImportableAsset>,
        schema_set: &'a SchemaSet,
        project_config: &'a HydrateProjectConfiguration,
        imported_importables: &'a mut HashMap<ImportableName, ImportedImportable>,
    ) -> ImportContext<'a> {
        ImportContext {
            path,
            importable_assets,
            schema_set,
            project_config,
            imported_importables: Rc::new(RefCell::new(imported_importables)),
        }
    }

    pub fn add_importable(
        &self,
        name: ImportableName,
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

    pub fn add_default_importable(
        &self,
        asset: SingleObject,
        import_data: Option<SingleObject>,
    ) {
        self.add_importable(ImportableName::default(), asset, import_data);
    }

    pub fn should_import(
        &self,
        name: &ImportableName,
    ) -> bool {
        self.importable_assets.contains_key(name)
    }

    // This is for assets by this import job
    pub fn asset_id_for_importable(
        &self,
        name: &ImportableName,
    ) -> Option<AssetId> {
        self.importable_assets.get(name).map(|x| x.id)
    }

    pub fn canonical_path_for_referenced_file_path(
        &self,
        name: ImportableName,
        path: &PathReference,
    ) -> PipelineResult<&CanonicalPathReference> {
        let mut hasher = siphasher::sip128::SipHasher::default();
        path.to_string().hash(&mut hasher);
        let path_reference_hash = Uuid::from_u128(hasher.finish128().as_u128());

        Ok(self.importable_assets
            .get(&name)
            .ok_or_else(|| format!("Importable not found when trying to resolve path {:?} referenced by importable {:?}", path, name))?
            .path_references.get(&path_reference_hash)
            .ok_or_else(|| format!("No asset ID found for importable when trying to resolve path {:?} referenced by importable {:?}", path, name))?)

        // if self.importable_assets
        //     .get(&name)
        //     .ok_or_else(|| format!("Default importable not found when trying to resolve path {:?} referenced by importable {:?}", path, name))?
        //     .canonical_path_references.get(&path.clone().simplify(self.project_config)).is_none() {
        //     Err(format!("No asset ID found for default importable when trying to resolve path {:?} referenced by importable {:?}", path, name))?
        // }

        //Ok(path.clone().simplify(self.project_config))
    }

    // This is for assets produced by importing other files
    pub fn asset_id_for_referenced_file_path(
        &self,
        name: ImportableName,
        path: &PathReference,
    ) -> PipelineResult<AssetId> {
        Ok(*self.importable_assets
            .get(&name)
            .ok_or_else(|| format!("Importable not found when trying to resolve path {:?} referenced by importable {:?}", path, name))?
            .canonical_path_references.get(&path.clone().simplify(self.project_config))
            .ok_or_else(|| format!("No asset ID found for importable when trying to resolve path {:?} referenced by importable {:?}", path, name))?)
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

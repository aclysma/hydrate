use std::alloc::System;
use std::ffi::OsStr;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::Uuid;
use hydrate_data::{BuildInfo, ImporterId, ImportInfo, ObjectId, ObjectLocation, ObjectName, ObjectSourceId};
use hydrate_schema::{HashMap, SchemaNamedType};
use crate::{AssetEngine, EditContextObjectImportInfoJson, HashSet, import_util, Importer, ImporterRegistry, ImporterRegistryBuilder, MetaFile, MetaFileJson, PathNode, PathNodeRoot, ScannedImportable};
use crate::DataSource;
use crate::edit_context::EditContext;
use crate::import_util::ImportToQueue;

// New trait design
// - fn revert_all(...)
//   - Determine disk state
//   - Determine memory state
//   - Delete/Load anything that doesn't match
// - fn flush_to_storage(...)
//   - Determine disk state??
//   - Determine memory state
//   - Save anything that doesn't match
// - fn asset_file_state(...) -> Saved, Modified, RuntimeGenerated
// - fn asset_is_generated(...)?
// - fn asset_needs_save(...)?
// - fn asset_scm_state(...) -> Locked, CheckedOut, Writable,
// - fn has disk changed and we need to reload?
// -
//
// - Should there be tree-based helpers on asset DB? Mainly to accelerate determining what data
//   source something is in, drawing UI tree, providing a consistent apparent state even when data
//   is in bad state. Map IDs to paths? Fix duplicates?
//
// IDEA: The database should store paths as strings and ID/Path based systems have to deal with
// conversion to ID if needed? Means renames touch lots of assets in memory.

struct FileMetadata {
    size_in_bytes: u64,
    last_modified_time: Option<SystemTime>,
}

impl FileMetadata {
    pub fn new(metadata: &std::fs::Metadata) -> Self {
        FileMetadata {
            size_in_bytes: metadata.len(),
            last_modified_time: metadata.modified().ok()
        }
    }

    pub fn has_changed(&self, metadata: &std::fs::Metadata) -> bool {
        self.size_in_bytes != metadata.len() || self.last_modified_time != metadata.modified().ok()
    }
}

#[derive(Clone)]
struct ObjectOnDiskState {
    containing_path: PathBuf,
    asset_file_path: PathBuf,
    name: String,
    is_directory: bool,
}

// Key: PathBuf
struct SourceFileDiskState {
    // may be generated or persisted
    generated_assets: HashSet<ObjectId>,
    persisted_assets: HashSet<ObjectId>,
    source_file_metadata: FileMetadata,
    // modified time? file length?


    importer_id: ImporterId,
    importables: HashMap<Option<String>, ObjectId>,
}

// Key: ObjectId
struct GeneratedAssetDiskState {
    source_file_path: PathBuf
    // Immutable, don't need to keep state for the asset, just the source file path
}

// Key: ObjectId
struct PersistedAssetDiskState {
    asset_file_path: PathBuf,
    asset_file_metadata: FileMetadata,
    // modified time? file length?
    // hash of asset's on-disk state?
}

enum AssetDiskState {
    Generated(GeneratedAssetDiskState),
    Persisted(PersistedAssetDiskState),
}

impl AssetDiskState {
    fn is_persisted(&self) -> bool {
        match self {
            AssetDiskState::Generated(_) => false,
            AssetDiskState::Persisted(_) => true,
        }
    }

    fn is_generated(&self) -> bool {
        !self.is_persisted()
    }

    fn as_generated_asset_disk_state(&self) -> Option<&GeneratedAssetDiskState> {
        match self {
            AssetDiskState::Generated(x) => Some(x),
            AssetDiskState::Persisted(_) => None,
        }
    }

    fn as_persisted_asset_disk_state(&self) -> Option<&PersistedAssetDiskState> {
        match self {
            AssetDiskState::Generated(_) => None,
            AssetDiskState::Persisted(x) => Some(x),
        }
    }
}

pub struct FileSystemPathBasedDataSource {
    object_source_id: ObjectSourceId,
    file_system_root_path: PathBuf,

    importer_registry: ImporterRegistry,

    // Any object ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_object_ids_on_disk_with_on_disk_state: HashMap<ObjectId, ObjectOnDiskState>,
    //all_assigned_path_ids: HashMap<PathBuf, ObjectId>,




    source_files_disk_state: HashMap<PathBuf, SourceFileDiskState>,
    assets_disk_state: HashMap<ObjectId, AssetDiskState>,


    path_node_schema: SchemaNamedType,
    path_node_root_schema: SchemaNamedType,
}

impl FileSystemPathBasedDataSource {
    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
        object_source_id: ObjectSourceId,
        importer_registry: &ImporterRegistry,
    ) -> Self {
        let path_node_schema = edit_context.schema_set().find_named_type(PathNode::schema_name()).unwrap().clone();
        let path_node_root_schema = edit_context.schema_set().find_named_type(PathNodeRoot::schema_name()).unwrap().clone();

        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system object data source {:?}",
            file_system_root_path,
        );

        FileSystemPathBasedDataSource {
            object_source_id,
            file_system_root_path: file_system_root_path.into(),
            importer_registry: importer_registry.clone(),
            all_object_ids_on_disk_with_on_disk_state: Default::default(),

            source_files_disk_state: Default::default(),
            assets_disk_state: Default::default(),

            path_node_schema,
            path_node_root_schema,
        }
    }

    fn is_object_owned_by_this_data_source(&self, edit_context: &EditContext, object_id: ObjectId) -> bool {
        if edit_context.object_schema(object_id).unwrap().fingerprint() == self.path_node_root_schema.fingerprint() {
            return false;
        }

        let root_location = edit_context.object_location_chain(object_id).last().cloned().unwrap_or_else(ObjectLocation::null);
        self.is_root_location_owned_by_this_data_source(&root_location)
    }

    fn is_root_location_owned_by_this_data_source(&self, root_location: &ObjectLocation) -> bool {
        root_location.path_node_id().as_uuid() == *self.object_source_id.uuid()
    }

    fn find_all_modified_objects(&self, edit_context: &EditContext) -> HashSet<ObjectId> {
        // We need to handle objects that had their paths changed. For ID-based data sources we can
        // simply rely on objects being marked modified if their own parent changed, but for a file
        // system a move can cause many files need to be removed/added to the file system.
        let mut modified_objects = edit_context.modified_objects().clone();
        for object_id in edit_context.objects().keys() {
            if self.is_object_owned_by_this_data_source(edit_context, *object_id) {
                let mut containing_file_path = self.containing_file_path_for_object(edit_context, *object_id);
                if let Some(on_disk_state) = self.all_object_ids_on_disk_with_on_disk_state.get(object_id) {
                    if containing_file_path != on_disk_state.containing_path {
                        // It has been moved, we will need to write it to the new location
                        // We will delete the old location later (we have to be careful with how we handle directories)
                        modified_objects.insert(*object_id);
                    }
                } else {
                    // It's not on disk, we will need to write it
                    modified_objects.insert(*object_id);
                }
            }
        }

        modified_objects
    }

    fn containing_file_path_for_object(&self, edit_context: &EditContext, object_id: ObjectId) -> PathBuf {
        let mut location_chain = edit_context.object_location_chain(object_id);

        let mut parent_dir = self.file_system_root_path.clone();

        // Pop the PathNodeRoot off the chain so we don't include it in the file path
        let path_node_root_id = location_chain.pop();

        // If the PathNodeRoot doesn't match this data source's object source ID, we're in an unexpected state.
        // Default to having the object show as being in the root of the datasource
        if path_node_root_id != Some(ObjectLocation::new(ObjectId::from_uuid(*self.object_source_id.uuid()))) {
            return parent_dir;
        }

        for location in location_chain.iter().rev() {
            let name = edit_context.object_name(location.path_node_id());
            parent_dir.push(name.as_string().unwrap());
        }

        parent_dir
    }

    // fn file_name_for_object(&self, edit_context: &EditContext, object_id: ObjectId) -> PathBuf {
    //     let object_name = edit_context.object_name(object_id).as_string().cloned().unwrap_or_else(|| object_id.as_uuid().to_string());
    //     let is_directory = edit_context.object_schema(object_id).unwrap().fingerprint() == self.path_node_schema.fingerprint();
    //
    //     assert!(!object_name.is_empty());
    //     if is_directory {
    //         PathBuf::from(object_name)
    //     } else {
    //         PathBuf::from(format!("{}.af", object_name))
    //     }
    // }

    // Pass object names through sanitize_object_name to ensure we don't have an empty string
    fn file_name_for_object(object_name: &str, is_directory: bool) -> PathBuf {
        //let object_name = edit_context.object_name(object_id).as_string().cloned().unwrap_or_else(|| object_id.as_uuid().to_string());
        //let is_directory = edit_context.object_schema(object_id).unwrap().fingerprint() == self.path_node_schema.fingerprint();

        if is_directory {
            PathBuf::from(object_name)
        } else {
            PathBuf::from(format!("{}.af", object_name))
        }
    }

    fn sanitize_object_name(object_id: ObjectId, object_name: &ObjectName) -> String {
        object_name.as_string().cloned().unwrap_or_else(|| object_id.as_uuid().to_string())
    }

    fn canonicalize_all_path_nodes(&self, edit_context: &mut EditContext) -> HashMap<PathBuf, ObjectId> {
        let mut all_paths: HashMap<PathBuf, ObjectId> = Default::default();

        // Go through all the objects and come up with a 1:1 mapping of path node ID to path
        // - Duplicate path nodes: delete all but one, update all references
        // - Cyclical references: delete the path nodes and place all objects contained in them at the root
        // - Empty names: use the object ID
        for (k, v) in edit_context.objects() {
            let mut location_chain = edit_context.object_location_chain(*k);
            let root_location = location_chain.last().cloned().unwrap_or_else(ObjectLocation::null);
            if !self.is_root_location_owned_by_this_data_source(&root_location) {
                // Skip anything not owned by this data source
                continue;
            }

            // The root location is not needed after this point, pop it off
            location_chain.pop();

            let is_path_node = v.schema().fingerprint() == self.path_node_schema.fingerprint();
            if !is_path_node {
                // Skip anything that is not a path node
                continue;
            }

            let mut root_dir = self.file_system_root_path.clone();
            for element in location_chain {
                let node_name = edit_context.object_name(element.path_node_id());
                let sanitized_name = Self::sanitize_object_name(element.path_node_id(), node_name);
                root_dir.push(sanitized_name);

                if all_paths.contains_key(&root_dir) {
                    // dupe found
                    // we can delete the dupe and find any objects parented to it and redirect them here later
                } else {
                    all_paths.insert(root_dir.clone(), element.path_node_id());
                }
            }
        }

        all_paths.insert(self.file_system_root_path.clone(), ObjectId::from_uuid(*self.object_source_id.uuid()));

        all_paths
    }

    fn ensure_object_location_exists(
        &self,
        ancestor_path: &Path,
        path_to_path_node_id: &mut HashMap<PathBuf, ObjectId>,
        edit_context: &mut EditContext
    ) -> ObjectLocation {
        //
        // Iterate backwards from the file on disk to the root of this data source.
        // Build the paths that need to exist. We will iterate this list in reverse
        // to ensure the entire chain of path nodes exist, creating any that are missing.
        //
        let mut ancestor_paths = Vec::default();
        let mut ancestor_path_iter = Some(ancestor_path);
        let mut found_root = false;
        while let Some(path) = ancestor_path_iter {
            if path == self.file_system_root_path {
                found_root = true;
                break;
            }

            ancestor_paths.push(path.to_path_buf());
            //ancestor_path = path.to_path_buf();
            ancestor_path_iter = path.parent();
        }

        // Make sure that when we crawled up the file tree, we terminated at the root of this data source
        assert!(found_root);

        // If we create a missing path node, we will have to parent it to the previous path node. So
        // keep track of the previous object's ID
        let mut previous_object_id = ObjectId::from_uuid(*self.object_source_id.uuid());

        // Now traverse the list of ancestors in REVERSE (root -> file)
        for ancestor_path in ancestor_paths.iter().rev() {
            if let Some(existing_path_node_id) = path_to_path_node_id.get(ancestor_path) {
                // The path node already exists, continue
                previous_object_id = *existing_path_node_id;
            } else {
                // The path node doesn't exist, we need to create it
                let file_name = ancestor_path.file_name().unwrap().to_string_lossy();
                let new_path_node_id = edit_context.new_object(
                    &ObjectName::new(file_name),
                    &ObjectLocation::new(previous_object_id),
                    self.path_node_schema.as_record().unwrap()
                );
                edit_context.clear_object_modified_flag(new_path_node_id);

                // add this path node to our canonical list of paths/IDs
                path_to_path_node_id.insert(ancestor_path.to_path_buf(), new_path_node_id);
                previous_object_id = new_path_node_id;
            }
        }

        ObjectLocation::new(previous_object_id)
    }
}

impl DataSource for FileSystemPathBasedDataSource {
    fn is_generated_asset(&self, object_id: ObjectId) -> bool {
        if let Some(asset_disk_state) = self.assets_disk_state.get(&object_id) {
            asset_disk_state.is_generated()
        } else {
            false
        }
    }

    // fn object_symbol_name(&self, edit_context: &EditContext, object_id: ObjectId) -> Option<String> {
    //     //let location_path = edit_context.ro
    //     None
    // }

    fn persist_generated_asset(&mut self, edit_context: &mut EditContext, object_id: ObjectId) {
        //let asset_disk_state = self.assets_disk_state.get_mut(object_id).unwrap();
        let old_asset_disk_state = self.assets_disk_state.remove(&object_id);
        let source_file_path = old_asset_disk_state.unwrap().as_generated_asset_disk_state().unwrap().source_file_path.clone();

        let mut containing_file_path = self.containing_file_path_for_object(edit_context, object_id);
        let is_directory = false;
        let object_name = Self::sanitize_object_name(object_id, edit_context.object_name(object_id));
        let file_name = Self::file_name_for_object(&object_name, is_directory);
        let asset_file_path = containing_file_path.join(file_name);
        // It's a object, create an asset file
        let data = crate::json_storage::EditContextObjectJson::save_edit_context_object_to_string(
            edit_context,
            object_id,
            true,
            None
        );

        std::fs::create_dir_all(&containing_file_path).unwrap();
        std::fs::write(&asset_file_path, data).unwrap();

        let asset_file_metadata = FileMetadata::new(&std::fs::metadata(&asset_file_path).unwrap());
        self.assets_disk_state.insert(object_id, AssetDiskState::Persisted(PersistedAssetDiskState {
            asset_file_metadata,
            asset_file_path: asset_file_path.clone()
        }));

        self.all_object_ids_on_disk_with_on_disk_state.insert(object_id, ObjectOnDiskState {
            containing_path: containing_file_path.clone(),
            asset_file_path: asset_file_path.clone(),
            is_directory,
            name: object_name.clone()
        });

        let source_file_disk_state = self.source_files_disk_state.get_mut(&source_file_path).unwrap();
        source_file_disk_state.generated_assets.remove(&object_id);
        source_file_disk_state.persisted_assets.insert(object_id);

        edit_context.clear_object_modified_flag(object_id);
    }

    fn load_from_storage(&mut self, edit_context: &mut EditContext, imports_to_queue: &mut Vec<ImportToQueue>) {
        //
        // Delete all objects from the database owned by this data source
        //
        let mut objects_to_delete = Vec::default();
        for (object_id, _) in edit_context.objects() {
            if self.is_object_owned_by_this_data_source(edit_context, *object_id) {
                objects_to_delete.push(*object_id);
            }
        }

        for object_to_delete in objects_to_delete {
            edit_context.delete_object(object_to_delete);
        }


        // for (object_id, asset_disk_state) in &self.assets_disk_state {
        //     edit_context.delete_object(*object_id);
        // }

        let mut path_to_path_node_id = self.canonicalize_all_path_nodes(edit_context);

        //
        // First visit all folders to create path nodes
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(&self.file_system_root_path, &["**"])
            .file_type(globwalk::FileType::DIR)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                self.ensure_object_location_exists(file.path(), &mut path_to_path_node_id, edit_context);
            }
        }

        //
        // Visit all files and categorize them as meta files, asset files, or source files
        // - Asset files end in .af
        // - Meta files end in .meta
        // - Anything else is presumed to be a source file
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(&self.file_system_root_path, &["**"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        let mut source_files = Vec::default();
        let mut asset_files = Vec::default();
        let mut meta_files = Vec::default();

        for file in walker {
            if let Ok(file) = file {
                if file.path().extension() == Some(OsStr::new("meta")) {
                    meta_files.push(file.path().to_path_buf());
                } else if file.path().extension() == Some(OsStr::new("af")) {
                    asset_files.push(file.path().to_path_buf());
                } else {
                    source_files.push(file.path().to_path_buf());
                }
            }
        }

        //
        // Scan all meta files, any asset file that exists and is referenced by a meta file will
        // be re-imported. (Because the original source asset is presumed to exist alongside the
        // meta file and source files in a path-based data source get re-imported automatically)
        //
        struct SourceFileImportableName {
            source_file: PathBuf,
            importable_name: String,
        }
        let mut objects_with_source_files = HashMap::<ObjectId, SourceFileImportableName>::default();
        let mut source_file_meta_files = HashMap::<PathBuf, MetaFile>::default();
        for meta_file in meta_files {
            let source_file = meta_file.with_extension("");
            if !source_file.exists() {
                println!("Could not find source file, can't re-import data. Restore the source file or delete the meta file.");
                continue;
            }
            //println!("meta file {:?} source file {:?}", meta_file, source_file);

            let contents = std::fs::read_to_string(meta_file.as_path()).unwrap();
            let meta_file_contents = crate::json_storage::MetaFileJson::load_from_string(&contents);
            for (importable_name, id_assignment) in &meta_file_contents.past_id_assignments {
                let sfin = SourceFileImportableName {
                    source_file: source_file.clone(),
                    importable_name: importable_name.clone()
                };
                let old = objects_with_source_files.insert(*id_assignment, sfin);
                // If this trips we have two meta files claiming to export the same ID
                assert!(old.is_none());
            }

            source_file_meta_files.insert(source_file, meta_file_contents);
        }

        let mut source_files_disk_state = HashMap::<PathBuf, SourceFileDiskState>::default();
        let mut assets_disk_state = HashMap::<ObjectId, AssetDiskState>::default();

        //
        // Load any asset files.
        //
        for asset_file in asset_files {
            //println!("asset file {:?}", asset_file);
            let contents = std::fs::read_to_string(asset_file.as_path()).unwrap();

            let object_location = self.ensure_object_location_exists(
                asset_file.as_path().parent().unwrap(),
                &mut path_to_path_node_id,
                edit_context
            );
            let object_id = crate::json_storage::EditContextObjectJson::load_edit_context_object_from_string(
                edit_context,
                None,
                self.object_source_id,
                Some(object_location.clone()),
                &contents
            );

            // if objects_with_source_files.contains_key(&object_id) {
            //     // We will force a re-import of this asset, meaning existing import data will be
            //     // overwritten, but we will retain this asset instead of creating a new one
            //     //TODO: Any validation?
            // }

            //TODO: Track some revision number instead of modified flags?
            edit_context.clear_object_modified_flag(object_id);
            edit_context.clear_location_modified_flag(&object_location);

            let asset_file_metadata = FileMetadata::new(&std::fs::metadata(&asset_file).unwrap());

            assets_disk_state.insert(object_id, AssetDiskState::Persisted(PersistedAssetDiskState {
                asset_file_path: asset_file,
                asset_file_metadata,
            }));
        }

        //
        // Scan all the source files and ensure IDs exist for all importables and build a lookup for
        // finding source files by path. Currently we only allow referencing the unnamed/"default"
        // importable by path? Maybe we only support implicit import when a file has a single importable?
        // Don't think it's impossible to support this but the point of supporting paths is to allow
        // working with files/workflows we can't control, and these things generally just use a plain path.
        // For now will go ahead and try to support it.
        //


        //
        // Scan all the source files and ensure stable IDs exist for all importables. We do this as
        // a first pass, and a second pass will actually create the assets and ensure references in
        // the file are satisfied and pointing to the correct asset
        //
        struct ScannedSourceFile<'a> {
            meta_file: MetaFile,
            importer: &'a Box<dyn Importer>,
            scanned_importables: Vec<ScannedImportable>,
        }
        let mut scanned_source_files = HashMap::<PathBuf, ScannedSourceFile>::default();

        let empty_string = "".to_string();

        for source_file in source_files {
            //println!("source file first pass {:?}", source_file);
            // Does a meta file exist?
            // - If it does: re-import it, but only create new assets if there is not already an asset file
            // - If it does not: re-import it and create all new asset files

            let extension = &source_file.extension();
            if extension.is_none() {
                // Can happen for files like .DS_Store
                continue;
            }

            let importers = self.importer_registry.importers_for_file_extension(
                &extension.unwrap().to_string_lossy()
            );

            if importers.is_empty() {
                // No importer found
            } else if importers.len() > 1 {
                // Multiple importers found, no way of disambiguating
            } else {
                let importer = self.importer_registry.importer(importers[0]).unwrap();

                let scanned_importables = importer.scan_file(
                    &source_file, edit_context.schema_set(), &self.importer_registry);

                //println!("  find meta file {:?}", source_file);
                let mut meta_file = source_file_meta_files.get(&source_file).cloned().unwrap_or_default();
                for scanned_importable in &scanned_importables {
                    // Does it exist in the meta file? If so, we need to reuse the ID
                    meta_file.past_id_assignments
                        .entry(scanned_importable.name.as_ref().cloned().unwrap_or_default())
                        .or_insert_with(|| ObjectId::from_uuid(Uuid::new_v4()));
                }

                let mut meta_file_path = source_file.clone().into_os_string();
                meta_file_path.push(".meta");

                let source_file_metadata = FileMetadata::new(&std::fs::metadata(&source_file).unwrap());

                let mut importables = HashMap::<Option<String>, ObjectId>::default();
                for scanned_importable in &scanned_importables {
                    let empty_string = String::default();
                    let imporable_object_id = meta_file.past_id_assignments.get(scanned_importable.name.as_ref().unwrap_or(&empty_string).as_str());
                    importables.insert(scanned_importable.name.clone(), *imporable_object_id.unwrap());
                }

                source_files_disk_state.insert(source_file.clone(), SourceFileDiskState {
                    generated_assets: Default::default(),
                    persisted_assets: Default::default(),
                    source_file_metadata,
                    importer_id: importer.importer_id(),
                    importables,
                });

                std::fs::write(meta_file_path, MetaFileJson::store_to_string(&meta_file)).unwrap();
                scanned_source_files.insert(source_file, ScannedSourceFile {
                    meta_file,
                    importer,
                    scanned_importables
                });
            }
        }

        //
        // Re-import source files
        //
        for (source_file_path, scanned_source_file) in &scanned_source_files {
            println!("source file second pass {:?}", source_file_path);
            let parent_dir = source_file_path.parent().unwrap();
            //println!("  import to dir {:?}", parent_dir);
            let import_location = ObjectLocation::new(*path_to_path_node_id.get(parent_dir).unwrap());

            let mut source_file_disk_state = source_files_disk_state.get_mut(source_file_path).unwrap();

            let mut requested_importables = HashMap::default();
            let mut assets_to_regenerate = HashSet::default();
            for scanned_importable in &scanned_source_file.scanned_importables {
                // The ID assigned to this importable. We have this now because we previously scanned
                // all source files and assigned IDs to any importable
                let importable_object_id = *scanned_source_files
                    .get(source_file_path)
                    .unwrap()
                    .meta_file.past_id_assignments
                    .get(scanned_importable.name.as_ref().unwrap_or(&empty_string))
                    .unwrap();

                // Create an object name for this asset
                let object_name = import_util::create_object_name(
                    source_file_path,
                    scanned_importable
                );

                let asset_file_exists = edit_context.has_object(importable_object_id);

                if !asset_file_exists {
                    edit_context.new_object_with_id(
                        importable_object_id,
                        &object_name,
                        &import_location,
                        &scanned_importable.asset_type,
                    ).unwrap();

                    // Create the import info for this asset
                    let import_info = import_util::create_import_info(
                        source_file_path,
                        scanned_source_file.importer,
                        scanned_importable
                    );
                    edit_context.set_import_info(importable_object_id, import_info);

                    assets_disk_state.insert(importable_object_id, AssetDiskState::Generated(GeneratedAssetDiskState {
                        source_file_path: source_file_path.clone()
                    }));
                    source_file_disk_state.generated_assets.insert(importable_object_id);
                } else {
                    assert_eq!(edit_context.object_schema(importable_object_id).unwrap().fingerprint(), scanned_importable.asset_type.fingerprint());
                    //edit_context.set_object_name(importable_object_id, object_name);
                    //edit_context.set_object_location(importable_object_id, *import_location);
                    //edit_context.set_import_info(importable_object_id, import_info);

                    // We iterated through asset files already, so just check that we inserted a AssetDiskState::Persisted into this map
                    assert!(assets_disk_state.get(&importable_object_id).unwrap().is_persisted());
                    source_file_disk_state.persisted_assets.insert(importable_object_id);
                }

                // For any referenced file, locate the ObjectID at that path. It must be in this data source,
                // and for now we only support referencing the default importable out of source files (so
                // can't reference asset files by path, for now). So it must exist.
                let mut referenced_source_file_object_ids = Vec::default();
                for file_reference in &scanned_importable.file_references {
                    let file_reference_absolute_path = if file_reference.path.is_relative() {
                        source_file_path.parent()
                            .unwrap()
                            .join(&file_reference.path)
                            .canonicalize()
                            .unwrap()
                    } else {
                        file_reference.path.clone()
                    };

                    //println!("referenced {:?} {:?}", file_reference_absolute_path, scanned_source_files.keys());
                    let referenced_object = scanned_source_files.get(&file_reference_absolute_path).unwrap();
                    assert_eq!(file_reference.importer_id, referenced_object.importer.importer_id());
                    referenced_source_file_object_ids.push(referenced_object.meta_file.past_id_assignments.get(""));
                }

                assert_eq!(
                    referenced_source_file_object_ids.len(),
                    scanned_importable.file_references.len()
                );

                for (k, v) in scanned_importable
                    .file_references
                    .iter()
                    .zip(referenced_source_file_object_ids)
                {
                    if let Some(v) = v {
                        edit_context
                            .set_file_reference_override(importable_object_id, k.path.clone(), *v);
                    }
                }

                requested_importables.insert(scanned_importable.name.clone(), importable_object_id);
                if !asset_file_exists {
                    assets_to_regenerate.insert(importable_object_id);
                }
            }

            imports_to_queue.push(ImportToQueue {
                source_file_path: source_file_path.to_path_buf(),
                importer_id: scanned_source_file.importer.importer_id(),
                requested_importables,
                assets_to_regenerate
            });
        }

        self.assets_disk_state = assets_disk_state;
        self.source_files_disk_state = source_files_disk_state;

        // //
        // // Import the file
        // // - Reuse existing assets if they are referenced by the meta file
        // // - Create new assets if they do not exist
        // //

        //
        // Validate that the rules for supporting loose source files in path-based data sources are being upheld
        //
        //
        //  - When source files are located in a path-based data source:
        //    - They always get re-scanned and re-imported every time the data source is opened
        //    - They cannot reference any files via path that are not also in that data source
        //    - Their assets cannot be renamed or moved. (Users must rename/move the source file)
        //    - Other assets cannot be stored in a location associated with the source file.
        //    - When importables are removed from a source file, the asset is not loaded and
        //      it may break asset references?
    }

    fn flush_to_storage(&mut self, edit_context: &mut EditContext) {

        // Delete files for objects that were deleted
        // for object_id in edit_context.modified_objects() {
        //     if self.all_object_ids_on_disk_with_original_path.contains_key(object_id)
        //         && !edit_context.has_object(*object_id)
        //     {
        //         //TODO: delete the object file
        //         self.all_object_ids_on_disk_with_original_path.remove(object_id);
        //     }
        // }


        let mut updated_all_object_ids_on_disk_with_on_disk_state = self.all_object_ids_on_disk_with_on_disk_state.clone();

        let modified_objects = self.find_all_modified_objects(edit_context);

        // We will write out any files that were modified or moved
        for object_id in &modified_objects {
            if let Some(object_info) = edit_context.objects().get(object_id) {
                if self.is_object_owned_by_this_data_source(edit_context, *object_id) {
                    if object_id.as_uuid() == *self.object_source_id.uuid() {
                        // never save the root object
                        continue;
                    }

                    if let Some(asset_disk_state) = self.assets_disk_state.get(object_id) {
                        if asset_disk_state.is_generated() {
                            // Never store generated assets, they exist because their source file is
                            // on disk and they aren't mutable in the editor
                            continue;
                        }
                    }

                    let mut containing_file_path = self.containing_file_path_for_object(edit_context, *object_id);
                    let is_directory = edit_context.object_schema(*object_id).unwrap().fingerprint() == self.path_node_schema.fingerprint();
                    let object_name = Self::sanitize_object_name(*object_id, edit_context.object_name(*object_id));
                    let file_name = Self::file_name_for_object(&object_name, is_directory);
                    let asset_file_path = containing_file_path.join(file_name);

                    if is_directory {
                        // It's a path node, ensure the dir exists
                        std::fs::create_dir_all(&asset_file_path).unwrap();
                    } else {
                        // It's a object, create an asset file
                        let data = crate::json_storage::EditContextObjectJson::save_edit_context_object_to_string(
                            edit_context,
                            *object_id,
                            true,
                            None
                        );

                        std::fs::create_dir_all(&containing_file_path).unwrap();
                        std::fs::write(&asset_file_path, data).unwrap();

                        let asset_file_metadata = FileMetadata::new(&std::fs::metadata(&asset_file_path).unwrap());
                        self.assets_disk_state.insert(*object_id, AssetDiskState::Persisted(PersistedAssetDiskState {
                            asset_file_metadata,
                            asset_file_path: asset_file_path.clone()
                        }));

                        // We know the asset was already persisted so we don't need to update source files state
                    }

                    updated_all_object_ids_on_disk_with_on_disk_state.insert(*object_id, ObjectOnDiskState {
                        containing_path: containing_file_path.clone(),
                        asset_file_path: asset_file_path.clone(),
                        is_directory,
                        name: object_name.clone()
                    });
                }
            }
        }

        // Delete anything on disk that shouldn't still be on disk
        // Maybe check before we start saving anything if the disk state has changed and offer to reload?

        //TODO: Implement probably in passes? Files first then directories? Or maybe we can just
        // reverse sort by filename length?


        // Update all_object_ids_on_disk_with_original_path
        self.all_object_ids_on_disk_with_on_disk_state = updated_all_object_ids_on_disk_with_on_disk_state;
    }
}
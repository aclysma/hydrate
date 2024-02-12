use crate::edit_context::EditContext;
use crate::{AssetSourceId, DataSource, PendingFileOperations};
use crate::{PathNode, PathNodeRoot};
use hydrate_base::hashing::HashSet;
use hydrate_data::json_storage::{MetaFile, MetaFileJson};
use hydrate_data::{
    AssetId, AssetLocation, AssetName, CanonicalPathReference, DataSetAssetInfo, HashObjectMode,
    ImportableName, ImporterId, PathReference,
};
use hydrate_pipeline::{
    HydrateProjectConfiguration, ImportJobSourceFile, ImportJobToQueue, ImportLogEvent, ImportType,
    Importer, ImporterRegistry, LogEventLevel, PipelineResult, RequestedImportable, ScanContext,
    ScannedImportable,
};
use hydrate_schema::{HashMap, SchemaNamedType};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

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

// Temporary struct used during load_from_storage call
struct ScannedSourceFile<'a> {
    meta_file: MetaFile,
    importer: &'a Arc<dyn Importer>,
    scanned_importables: Vec<ScannedImportable>,
}

struct FileMetadata {
    // size_in_bytes: u64,
    // last_modified_time: Option<SystemTime>,
}

impl FileMetadata {
    pub fn new(_metadata: &std::fs::Metadata) -> Self {
        FileMetadata {
            // size_in_bytes: metadata.len(),
            // last_modified_time: metadata.modified().ok()
        }
    }

    // pub fn has_changed(&self, metadata: &std::fs::Metadata) -> bool {
    //     self.size_in_bytes != metadata.len() || self.last_modified_time != metadata.modified().ok()
    // }
}

// Key: PathBuf
struct SourceFileDiskState {
    // may be generated or persisted
    generated_assets: HashSet<AssetId>,
    persisted_assets: HashSet<AssetId>,
    //source_file_metadata: FileMetadata,
    _importer_id: ImporterId,
    _importables: HashMap<ImportableName, AssetId>,
}

// Key: AssetId
struct GeneratedAssetDiskState {
    source_file_path: PathBuf, // Immutable, don't need to keep state for the asset, just the source file path
}

// Key: AssetId
struct PersistedAssetDiskState {
    asset_file_path: PathBuf,
    _asset_file_metadata: FileMetadata,
    object_hash: u64,
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
}

pub struct FileSystemPathBasedDataSource {
    asset_source_id: AssetSourceId,
    file_system_root_path: PathBuf,

    importer_registry: ImporterRegistry,

    //all_assigned_path_ids: HashMap<PathBuf, AssetId>,
    source_files_disk_state: HashMap<PathBuf, SourceFileDiskState>,
    assets_disk_state: HashMap<AssetId, AssetDiskState>,

    path_node_schema: SchemaNamedType,
    path_node_root_schema: SchemaNamedType,
}

impl FileSystemPathBasedDataSource {
    pub fn asset_source_id(&self) -> AssetSourceId {
        self.asset_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
        asset_source_id: AssetSourceId,
        importer_registry: &ImporterRegistry,
    ) -> Self {
        let path_node_schema = edit_context
            .schema_set()
            .find_named_type(PathNode::schema_name())
            .unwrap()
            .clone();
        let path_node_root_schema = edit_context
            .schema_set()
            .find_named_type(PathNodeRoot::schema_name())
            .unwrap()
            .clone();

        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system asset data source {:?}",
            file_system_root_path,
        );

        FileSystemPathBasedDataSource {
            asset_source_id,
            file_system_root_path: file_system_root_path.into(),
            importer_registry: importer_registry.clone(),

            source_files_disk_state: Default::default(),
            assets_disk_state: Default::default(),

            path_node_schema,
            path_node_root_schema,
        }
    }

    fn is_asset_owned_by_this_data_source(
        &self,
        edit_context: &EditContext,
        asset_id: AssetId,
    ) -> bool {
        if edit_context.asset_schema(asset_id).unwrap().fingerprint()
            == self.path_node_root_schema.fingerprint()
        {
            return false;
        }

        let root_location = edit_context
            .asset_location_chain(asset_id)
            .unwrap_or_default()
            .last()
            .cloned()
            .unwrap_or_else(AssetLocation::null);
        self.is_root_location_owned_by_this_data_source(&root_location)
    }

    fn is_root_location_owned_by_this_data_source(
        &self,
        root_location: &AssetLocation,
    ) -> bool {
        root_location.path_node_id().as_uuid() == *self.asset_source_id.uuid()
    }

    fn path_for_asset(
        &self,
        containing_file_path: &Path,
        asset_id: AssetId,
        asset_info: &DataSetAssetInfo,
    ) -> PathBuf {
        let is_directory = asset_info.schema().fingerprint() == self.path_node_schema.fingerprint();
        let asset_name = Self::sanitize_asset_name(asset_id, asset_info.asset_name());
        let file_name = Self::file_name_for_asset(&asset_name, is_directory);
        let asset_file_path = containing_file_path.join(file_name);
        asset_file_path
    }

    fn containing_file_path_for_asset(
        &self,
        edit_context: &EditContext,
        asset_id: AssetId,
    ) -> PathBuf {
        let mut location_chain = edit_context
            .asset_location_chain(asset_id)
            .unwrap_or_default();

        let mut parent_dir = self.file_system_root_path.clone();

        // Pop the PathNodeRoot off the chain so we don't include it in the file path
        let path_node_root_id = location_chain.pop();

        // If the PathNodeRoot doesn't match this data source's asset source ID, we're in an unexpected state.
        // Default to having the asset show as being in the root of the datasource
        if path_node_root_id
            != Some(AssetLocation::new(AssetId::from_uuid(
                *self.asset_source_id.uuid(),
            )))
        {
            return parent_dir;
        }

        for location in location_chain.iter().rev() {
            let name = edit_context.asset_name(location.path_node_id()).unwrap();
            parent_dir.push(name.as_string().unwrap());
        }

        parent_dir
    }

    // fn file_name_for_asset(&self, edit_context: &EditContext, asset_id: AssetId) -> PathBuf {
    //     let asset_name = edit_context.asset_name(asset_id).as_string().cloned().unwrap_or_else(|| asset_id.as_uuid().to_string());
    //     let is_directory = edit_context.asset_schema(asset_id).unwrap().fingerprint() == self.path_node_schema.fingerprint();
    //
    //     assert!(!asset_name.is_empty());
    //     if is_directory {
    //         PathBuf::from(asset_name)
    //     } else {
    //         PathBuf::from(format!("{}.af", asset_name))
    //     }
    // }

    // Pass asset names through sanitize_asset_name to ensure we don't have an empty string
    fn file_name_for_asset(
        asset_name: &str,
        is_directory: bool,
    ) -> PathBuf {
        //let asset_name = edit_context.asset_name(asset_id).as_string().cloned().unwrap_or_else(|| asset_id.as_uuid().to_string());
        //let is_directory = edit_context.asset_schema(asset_id).unwrap().fingerprint() == self.path_node_schema.fingerprint();

        if is_directory {
            PathBuf::from(asset_name)
        } else {
            PathBuf::from(format!("{}.af", asset_name))
        }
    }

    fn sanitize_asset_name(
        asset_id: AssetId,
        asset_name: &AssetName,
    ) -> String {
        asset_name
            .as_string()
            .cloned()
            .unwrap_or_else(|| asset_id.as_uuid().to_string())
    }

    fn canonicalize_all_path_nodes(
        &self,
        edit_context: &mut EditContext,
    ) -> HashMap<PathBuf, AssetId> {
        let mut all_paths: HashMap<PathBuf, AssetId> = Default::default();

        // Go through all the assets and come up with a 1:1 mapping of path node ID to path
        // - Duplicate path nodes: delete all but one, update all references
        // - Cyclical references: delete the path nodes and place all assets contained in them at the root
        // - Empty names: use the asset ID
        for (k, v) in edit_context.assets() {
            let mut location_chain = edit_context.asset_location_chain(*k).unwrap_or_default();
            let root_location = location_chain
                .last()
                .cloned()
                .unwrap_or_else(AssetLocation::null);
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
                let node_name = edit_context.asset_name(element.path_node_id()).unwrap();
                let sanitized_name = Self::sanitize_asset_name(element.path_node_id(), node_name);
                root_dir.push(sanitized_name);

                if all_paths.contains_key(&root_dir) {
                    // dupe found
                    // we can delete the dupe and find any assets parented to it and redirect them here later
                } else {
                    all_paths.insert(root_dir.clone(), element.path_node_id());
                }
            }
        }

        all_paths.insert(
            self.file_system_root_path.clone(),
            AssetId::from_uuid(*self.asset_source_id.uuid()),
        );

        all_paths
    }

    fn ensure_asset_location_exists(
        &self,
        ancestor_path: &Path,
        path_to_path_node_id: &mut HashMap<PathBuf, AssetId>,
        edit_context: &mut EditContext,
    ) -> AssetLocation {
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
        // keep track of the previous asset's ID
        let mut previous_asset_id = AssetId::from_uuid(*self.asset_source_id.uuid());

        // Now traverse the list of ancestors in REVERSE (root -> file)
        for ancestor_path in ancestor_paths.iter().rev() {
            if let Some(existing_path_node_id) = path_to_path_node_id.get(ancestor_path) {
                // The path node already exists, continue
                previous_asset_id = *existing_path_node_id;
            } else {
                // The path node doesn't exist, we need to create it
                let file_name = ancestor_path.file_name().unwrap().to_string_lossy();
                let new_path_node_id = edit_context.new_asset(
                    &AssetName::new(file_name),
                    &AssetLocation::new(previous_asset_id),
                    self.path_node_schema.as_record().unwrap(),
                );

                // add this path node to our canonical list of paths/IDs
                path_to_path_node_id.insert(ancestor_path.to_path_buf(), new_path_node_id);
                previous_asset_id = new_path_node_id;
            }
        }

        AssetLocation::new(previous_asset_id)
    }

    fn find_canonical_path_references(
        project_config: &HydrateProjectConfiguration,
        source_file_path: &PathBuf,
        scanned_importable: &ScannedImportable,
        scanned_source_files: &HashMap<PathBuf, ScannedSourceFile>,
    ) -> PipelineResult<HashMap<CanonicalPathReference, AssetId>> {
        // For any referenced file, locate the AssetID at that path. It must be in this data source,
        // and at this point must exist in the meta file.
        let mut canonical_path_references = HashMap::default();

        for (path_reference, &importer_id) in &scanned_importable.referenced_source_file_info {
            let path_reference_absolute = path_reference
                .canonicalized_absolute_path(project_config, source_file_path)
                .unwrap();

            //println!("referenced {:?} {:?}", path_reference_absolute_path, scanned_source_files.keys());
            //println!("pull from {:?}", scanned_source_files.keys());
            //println!("referenced {:?}", path_reference_absolute_path);
            let referenced_scanned_source_file = scanned_source_files
                .get(&PathBuf::from(path_reference_absolute.path()))
                .ok_or_else(|| format!("{:?} is referencing source file {:?} but it does not exist or failed to import", source_file_path, path_reference.path()))?;
            assert_eq!(
                importer_id,
                referenced_scanned_source_file.importer.importer_id()
            );
            canonical_path_references.insert(
                path_reference.clone(),
                *referenced_scanned_source_file
                    .meta_file
                    .past_id_assignments
                    .get(path_reference.importable_name())
                    .ok_or_else(|| format!(
                        "{:?} is referencing importable {:?} in {:?} but it was not found when the file was scanned",
                        source_file_path,
                        path_reference.path(),
                        path_reference.importable_name())
                    )
                    .unwrap()
            );
        }
        Ok(canonical_path_references)
    }
}

impl DataSource for FileSystemPathBasedDataSource {
    fn is_generated_asset(
        &self,
        asset_id: AssetId,
    ) -> bool {
        if let Some(asset_disk_state) = self.assets_disk_state.get(&asset_id) {
            asset_disk_state.is_generated()
        } else {
            false
        }
    }

    // fn asset_symbol_name(&self, edit_context: &EditContext, asset_id: AssetId) -> Option<String> {
    //     //let location_path = edit_context.ro
    //     None
    // }

    fn persist_generated_asset(
        &mut self,
        edit_context: &mut EditContext,
        asset_id: AssetId,
    ) {
        if !self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
            return;
        }

        let old_asset_disk_state = self.assets_disk_state.get(&asset_id).unwrap();
        if !old_asset_disk_state.is_generated() {
            return;
        }

        let old_asset_disk_state = self.assets_disk_state.remove(&asset_id);
        let source_file_path = old_asset_disk_state
            .unwrap()
            .as_generated_asset_disk_state()
            .unwrap()
            .source_file_path
            .clone();

        let mut meta_file_path = source_file_path.clone().into_os_string();
        meta_file_path.push(".meta");

        //
        // Write the asset
        //
        let containing_file_path = self.containing_file_path_for_asset(edit_context, asset_id);
        let asset_info = edit_context.assets().get(&asset_id).unwrap();
        let asset_file_path = self.path_for_asset(&containing_file_path, asset_id, asset_info);
        // It's a asset, create an asset file
        let data = crate::json_storage::AssetJson::save_asset_to_string(
            edit_context.schema_set(),
            edit_context.assets(),
            asset_id,
            true,
            None,
        );

        std::fs::create_dir_all(&containing_file_path).unwrap();
        std::fs::write(&asset_file_path, data).unwrap();

        //
        // Update the meta file
        //
        let contents = std::fs::read_to_string(&meta_file_path).unwrap();
        let mut meta_file_contents = MetaFileJson::load_from_string(&contents);
        meta_file_contents.persisted_assets.insert(asset_id);
        std::fs::write(
            &meta_file_path,
            MetaFileJson::store_to_string(&meta_file_contents),
        )
        .unwrap();

        //
        // Update representation of disk state
        //
        let object_hash = edit_context
            .data_set()
            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationChainNames)
            .unwrap();

        let asset_file_metadata = FileMetadata::new(&std::fs::metadata(&asset_file_path).unwrap());
        self.assets_disk_state.insert(
            asset_id,
            AssetDiskState::Persisted(PersistedAssetDiskState {
                _asset_file_metadata: asset_file_metadata,
                asset_file_path: asset_file_path.clone(),
                object_hash,
            }),
        );

        let source_file_disk_state = self
            .source_files_disk_state
            .get_mut(&source_file_path)
            .unwrap();
        source_file_disk_state.generated_assets.remove(&asset_id);
        source_file_disk_state.persisted_assets.insert(asset_id);
    }

    fn load_from_storage(
        &mut self,
        project_config: &HydrateProjectConfiguration,
        edit_context: &mut EditContext,
        import_job_to_queue: &mut ImportJobToQueue,
    ) {
        profiling::scope!(&format!(
            "load_from_storage {:?}",
            self.file_system_root_path
        ));
        //
        // Delete all assets from the database owned by this data source
        //
        let mut assets_to_delete = Vec::default();
        for (asset_id, _) in edit_context.assets() {
            if self.is_asset_owned_by_this_data_source(edit_context, *asset_id) {
                assets_to_delete.push(*asset_id);
            }
        }

        for asset_to_delete in assets_to_delete {
            edit_context.delete_asset(asset_to_delete).unwrap();
        }

        // for (asset_id, asset_disk_state) in &self.assets_disk_state {
        //     edit_context.delete_asset(*asset_id);
        // }

        let mut path_to_path_node_id = self.canonicalize_all_path_nodes(edit_context);

        let mut source_files = Vec::default();
        let mut asset_files = Vec::default();
        let mut meta_files = Vec::default();

        let mut source_files_disk_state = HashMap::<PathBuf, SourceFileDiskState>::default();
        let mut assets_disk_state = HashMap::<AssetId, AssetDiskState>::default();

        {
            profiling::scope!("Categorize files on disk");
            //
            // First visit all folders to create path nodes
            //
            let walker =
                globwalk::GlobWalkerBuilder::from_patterns(&self.file_system_root_path, &["**"])
                    .file_type(globwalk::FileType::DIR)
                    .build()
                    .unwrap();

            for file in walker {
                if let Ok(file) = file {
                    let asset_file = dunce::canonicalize(&file.path()).unwrap();
                    let asset_location = self.ensure_asset_location_exists(
                        &asset_file,
                        &mut path_to_path_node_id,
                        edit_context,
                    );
                    let asset_id = asset_location.path_node_id();

                    let asset_file_metadata =
                        FileMetadata::new(&std::fs::metadata(&asset_file).unwrap());
                    let object_hash = edit_context
                        .data_set()
                        .hash_object(asset_id, HashObjectMode::FullObjectWithLocationChainNames)
                        .unwrap();

                    assets_disk_state.insert(
                        asset_id,
                        AssetDiskState::Persisted(PersistedAssetDiskState {
                            asset_file_path: asset_file,
                            _asset_file_metadata: asset_file_metadata,
                            object_hash,
                        }),
                    );
                }
            }

            //
            // Visit all files and categorize them as meta files, asset files, or source files
            // - Asset files end in .af
            // - Meta files end in .meta
            // - Anything else is presumed to be a source file
            //
            let walker =
                globwalk::GlobWalkerBuilder::from_patterns(&self.file_system_root_path, &["**"])
                    .file_type(globwalk::FileType::FILE)
                    .build()
                    .unwrap();

            for file in walker {
                if let Ok(file) = file {
                    let file = dunce::canonicalize(&file.path()).unwrap();
                    if file.extension() == Some(OsStr::new("meta")) {
                        meta_files.push(file.to_path_buf());
                    } else if file.extension() == Some(OsStr::new("af")) {
                        asset_files.push(file.to_path_buf());
                    } else {
                        source_files.push(file.to_path_buf());
                    }
                }
            }
        }

        //
        // Scan all meta files, any asset file that exists and is referenced by a meta file will
        // be re-imported. (Because the original source asset is presumed to exist alongside the
        // meta file and source files in a path-based data source get re-imported automatically)
        //
        let mut source_file_meta_files = HashMap::<PathBuf, MetaFile>::default();
        {
            profiling::scope!("Read meta files");
            for meta_file in meta_files {
                let source_file = meta_file.with_extension("");
                if !source_file.exists() {
                    println!("Could not find source file, can't re-import data. Restore the source file or delete the meta file.");
                    continue;
                }
                //println!("meta file {:?} source file {:?}", meta_file, source_file);

                let contents = std::fs::read_to_string(meta_file.as_path()).unwrap();
                let meta_file_contents = MetaFileJson::load_from_string(&contents);

                source_file_meta_files.insert(source_file, meta_file_contents);
            }
        }

        //
        // Load any asset files.
        //
        {
            profiling::scope!("Load Asset Files");
            for asset_file in asset_files {
                //println!("asset file {:?}", asset_file);
                let contents = std::fs::read_to_string(asset_file.as_path()).unwrap();

                let asset_location = self.ensure_asset_location_exists(
                    asset_file.as_path().parent().unwrap(),
                    &mut path_to_path_node_id,
                    edit_context,
                );
                let default_asset_location =
                    AssetLocation::new(AssetId(*self.asset_source_id.uuid()));
                let schema_set = edit_context.schema_set().clone();
                let asset_id = crate::json_storage::AssetJson::load_asset_from_string(
                    edit_context,
                    &schema_set,
                    None,
                    default_asset_location,
                    Some(asset_location.clone()),
                    &contents,
                )
                .unwrap();

                let asset_file_metadata =
                    FileMetadata::new(&std::fs::metadata(&asset_file).unwrap());

                let object_hash = edit_context
                    .data_set()
                    .hash_object(asset_id, HashObjectMode::FullObjectWithLocationChainNames)
                    .unwrap();

                assets_disk_state.insert(
                    asset_id,
                    AssetDiskState::Persisted(PersistedAssetDiskState {
                        asset_file_path: asset_file,
                        _asset_file_metadata: asset_file_metadata,
                        object_hash,
                    }),
                );
            }
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
        let mut scanned_source_files = HashMap::<PathBuf, ScannedSourceFile>::default();

        {
            profiling::scope!("Scan Source Files");

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

                let importers = self
                    .importer_registry
                    .importers_for_file_extension(&extension.unwrap().to_string_lossy());

                if importers.is_empty() {
                    // No importer found
                } else if importers.len() > 1 {
                    // Multiple importers found, no way of disambiguating
                } else {
                    let importer = self.importer_registry.importer(importers[0]).unwrap();

                    let mut scanned_importables = HashMap::default();
                    {
                        profiling::scope!(&format!(
                            "Importer::scan_file {}",
                            source_file.to_string_lossy()
                        ));
                        let scan_result = importer.scan_file(ScanContext::new(
                            &source_file,
                            edit_context.schema_set(),
                            &self.importer_registry,
                            project_config,
                            &mut scanned_importables,
                            &mut import_job_to_queue.log_data.log_events,
                        ));

                        match scan_result {
                            Ok(result) => result,
                            Err(e) => {
                                import_job_to_queue
                                    .log_data
                                    .log_events
                                    .push(ImportLogEvent {
                                        path: source_file.clone(),
                                        asset_id: None,
                                        level: LogEventLevel::FatalError,
                                        message: format!(
                                            "scan_file returned error: {}",
                                            e.to_string()
                                        ),
                                    });

                                continue;
                            }
                        }
                    };

                    //println!("  find meta file {:?}", source_file);
                    let mut meta_file = source_file_meta_files
                        .get(&source_file)
                        .cloned()
                        .unwrap_or_default();
                    for (_, scanned_importable) in &scanned_importables {
                        // Does it exist in the meta file? If so, we need to reuse the ID
                        meta_file
                            .past_id_assignments
                            .entry(scanned_importable.name.clone())
                            .or_insert_with(|| AssetId::from_uuid(Uuid::new_v4()));
                    }

                    let mut meta_file_path = source_file.clone().into_os_string();
                    meta_file_path.push(".meta");

                    //let source_file_metadata = FileMetadata::new(&std::fs::metadata(&source_file).unwrap());

                    let mut importables = HashMap::<ImportableName, AssetId>::default();
                    for (_, scanned_importable) in &scanned_importables {
                        let imporable_asset_id =
                            meta_file.past_id_assignments.get(&scanned_importable.name);
                        importables.insert(
                            scanned_importable.name.clone(),
                            *imporable_asset_id.unwrap(),
                        );
                    }

                    source_files_disk_state.insert(
                        source_file.clone(),
                        SourceFileDiskState {
                            generated_assets: Default::default(),
                            persisted_assets: Default::default(),
                            //source_file_metadata,
                            _importer_id: importer.importer_id(),
                            _importables: importables,
                        },
                    );

                    std::fs::write(meta_file_path, MetaFileJson::store_to_string(&meta_file))
                        .unwrap();
                    scanned_source_files.insert(
                        source_file,
                        ScannedSourceFile {
                            meta_file,
                            importer,
                            scanned_importables: scanned_importables.into_values().collect(),
                        },
                    );
                }
            }
        }

        //
        // Re-import source files
        //
        {
            profiling::scope!("Enqueue import operations");
            for (source_file_path, scanned_source_file) in &scanned_source_files {
                let parent_dir = source_file_path.parent().unwrap();
                //println!("  import to dir {:?}", parent_dir);
                let import_location =
                    AssetLocation::new(*path_to_path_node_id.get(parent_dir).unwrap());

                let source_file_disk_state =
                    source_files_disk_state.get_mut(source_file_path).unwrap();

                let mut requested_importables = HashMap::default();
                for scanned_importable in &scanned_source_file.scanned_importables {
                    // The ID assigned to this importable. We have this now because we previously scanned
                    // all source files and assigned IDs to any importable
                    let importable_asset_id = *scanned_source_files
                        .get(source_file_path)
                        .unwrap()
                        .meta_file
                        .past_id_assignments
                        .get(&scanned_importable.name)
                        .unwrap();

                    // Create an asset name for this asset
                    let asset_name =
                        hydrate_pipeline::create_asset_name(source_file_path, scanned_importable);

                    let asset_file_exists = assets_disk_state.get(&importable_asset_id).is_some();
                    let asset_is_persisted = scanned_source_file
                        .meta_file
                        .persisted_assets
                        .contains(&importable_asset_id);

                    if asset_is_persisted && !asset_file_exists {
                        // If the asset is persisted but deleted, we do not want to import it
                        continue;
                    }

                    if !asset_is_persisted {
                        assets_disk_state.insert(
                            importable_asset_id,
                            AssetDiskState::Generated(GeneratedAssetDiskState {
                                source_file_path: source_file_path.clone(),
                            }),
                        );
                        source_file_disk_state
                            .generated_assets
                            .insert(importable_asset_id);
                    } else {
                        assert!(asset_file_exists);
                        assert_eq!(
                            edit_context
                                .asset_schema(importable_asset_id)
                                .unwrap()
                                .fingerprint(),
                            scanned_importable.asset_type.fingerprint()
                        );
                        //edit_context.set_asset_name(importable_asset_id, asset_name);
                        //edit_context.set_asset_location(importable_asset_id, *import_location);
                        //edit_context.set_import_info(importable_asset_id, import_info);

                        // We iterated through asset files already, so just check that we inserted a AssetDiskState::Persisted into this map
                        assert!(assets_disk_state
                            .get(&importable_asset_id)
                            .unwrap()
                            .is_persisted());
                        source_file_disk_state
                            .persisted_assets
                            .insert(importable_asset_id);
                    }

                    let canonical_path_references = Self::find_canonical_path_references(
                        project_config,
                        source_file_path,
                        &scanned_importable,
                        &scanned_source_files,
                    );
                    match canonical_path_references {
                        Ok(canonical_path_references) => {
                            let source_file = PathReference::new(
                                "".to_string(),
                                source_file_path.to_string_lossy().to_string(),
                                scanned_importable.name.clone(),
                            )
                            .simplify(project_config);

                            let requested_importable = RequestedImportable {
                                asset_id: importable_asset_id,
                                schema: scanned_importable.asset_type.clone(),
                                asset_name,
                                asset_location: import_location,
                                //importer_id: scanned_source_file.importer.importer_id(),
                                source_file,
                                canonical_path_references,
                                path_references: scanned_importable.referenced_source_files.clone(),
                                replace_with_default_asset: !asset_is_persisted,
                            };

                            requested_importables
                                .insert(scanned_importable.name.clone(), requested_importable);
                        }
                        Err(e) => {
                            import_job_to_queue
                                .log_data
                                .log_events
                                .push(ImportLogEvent {
                                    path: source_file_path.clone(),
                                    asset_id: Some(importable_asset_id),
                                    level: LogEventLevel::FatalError,
                                    message: format!(
                                        "While resolving references to other assets: {}",
                                        e.to_string()
                                    ),
                                });
                        }
                    }
                }

                if !requested_importables.is_empty() {
                    import_job_to_queue
                        .import_job_source_files
                        .push(ImportJobSourceFile {
                            source_file_path: source_file_path.to_path_buf(),
                            importer_id: scanned_source_file.importer.importer_id(),
                            requested_importables,
                            import_type: ImportType::ImportIfImportDataStale,
                        });
                }
            }
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

    fn flush_to_storage(
        &mut self,
        edit_context: &mut EditContext,
    ) {
        profiling::scope!(&format!(
            "flush_to_storage {:?}",
            self.file_system_root_path
        ));

        let mut pending_writes = Vec::<AssetId>::default();
        let mut pending_deletes = Vec::<AssetId>::default();

        for &asset_id in edit_context.assets().keys() {
            if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                // ignore the root asset
                continue;
            }

            if self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
                match self.assets_disk_state.get(&asset_id) {
                    None => {
                        // There is a newly created asset that has never been saved
                        pending_writes.push(asset_id);
                    }
                    Some(asset_disk_state) => {
                        let object_hash = edit_context
                            .data_set()
                            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationChainNames)
                            .unwrap();
                        match asset_disk_state {
                            AssetDiskState::Generated(_) => {
                                // We never consider a generated asset as modified, and we expect UI to never alter
                                // the asset data
                            }
                            AssetDiskState::Persisted(persisted_asset_disk_state) => {
                                if persisted_asset_disk_state.object_hash != object_hash {
                                    // The object has been modified and no longer matches disk state
                                    pending_writes.push(asset_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Is there anything that's been deleted?
        for (&asset_id, asset_disk_state) in &self.assets_disk_state {
            match asset_disk_state {
                AssetDiskState::Generated(_) => {
                    // We never consider a generated asset as modified, and we expect UI to never alter
                    // the asset data
                }
                AssetDiskState::Persisted(_) => {
                    if !edit_context.has_asset(asset_id)
                        || !self.is_asset_owned_by_this_data_source(edit_context, asset_id)
                    {
                        // There is an asset that no longer exists, but the file is still on disk
                        pending_deletes.push(asset_id);
                    }
                }
            }
        }

        // Delete files for assets that were deleted
        // for asset_id in edit_context.modified_assets() {
        //     if self.all_asset_ids_on_disk_with_original_path.contains_key(asset_id)
        //         && !edit_context.has_asset(*asset_id)
        //     {
        //         //TODO: delete the asset file
        //         self.all_asset_ids_on_disk_with_original_path.remove(asset_id);
        //     }
        // }

        //let modified_assets = self.find_all_modified_assets(edit_context);

        // We will write out any files that were modified or moved
        for asset_id in &pending_writes {
            if let Some(asset_info) = edit_context.assets().get(asset_id) {
                if self.is_asset_owned_by_this_data_source(edit_context, *asset_id) {
                    if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                        // never save the root asset
                        continue;
                    }

                    if let Some(asset_disk_state) = self.assets_disk_state.get(asset_id) {
                        if asset_disk_state.is_generated() {
                            // Never store generated assets, they exist because their source file is
                            // on disk and they aren't mutable in the editor
                            continue;
                        }
                    }

                    let containing_file_path =
                        self.containing_file_path_for_asset(edit_context, *asset_id);
                    let is_directory =
                        asset_info.schema().fingerprint() == self.path_node_schema.fingerprint();
                    let asset_file_path =
                        self.path_for_asset(&containing_file_path, *asset_id, asset_info);

                    if is_directory {
                        // It's a path node, ensure the dir exists
                        std::fs::create_dir_all(&asset_file_path).unwrap();
                    } else {
                        // It's a asset, create an asset file
                        let data = crate::json_storage::AssetJson::save_asset_to_string(
                            edit_context.schema_set(),
                            edit_context.assets(),
                            *asset_id,
                            true,
                            None,
                        );

                        std::fs::create_dir_all(&containing_file_path).unwrap();
                        std::fs::write(&asset_file_path, data).unwrap();

                        let object_hash = edit_context
                            .data_set()
                            .hash_object(
                                *asset_id,
                                HashObjectMode::FullObjectWithLocationChainNames,
                            )
                            .unwrap();

                        let asset_file_metadata =
                            FileMetadata::new(&std::fs::metadata(&asset_file_path).unwrap());
                        self.assets_disk_state.insert(
                            *asset_id,
                            AssetDiskState::Persisted(PersistedAssetDiskState {
                                _asset_file_metadata: asset_file_metadata,
                                asset_file_path: asset_file_path.clone(),
                                object_hash,
                            }),
                        );

                        // We know the asset was already persisted so we don't need to update source files state
                    }
                }
            }
        }

        let mut deferred_directory_deletes = Vec::default();

        // First pass to delete files
        for &asset_id in &pending_deletes {
            match self.assets_disk_state.get(&asset_id) {
                None => {
                    // Unexpected, assets pending deletion should be on disk. But we don't need to do anything.
                    panic!("assets pending deletion should be on disk");
                }
                Some(disk_state) => {
                    match disk_state {
                        AssetDiskState::Generated(_) => {
                            // Unexpected, generated assets should not be considered modified and so should not
                            // be pending deletion.
                            panic!("generated assets should not be considered modified and so should not be pending deletion");
                        }
                        AssetDiskState::Persisted(disk_state) => {
                            if disk_state.asset_file_path.is_dir() {
                                // Defer directory deletion so that any files that might be in them get deleted first.
                                // We can't delete directories that have files in them.
                                deferred_directory_deletes
                                    .push((asset_id, disk_state.asset_file_path.clone()));
                            } else {
                                std::fs::remove_file(&disk_state.asset_file_path).unwrap();
                                self.assets_disk_state.remove(&asset_id);
                            }
                        }
                    }
                }
            }
        }

        // Reverse sort ensures that subdirectories are processed first
        deferred_directory_deletes.sort_by(|(_, lhs), (_, rhs)| rhs.cmp(lhs));

        // Second pass to delete directories if they are empty and path node does not exist
        for (_, directory) in deferred_directory_deletes {
            let is_empty = directory.read_dir().unwrap().next().is_none();
            if is_empty {
                std::fs::remove_dir(&directory).unwrap();
            }
        }
    }

    fn edit_context_has_unsaved_changes(
        &self,
        edit_context: &EditContext,
    ) -> bool {
        for (&asset_id, asset_info) in edit_context.assets() {
            if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                // ignore the root asset
                continue;
            }

            if self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
                match self.assets_disk_state.get(&asset_id) {
                    None => {
                        // There is a newly created asset that has never been saved
                        println!("asset name: {:?}", asset_info.asset_name());
                        return true;
                    }
                    Some(asset_disk_state) => {
                        let object_hash = edit_context
                            .data_set()
                            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationChainNames)
                            .unwrap();
                        match asset_disk_state {
                            AssetDiskState::Generated(_) => {
                                // We never consider a generated asset as modified, and we expect UI to never alter
                                // the asset data
                            }
                            AssetDiskState::Persisted(persisted_asset_disk_state) => {
                                if persisted_asset_disk_state.object_hash != object_hash {
                                    // The object has been modified and no longer matches disk state
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Is there anything that's been deleted?
        for (&asset_id, asset_disk_state) in &self.assets_disk_state {
            match asset_disk_state {
                AssetDiskState::Generated(_) => {
                    // We never consider a generated asset as modified, and we expect UI to never alter
                    // the asset data
                }
                AssetDiskState::Persisted(_) => {
                    if !edit_context.has_asset(asset_id)
                        || !self.is_asset_owned_by_this_data_source(edit_context, asset_id)
                    {
                        // There is an asset that no longer exists, but the file is still on disk
                        return true;
                    }
                }
            }
        }

        return false;
    }

    fn append_pending_file_operations(
        &self,
        edit_context: &EditContext,
        pending_file_operations: &mut PendingFileOperations,
    ) {
        for (&asset_id, asset_info) in edit_context.assets() {
            if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                // ignore the root asset
                continue;
            }

            if self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
                match self.assets_disk_state.get(&asset_id) {
                    None => {
                        // There is a newly created asset that has never been saved
                        let containing_path =
                            self.containing_file_path_for_asset(edit_context, asset_id);
                        let asset_path =
                            self.path_for_asset(&containing_path, asset_id, asset_info);
                        pending_file_operations
                            .create_operations
                            .push((asset_id, asset_path));
                    }
                    Some(asset_disk_state) => {
                        let object_hash = edit_context
                            .data_set()
                            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationChainNames)
                            .unwrap();
                        match asset_disk_state {
                            AssetDiskState::Generated(_) => {
                                // We never consider a generated asset as modified, and we expect UI to never alter
                                // the asset data
                            }
                            AssetDiskState::Persisted(persisted_asset_disk_state) => {
                                if persisted_asset_disk_state.object_hash != object_hash {
                                    // The object has been modified and no longer matches disk state
                                    pending_file_operations.modify_operations.push((
                                        asset_id,
                                        persisted_asset_disk_state.asset_file_path.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Is there anything that's been deleted?
        for (&asset_id, asset_disk_state) in &self.assets_disk_state {
            match asset_disk_state {
                AssetDiskState::Generated(_) => {
                    // We never consider a generated asset as modified, and we expect UI to never alter
                    // the asset data
                }
                AssetDiskState::Persisted(persisted_asset_disk_state) => {
                    if !edit_context.has_asset(asset_id)
                        || !self.is_asset_owned_by_this_data_source(edit_context, asset_id)
                    {
                        // There is an asset that no longer exists, but the file is still on disk
                        pending_file_operations
                            .delete_operations
                            .push((asset_id, persisted_asset_disk_state.asset_file_path.clone()));
                    }
                }
            }
        }
    }
}

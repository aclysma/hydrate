use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use hydrate_data::{BuildInfo, ImportInfo, ObjectId, ObjectLocation, ObjectName, ObjectSourceId};
use hydrate_schema::{HashMap, SchemaNamedType};
use crate::{AssetEngine, EditContextObjectImportInfoJson, HashSet, import_util, Importer, ImporterRegistry, ImporterRegistryBuilder, MetaFile, MetaFileJson, PathNode, PathNodeRoot, ScannedImportable};
use crate::DataSource;
use crate::edit_context::EditContext;
use crate::import_util::ImportToQueue;

#[derive(Clone)]
struct ObjectOnDiskState {
    containing_path: PathBuf,
    full_path: PathBuf,
    name: String,
    is_directory: bool,
}

pub struct FileSystemPathBasedDataSource {
    object_source_id: ObjectSourceId,
    file_system_root_path: PathBuf,

    importer_registry: ImporterRegistry,

    // Any object ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_object_ids_on_disk_with_on_disk_state: HashMap<ObjectId, ObjectOnDiskState>,
    //all_assigned_path_ids: HashMap<PathBuf, ObjectId>,

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
    fn reload_all(&mut self, edit_context: &mut EditContext, imports_to_queue: &mut Vec<ImportToQueue>) {
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
            println!("meta file {:?} source file {:?}", meta_file, source_file);

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

        //
        // Load any asset files.
        //
        let mut asset_files_by_object_id = HashMap::default();
        for asset_file in asset_files {
            println!("asset file {:?}", asset_file);
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

            edit_context.clear_object_modified_flag(object_id);
            edit_context.clear_location_modified_flag(&object_location);
            asset_files_by_object_id.insert(asset_file, object_id);
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
            println!("source file first pass {:?}", source_file);
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
                    &source_file, edit_context.schema_set());

                println!("  find meta file {:?}", source_file);
                let mut meta_file = source_file_meta_files.get(&source_file).cloned().unwrap_or_default();
                for scanned_importable in &scanned_importables {
                    // Does it exist in the meta file? If so, we need to reuse the ID
                    meta_file.past_id_assignments
                        .entry(scanned_importable.name.as_ref().cloned().unwrap_or_default())
                        .or_insert_with(|| ObjectId::from_uuid(Uuid::new_v4()));
                }

                let mut meta_file_path = source_file.clone().into_os_string();
                meta_file_path.push(".meta");

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
            println!("  import to dir {:?}", parent_dir);
            let import_location = ObjectLocation::new(*path_to_path_node_id.get(parent_dir).unwrap());

            let mut requested_importables = HashMap::default();
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

                } else {
                    assert_eq!(edit_context.object_schema(importable_object_id).unwrap().fingerprint(), scanned_importable.asset_type.fingerprint());
                    //edit_context.set_object_name(importable_object_id, object_name);
                    //edit_context.set_object_location(importable_object_id, *import_location);
                    //edit_context.set_import_info(importable_object_id, import_info);
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
            }

            imports_to_queue.push(ImportToQueue {
                source_file_path: source_file_path.to_path_buf(),
                importer_id: scanned_source_file.importer.importer_id(),
                requested_importables
            });
        }

        // //
        // // Import the file
        // // - Reuse existing assets if they are referenced by the meta file
        // // - Create new assets if they do not exist
        // //
        // let scanned_importables = importer.scan_file(
        //     &source_file, edit_context.schema_set());
        //
        // let mut meta_file = source_file_meta_files.get(&source_file).unwrap().clone();
        // for scanned_importable in &scanned_importables {
        //     // Does it exist in the meta file? If so, we need to reuse the ID
        //     let object_id = meta_file.past_id_assignments
        //         .entry(scanned_importable.name.unwrap_or_default())
        //         .or_insert_with(ObjectId::from_uuid(Uuid::new_v4()));
        //
        //     if edit_context.has_object(*object_id) {
        //         // The object already exists, just kick a re-import of the data.
        //         // Validate import info/name?
        //         // Validate file references? Just make sure the referenced ID exists
        //     } else {
        //         // We have to create the asset. This includes:
        //         // - Creating an ImportInfo
        //         // - Choosing an object name
        //         // - Resolving referenced files. For source files in a path-based data set,
        //         //   that have no assets persisted to disk, the given file must be stored in
        //         //   this dataset at the specified location
        //     }
        //
        //     let mut file_references = Vec::default();
        //     for file_reference in &scanned_importable.file_references {
        //         file_references.push(file_reference.path.clone());
        //     }
        // }


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

        //
        // Create assets automatically for loose source files
        //
        /*
        for source_file in source_files {
            if let Some(extension)  = source_file.extension() {
                //let mut asset_file_path = source_file.clone().into_os_string();
                //asset_file_path.push(".af");
                // let asset_file_exists = asset_files.contains_key(Path::new(&asset_file_path));
                // if asset_file_exists {
                //     // We don't need to do anything
                //     println!("Source file with existing asset file {:?} {:?}", source_file, &asset_file_path);
                // } else {
                //     // Create an asset
                //     println!("Source file with no existing asset file {:?} {:?}", source_file, asset_file_path);

                println!("loose source file {:?}", source_file);

                let importers = self.importer_registry.importers_for_file_extension(&extension.to_string_lossy());
                if importers.is_empty() {
                    // No importer found
                } else if importers.len() > 1 {
                    // Multiple importers found, no way of disambiguating
                } else {
                    // Since a source file may produce many assets, we cannot simply search for a single asset by the same name
                    // with .af appended. We have to actually scan the file to see what importables are in it and if we have
                    // permanently persisted any of them to assets already. Any that haven't been persisted we will automatically
                    // import and produce "default" assets for.

                    //TODO: maybe scan and check which af files exist? provide opportunity to default-init them based on content
                    // of files?


                    let importer = self.importer_registry.importer(importers[0]).unwrap();


                    {
                        let importables = importer.scan_file(&source_file, edit_context.schema_set());
                        println!("importables: {:?}", importables);
                    }

                    //TODO: Don't unwrap
                    let parent_dir = source_file.parent().unwrap();
                    println!("  import to dir {:?}", parent_dir);
                    let import_location = path_to_path_node_id.get(parent_dir).unwrap();


                    //TODO: This is trying to import all importables, but we don't want to import anything that already exists
                    let mut imports_to_queue = Vec::default();
                    crate::pipeline::import_util::recursively_gather_import_operations_and_create_assets(
                        &source_file,
                        importer,
                        edit_context,
                        &self.importer_registry,
                        &ObjectLocation::new(*import_location),
                        &mut imports_to_queue,
                    );

                    println!("  SOURCE FILE: {:?}", source_file);
                    for import_to_queue in &imports_to_queue {
                        // We assum


                        println!("    IMPORT FILE: {:?}", import_to_queue.source_file_path);
                        for (importable_name, created_object_id) in &import_to_queue.created_objects {
                            println!("      Reference: {:?} {:?}", importable_name, created_object_id);
                        }
                    }
                }
                // }
            }
        }
         */
    }




    // Figure out if an asset file already exists. That asset should have a matching
    // source_file_path. If it doesn't, not sure how to handle it atm? What happens
    // if there already exists an asset file where we would want to write an asset
    // file? I guess we have to warn/reject if there is a source file that would
    // write to an asset file and that asset file is not completely compatible
    // with what the source file would write there?

    // Potential ways to fail:
    // - A source file references another file that exists, but we already imported
    //   it elsewhere. It becomes ambiguous whether or not we use the data that had
    //   been explicitly imported earlier or the referenced source file on disk
    //   - We can disallow any import assets from other data sources from claiming
    //     to have been imported from the path-based data source's disk location
    //   - We could be strict that if an asset claims to be sourced from a path-based
    //     disk location, the asset must be located alongside the source file


    //   - If an asset claims it was created by importing a source file that is in
    //     a path-based data source, then the asset must be named/located such that
    //     it is adjacent to the source file.
    //     - Actually any asset claiming its source file is in a path-based data
    //       source MUST be located/named consistently with that source file
    //   - Source files automatically imported in a path-based data source that
    //     do not have an asset cannot reference any files by path that are not
    //     also in that data source
    //   - This implies restrictions:
    //     - Assets cannot be renamed or moved if their source file is in a path-based
    //       data source, unless the source file is also renamed. The name/location
    //       of that source file dictates the name/location of the assets
    //     -
    //




    // - An asset file already exists at the location a source file need to write to
    //   but the asset file does not have anything to do with that source file
    //   -




    fn save_all_modified(&mut self, edit_context: &mut EditContext) {

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

                    let mut containing_file_path = self.containing_file_path_for_object(edit_context, *object_id);
                    let is_directory = edit_context.object_schema(*object_id).unwrap().fingerprint() == self.path_node_schema.fingerprint();
                    let object_name = Self::sanitize_object_name(*object_id, edit_context.object_name(*object_id));
                    let file_name = Self::file_name_for_object(&object_name, is_directory);
                    let full_file_path = containing_file_path.join(file_name);

                    let new_on_disk_state = ObjectOnDiskState {
                        containing_path: containing_file_path.clone(),
                        full_path: full_file_path.clone(),
                        is_directory,
                        name: object_name.clone()
                    };

                    if is_directory {
                        // It's a path node, ensure the dir exists
                        std::fs::create_dir_all(&full_file_path).unwrap();
                    } else {
                        // It's a object, create an asset file
                        let data = crate::json_storage::EditContextObjectJson::save_edit_context_object_to_string(
                            edit_context,
                            *object_id,
                            true,
                            None
                        );

                        std::fs::create_dir_all(&containing_file_path).unwrap();
                        std::fs::write(full_file_path, data).unwrap();
                    }

                    updated_all_object_ids_on_disk_with_on_disk_state.insert(*object_id, new_on_disk_state);
                }
            }
        }

        // Delete anything on disk that shouldn't still be on disk
        //TODO: Implement probably in passes? Files first then directories? Or maybe we can just
        // reverse sort by filename length?


        // Update all_object_ids_on_disk_with_original_path
        self.all_object_ids_on_disk_with_on_disk_state = updated_all_object_ids_on_disk_with_on_disk_state;
    }

    fn reload_all_modified(&mut self, edit_context: &mut EditContext) {
        // Find all existing modified objects
        // I think this includes added, deleted, and edited objects?
        let modified_objects = self.find_all_modified_objects(edit_context);

        let mut existing_modified_objects: Vec<_> = Default::default();
        let mut saved_modified_objects: Vec<_> = Default::default();

        for modified_object in &modified_objects {
            if let Some(object_info) = edit_context.objects().get(modified_object) {
                existing_modified_objects.push(*modified_object);
            }

            if self.all_object_ids_on_disk_with_on_disk_state.contains_key(modified_object) {
                saved_modified_objects.push(*modified_object);
            }
        }

        // Delete any modified object that exists in the edit context belonging to this data source
        for modified_object in existing_modified_objects {
            edit_context.delete_object(modified_object);
        }

        //
        let mut path_to_path_node_id = self.canonicalize_all_path_nodes(edit_context);

        // Reload any modified object that exists on disk belonging to this data source
        for modified_object in saved_modified_objects {
            let state_on_disk = self.all_object_ids_on_disk_with_on_disk_state.get(&modified_object).unwrap();

            if let Ok(contents) = std::fs::read_to_string(&state_on_disk.full_path) {
                let object_location = self.ensure_object_location_exists(
                    state_on_disk.full_path.parent().unwrap(),
                    &mut path_to_path_node_id,
                    edit_context
                );
                crate::json_storage::EditContextObjectJson::load_edit_context_object_from_string(
                    edit_context,
                    None,
                    self.object_source_id,
                    Some(object_location),
                    &contents
                );
            } else {
                // We failed to find the file
            }
        }
    }
}
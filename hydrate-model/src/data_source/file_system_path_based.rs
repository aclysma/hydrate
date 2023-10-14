use std::path::{Path, PathBuf};
use hydrate_data::{ObjectId, ObjectLocation, ObjectName, ObjectSourceId};
use hydrate_schema::{HashMap, SchemaNamedType};
use crate::{EditContextObjectImportInfoJson, HashSet, PathNode, PathNodeRoot};
use crate::DataSource;
use crate::edit_context::EditContext;

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

    // Any object ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_object_ids_on_disk_with_on_disk_state: HashMap<ObjectId, ObjectOnDiskState>,
    //all_assigned_path_ids: HashMap<PathBuf, ObjectId>,

    path_node_schema: SchemaNamedType,
    path_node_root_schema: SchemaNamedType,
}

impl FileSystemPathBasedDataSource {
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

    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
        object_source_id: ObjectSourceId,
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
            all_object_ids_on_disk_with_on_disk_state: Default::default(),
            path_node_schema,
            path_node_root_schema,
        }
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
    fn reload_all(&mut self, edit_context: &mut EditContext) {
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
        // Standard load of asset files
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(&self.file_system_root_path, &["**.af"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                println!("asset file {:?}", file);
                let contents = std::fs::read_to_string(file.path()).unwrap();

                let object_location = self.ensure_object_location_exists(
                    file.path().parent().unwrap(),
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

                edit_context.clear_object_modified_flag(object_id);
                edit_context.clear_location_modified_flag(&object_location);
            }
        }

        //
        // Create assets automatically for loose assets
        //
    }

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
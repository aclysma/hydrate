use crate::edit_context::EditContext;
use crate::{DataSource, EditorModel, HashMap, HashSet, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

// #[derive(Debug)]
// pub struct FileState {
//     // Absolute path to the file
//     path: PathBuf,
//     size_in_bytes: u64,
//     last_modified_timestamp: std::time::SystemTime,
// }
//
// impl FileState {
//     pub fn path(&self) -> &Path {
//         &self.path
//     }
// }

pub struct FileSystemTreeDataSource {
    object_source_id: ObjectSourceId,
    // Always ends with exactly one slash
    mount_path: ObjectPath,
    file_system_root_path: PathBuf,
    //file_states: HashMap<PathBuf, FileState>,
    //object_locations: HashMap<ObjectId, PathBuf>,
}

impl DataSource for FileSystemTreeDataSource {
    fn reload_all(&mut self, edit_context: &mut EditContext) {
        let walker = globwalk::GlobWalkerBuilder::new(&self.file_system_root_path, "**")
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for walker_file in walker {
            let file_path = walker_file.as_ref().unwrap().path();
            if let Some(extension) = file_path.extension() {
                if extension == OsStr::new("nxt") {
                    // nexdb text file

                    //TODO: Support mounting to a logical directory?
                    let object_location = Self::do_file_system_path_to_location(
                        self.object_source_id,
                        &self.mount_path,
                        &self.file_system_root_path,
                        file_path,
                    )
                        .unwrap();
                    let contents = std::fs::read_to_string(file_path).unwrap();

                    let objects = crate::data_storage::json::TreeSourceDataStorageJsonSingleFile::load_objects_from_string(
                        edit_context,
                        object_location.clone(),
                        &contents,
                    );
                    // for object in objects {
                    //     object_locations.insert(object, file_path.to_path_buf());
                    // }

                    log::info!("Loaded {} objects from {:?}", objects.len(), file_path);
                    for object in objects {
                        //loaded_objects.insert(object);
                        edit_context.clear_object_modified_flag(object);
                    }
                    //
                    // loaded_locations.insert(object_location);
                    edit_context.clear_location_modified_flag(&object_location);
                }
            }
        }
    }

    fn save_all_modified(&mut self, edit_context: &mut EditContext) {
        //
        // Build a list of locations (i.e. files) we need to save
        //
        let mut locations_to_save = HashMap::default();
        for modified_location in edit_context.modified_locations() {
            if modified_location.source() == self.object_source_id {
                locations_to_save.insert(modified_location, Vec::default());
            }
        }

        //
        // Build a list of object IDs that need to be included in those files
        //
        for (object_id, object_info) in edit_context.objects() {
            let location = object_info.object_location();
            if let Some(objects) = locations_to_save.get_mut(location) {
                objects.push(*object_id);
            }
        }


        //
        // Write the files to disk, including all objects that should be present in them
        //
        for (location, object_ids) in locations_to_save {
            if object_ids.is_empty() {
                //TODO: Handle rewriting files that had all objects removed?
                // delete the file
            } else {
                let data =
                    crate::data_storage::json::TreeSourceDataStorageJsonSingleFile::store_objects_to_string(edit_context, &object_ids);
                let file_path = self.location_to_file_system_path(&location).unwrap();
                std::fs::write(file_path, data).unwrap();
            }
        }
    }

    fn reload_all_modified(&mut self, edit_context: &mut EditContext) {
        //
        // Build a list of locations (i.e. files) we need to revert
        //
        let mut locations_to_revert = HashMap::default();
        for modified_location in edit_context.modified_locations() {
            if modified_location.source() == self.object_source_id {
                locations_to_revert.insert(modified_location.clone(), Vec::default());
            }
        }

        //
        // Build a list of object IDs that need to be reverted (including deleting new objects)
        //
        for (object_id, object_info) in edit_context.objects() {
            let location = object_info.object_location();
            if let Some(objects) = locations_to_revert.get_mut(location) {
                objects.push(*object_id);
            }
        }

        //
        // Delete all objects known to exist in the files we are about to reload
        //
        for (_, object_ids) in &locations_to_revert {
            for object in object_ids {
                edit_context.delete_object(*object);
            }
        }

        //
        // Reload the files from disk
        //
        for (location, object_ids) in locations_to_revert {
            let file_path = self.location_to_file_system_path(&location).unwrap();
            let data = std::fs::read_to_string(file_path).unwrap();

            crate::data_storage::json::TreeSourceDataStorageJsonSingleFile::load_objects_from_string(edit_context, location, &data);

            //let source = self.data_sources.get(&location.source()).unwrap();
            //std::fs::write(file_path, data).unwrap();
        }
    }
}

impl FileSystemTreeDataSource {
    pub fn mount_path(&self) -> &ObjectPath {
        &self.mount_path
    }

    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        mount_path: ObjectPath,
        edit_context: &mut EditContext,
        //loaded_objects: &mut HashSet<ObjectId>,
       //loaded_locations: &mut HashSet<ObjectLocation>,
    ) -> Self {
        // Mount path should end in exactly one slash (we append paths to the end of it)
        assert!(mount_path.as_string().ends_with("/"));
        assert!(!mount_path.as_string().ends_with("//"));

        let object_source_id = ObjectSourceId::new();
        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system tree data source {:?} at mount point {:?}",
            file_system_root_path,
            mount_path
        );

        FileSystemTreeDataSource {
            object_source_id,
            mount_path,
            file_system_root_path,
            //file_states,
            //object_locations
        }
    }

    fn do_file_system_path_to_location(
        object_source_id: ObjectSourceId,
        mount_path: &ObjectPath,
        file_system_root_path: &Path,
        file_path: &Path,
    ) -> Option<ObjectLocation> {
        let relative_path_from_root = file_path
            .strip_prefix(file_system_root_path)
            .ok()?
            .to_str()?;
        let virtual_path = mount_path.join(relative_path_from_root);
        Some(ObjectLocation::new(object_source_id, virtual_path))
    }

    pub fn location_to_file_system_path(
        &self,
        object_location: &ObjectLocation,
    ) -> Option<PathBuf> {
        let path = object_location.path().strip_prefix(&self.mount_path)?;
        let relative_file_path = Path::new(path.as_string());
        let absolute_file_path = self.file_system_root_path.join(relative_file_path);
        assert!(absolute_file_path.starts_with(&self.file_system_root_path));
        Some(absolute_file_path)
    }

    // pub fn file_system_root_path(&self) -> &Path {
    //     &self.file_system_root_path
    // }

    // pub fn file_states(&self) -> &HashMap<PathBuf, FileState> {
    //     &self.file_states
    // }

    // pub fn object_locations(&self) -> &HashMap<ObjectId, PathBuf> {
    //     &self.object_locations
    // }
}

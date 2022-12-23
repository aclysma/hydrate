use std::ffi::OsStr;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::edit_context::EditContext;
use crate::{DataSet, DataSource, HashMap, HashSet, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};
use crate::storage::dir_tree_blob_store::{path_to_uuid, uuid_to_path};

#[derive(Serialize, Deserialize, Debug)]
struct DirectoryFile {
    name: String,
    parent_dir: Option<Uuid>
}


fn find_dir_files(root_path: &Path) -> HashMap<Uuid, DirectoryFile> {
    let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.d"])
        .file_type(globwalk::FileType::FILE)
        .build()
        .unwrap();

    let mut directories = HashMap::<Uuid, DirectoryFile>::default();

    for file in walker {
        if let Ok(file) = file {
            println!("dir file {:?}", file);
            let dir_uuid = path_to_uuid(root_path, file.path()).unwrap();
            let contents = std::fs::read_to_string(file.path()).unwrap();
            let dir_file: DirectoryFile = serde_json::from_str(&contents).unwrap();

            directories.insert(dir_uuid, dir_file);
        }
    }

    directories
}

fn load_asset_files(
    edit_context: &mut EditContext,
    root_path: &Path,
    object_source_id: ObjectSourceId,
    dir_uuid_to_path: &HashMap::<Uuid, ObjectPath>,
    all_object_ids_on_disk: &mut HashSet<ObjectId>,
) {
    let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.af"])
        .file_type(globwalk::FileType::FILE)
        .build()
        .unwrap();

    for file in walker {
        if let Ok(file) = file {
            println!("asset file {:?}", file);
            let file_uuid = path_to_uuid(root_path, file.path()).unwrap();
            let contents = std::fs::read_to_string(file.path()).unwrap();
            crate::data_storage::json::ObjectSourceDataStorageJsonObject::load_object_from_string(edit_context, file_uuid, &contents, |parent_uuid| {
                let path = if let Some(parent_uuid) = parent_uuid {
                    dir_uuid_to_path.get(&parent_uuid)
                } else {
                    Some(ObjectPath::root_ref())
                };

                ObjectLocation::new(object_source_id, path.unwrap_or(&ObjectPath::root()).clone())
            });
            let object_id = ObjectId(file_uuid.as_u128());
            let object_location = edit_context.objects().get(&object_id).unwrap().object_location.clone();
            edit_context.clear_object_modified_flag(object_id);
            edit_context.clear_location_modified_flag(&object_location);
            all_object_ids_on_disk.insert(object_id);
        }
    }
}

pub struct FileSystemObjectDataSource {
    object_source_id: ObjectSourceId,
    file_system_root_path: PathBuf,
    //file_states: HashMap<PathBuf, FileState>,
    //object_locations: HashMap<ObjectId, PathBuf>,
    dir_uuid_to_path: HashMap<Uuid, ObjectPath>,
    path_to_dir_uuid: HashMap<ObjectPath, Uuid>,

    // Any object ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_object_ids_on_disk: HashSet<ObjectId>,
}

impl DataSource for FileSystemObjectDataSource {
    fn reload_all(&mut self, edit_context: &mut EditContext) {
        let dir_files = find_dir_files(&self.file_system_root_path);

        let mut dir_uuid_to_path = HashMap::<Uuid, ObjectPath>::default();
        let mut path_to_dir_uuid = HashMap::<ObjectPath, Uuid>::default();
        for (uuid, dir_file) in &dir_files {
            let mut parent_names: Vec<String> = Default::default();
            parent_names.push(dir_file.name.clone());

            let mut df = dir_file;
            while let Some(parent_dir) = df.parent_dir {
                if let Some(parent_dir_file) = dir_files.get(&parent_dir) {
                    parent_names.push(parent_dir_file.name.clone());
                    df = parent_dir_file;
                } else {
                    //TODO: Could not find parent, how do we handle?
                    break;
                }
            }

            let mut path = ObjectPath::root();
            for parent_name in parent_names.iter().rev() {
                path = path.join(parent_name);
            }

            dir_uuid_to_path.insert(*uuid, path.clone());
            path_to_dir_uuid.insert(path, *uuid);
        }

        //println!("dir_uuid_to_path {:?}", dir_uuid_to_path);

        load_asset_files(edit_context, &self.file_system_root_path, self.object_source_id, &dir_uuid_to_path, &mut self.all_object_ids_on_disk);
        self.dir_uuid_to_path = dir_uuid_to_path;
        self.path_to_dir_uuid = path_to_dir_uuid;
    }


    fn save_all_modified(&mut self, edit_context: &mut EditContext) {
        // Delete files for objects that were deleted
        for object_id in edit_context.modified_objects() {
            if self.all_object_ids_on_disk.contains(object_id) && !edit_context.has_object(*object_id) {
                //TODO: delete the object file
                self.all_object_ids_on_disk.remove(object_id);
            }
        }

        for object_id in edit_context.modified_objects() {
            if let Some(object_info) = edit_context.objects().get(object_id) {
                if object_info.object_location().source() == self.object_source_id {
                    let object_path = object_info.object_location.path();
                    //let parent_dir = self.path_to_dir_uuid.get(object_path).copied();

                    //TODO: create dir objects?
                    let parent_dir = self.get_or_create_dir(object_path);

                    let data = crate::data_storage::json::ObjectSourceDataStorageJsonObject::save_object_to_string(edit_context, *object_id, parent_dir);
                    let file_path = uuid_to_path(&self.file_system_root_path, object_id.as_uuid(), "af");
                    self.all_object_ids_on_disk.insert(*object_id);

                    if let Some(parent) = file_path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }

                    std::fs::write(file_path, data).unwrap();
                }
            }
        }
    }

    fn reload_all_modified(&mut self, edit_context: &mut EditContext) {
        let mut existing_modified_objects: Vec<_> = Default::default();
        let mut saved_modified_objects: Vec<_> = Default::default();

        // Find all existing modified objects
        for modified_object in edit_context.modified_objects() {
            if let Some(object_info) = edit_context.objects().get(modified_object) {
                if object_info.object_location().source() == self.object_source_id {
                    existing_modified_objects.push(*modified_object);
                }
            }

            if self.all_object_ids_on_disk.contains(modified_object) {
                saved_modified_objects.push(*modified_object);
            }
        }

        // Delete any modified object that exists in the edit context belonging to this data source
        for modified_object in existing_modified_objects {
            edit_context.delete_object(modified_object);
        }

        // Reload any modified object that exists on disk belonging to this data source
        for modified_object in saved_modified_objects {
            let file_path = uuid_to_path(&self.file_system_root_path, modified_object.as_uuid(), "af");

            if let Ok(contents) = std::fs::read_to_string(file_path) {
                crate::data_storage::json::ObjectSourceDataStorageJsonObject::load_object_from_string(edit_context, modified_object.as_uuid(), &contents, |parent_uuid| {
                    let path = if let Some(parent_uuid) = parent_uuid {
                        self.dir_uuid_to_path.get(&parent_uuid)
                    } else {
                        Some(ObjectPath::root_ref())
                    };

                    ObjectLocation::new(self.object_source_id, path.unwrap_or(&ObjectPath::root()).clone())
                });
            }
        }
    }
}

impl FileSystemObjectDataSource {
    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
    ) -> Self {
        let object_source_id = ObjectSourceId::new();
        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system object data source {:?}",
            file_system_root_path,
        );

        FileSystemObjectDataSource {
            object_source_id,
            file_system_root_path: file_system_root_path.into(),
            dir_uuid_to_path: Default::default(),
            path_to_dir_uuid: Default::default(),
            all_object_ids_on_disk: Default::default(),
        }
    }

    fn get_or_create_dir(&mut self, path: &ObjectPath) -> Option<Uuid> {
        // Root always exists, does not need a path node
        if path.is_root_path() {
            return None;
        }

        unimplemented!();
    }

    /*
    fn get_or_create_dir(&mut self, path: &ObjectPath) -> Option<Uuid> {
        // Root always exists, does not need a dir file/UUID
        if path.is_root_path() {
            return None;
        }

        if let Some(uuid) = self.path_to_dir_uuid.get(path) {
            // Dir exists, return it
            Some(*uuid)
        } else {
            // Dir doesn't exist, get_or_create the parent, then create the dir.
            // We can assume this returns Some because we early-out above if it's the root path
            let (parent_path, name) = path.parent_path_and_name().unwrap();
            let parent_path_uuid = self.get_or_create_dir(&parent_path);

            // let parent_path_uuid = if let Some(parent_path) = parent_path {
            //     // Parent isn't a root path, get or create it
            //     self.get_or_create_dir(&parent_path)
            // } else {
            //     // Parent dir is root path and doesn't need to be created
            //     None
            // };

            let dir_uuid = Uuid::new_v4();

            //
            // Write the dir file
            //
            let dir_file = DirectoryFile {
                parent_dir: parent_path_uuid,
                name
            };

            let dir_file_contents = serde_json::to_string_pretty(&dir_file).unwrap();
            let dir_file_path = uuid_to_path(&self.file_system_root_path, dir_uuid, "d");

            // Create directories if needed
            if let Some(parent) = dir_file_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            // Write the file
            std::fs::write(dir_file_path, dir_file_contents).unwrap();

            //
            // Update local state
            //
            self.path_to_dir_uuid.insert(path.clone(), dir_uuid);
            self.dir_uuid_to_path.insert(dir_uuid, path.clone());

            Some(dir_uuid)
        }

        // if !self.path_to_dir_uuid.contains_key(&path) {
        //     let parent_path_uuid = if let Some(parent_path) = path.parent_path() {
        //         self.ensure_dir_objects_exist(parent_path)
        //     } else {
        //         None
        //     };
        //
        //     DirectoryFile {
        //         parent_dir:
        //     }
        //
        //     Uuid::new_v4();
        //
        //     // let components = path.split_components();
        //     //
        //     // let mut path = ObjectPath::root();
        //     // for component in components {
        //     //     path = path.join(component);
        //     //     if !self.path_to_dir_uuid
        //     // }
        // }


    }
    */

    // pub fn object_id_to_file_system_path(
    //     &self,
    //     object_id: ObjectId,
    // ) -> PathBuf {
    //     uuid_to_path(&self.file_system_root_path, object_id.as_uuid())
    // }
}

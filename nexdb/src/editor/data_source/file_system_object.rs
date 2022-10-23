use std::ffi::OsStr;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::edit_context::EditContext;
use crate::{DataSet, DataSource, HashMap, HashSet, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};

#[derive(Serialize, Deserialize, Debug)]
struct DirectoryFile {
    name: String,
    parent_dir: Option<Uuid>
}

fn uuid_to_path(root: &Path, uuid: Uuid) -> PathBuf {
    // Convert UUID to a 32-character hex string (no hyphens)
    // example: 8cf25195abd839981ea3c93c8fd2843f
    let mut buffer = [0; 32];
    let encoded = uuid.to_simple().encode_lower(&mut buffer).to_string();
    // Produce path like [root]/8/cf/25195abd839981ea3c93c8fd2843f
    root.join(&encoded[0..1]).join(&encoded[1..3]).join(&encoded[3..32])
}

fn path_to_uuid(root: &Path, file_path: &Path) -> Option<Uuid> {
    // Remove root from the path
    let relative_path_from_root = file_path
        .strip_prefix(root)
        .ok()?;

    // We append the path into this string
    let mut path_and_name = String::with_capacity(32);

    // Split the path by directory paths
    let components: Vec<_> = relative_path_from_root.components().collect();

    // Iterate all segments of the path except the last one
    if components.len() > 1 {
        for component in components[0..components.len()-1].iter() {
            path_and_name.push_str(&component.as_os_str().to_str().unwrap());
        }
    }

    // Append the last segment, removing the extension if there is one
    if let Some(last_component) = components.last() {
        let mut last_str = last_component.as_os_str().to_str()?;

        // Remove the extension
        if let Some(extension_begin) = last_str.find('.') {
            last_str = last_str.strip_suffix(&last_str[extension_begin..]).unwrap();
        }

        // Add zero padding between dirs (which should be highest order bits) and filename
        //TODO: Maybe just assert all the component lengths are as expected
        let str_len = path_and_name.len() + last_str.len();
        if str_len < 32 {
            path_and_name.push_str(&"0".repeat(32 - str_len));
        }

        path_and_name.push_str(last_str);
    }

    u128::from_str_radix(&path_and_name, 16).ok().map(|x| Uuid::from_u128(x))
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
    mount_path: &ObjectPath,
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
            let object_location = ObjectLocation::new(object_source_id, ObjectPath::new("db:/"));
            let contents = std::fs::read_to_string(file.path()).unwrap();
            crate::data_storage::json::ObjectSourceDataStorageJsonObject::load_object_from_string(edit_context, file_uuid, &contents, |parent_uuid| {
                let path = if let Some(parent_uuid) = parent_uuid {
                    dir_uuid_to_path.get(&parent_uuid)
                } else {
                    Some(mount_path)
                };

                ObjectLocation::new(object_source_id, path.unwrap_or(mount_path).clone())
            });
            let object_id = ObjectId(file_uuid.as_u128());
            edit_context.clear_object_modified_flag(object_id);
            edit_context.clear_location_modified_flag(&object_location);
            all_object_ids_on_disk.insert(object_id);
        }
    }
}

pub struct FileSystemObjectDataSource {
    object_source_id: ObjectSourceId,
    // Always ends with exactly one slash
    mount_path: ObjectPath,
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

            let mut path = self.mount_path.clone();
            for parent_name in parent_names.iter().rev() {
                path = path.join(parent_name);
            }

            dir_uuid_to_path.insert(*uuid, path.clone());
            path_to_dir_uuid.insert(path, *uuid);
        }

        //println!("dir_uuid_to_path {:?}", dir_uuid_to_path);

        load_asset_files(edit_context, &self.file_system_root_path, &self.mount_path, self.object_source_id, &dir_uuid_to_path, &mut self.all_object_ids_on_disk);
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
            //for (object_id, object_info) in edit_context.objects() {
                    //if edit_context.modified_objects().contains(object_id) {
                        if let Some(object_info) = edit_context.objects().get(object_id) {
                            if object_info.object_location().source() == self.object_source_id {
                                let object_path = object_info.object_location.path();
                                let parent_dir = self.path_to_dir_uuid.get(object_path).copied();
                                let data = crate::data_storage::json::ObjectSourceDataStorageJsonObject::save_object_to_string(edit_context, *object_id, parent_dir);
                                let file_path = uuid_to_path(&self.file_system_root_path, object_id.as_uuid());
                                self.all_object_ids_on_disk.insert(*object_id);
                                std::fs::write(file_path, data).unwrap();
                            }
                        }
                    //}
            //}
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
            let file_path = uuid_to_path(&self.file_system_root_path, modified_object.as_uuid());

            if let Ok(contents) = std::fs::read_to_string(file_path) {
                crate::data_storage::json::ObjectSourceDataStorageJsonObject::load_object_from_string(edit_context, modified_object.as_uuid(), &contents, |parent_uuid| {
                    let path = if let Some(parent_uuid) = parent_uuid {
                        self.dir_uuid_to_path.get(&parent_uuid)
                    } else {
                        Some(&self.mount_path)
                    };

                    ObjectLocation::new(self.object_source_id, path.unwrap_or(&self.mount_path).clone())
                });
            }
        }
    }
}

impl FileSystemObjectDataSource {
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
    ) -> Self {
        // Mount path should end in exactly one slash (we append paths to the end of it)
        assert!(mount_path.as_string().ends_with("/"));
        assert!(!mount_path.as_string().ends_with("//"));

        let object_source_id = ObjectSourceId::new();
        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system object data source {:?} at mount point {:?}",
            file_system_root_path,
            mount_path
        );

        FileSystemObjectDataSource {
            object_source_id,
            mount_path,
            file_system_root_path: file_system_root_path.into(),
            dir_uuid_to_path: Default::default(),
            path_to_dir_uuid: Default::default(),
            all_object_ids_on_disk: Default::default(),
        }
    }

    pub fn object_id_to_file_system_path(
        &self,
        object_id: ObjectId,
    ) -> PathBuf {
        uuid_to_path(&self.file_system_root_path, object_id.as_uuid())
    }
}

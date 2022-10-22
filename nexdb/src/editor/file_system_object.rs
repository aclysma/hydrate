use std::ffi::OsStr;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::edit_context::EditContext;
use crate::{HashMap, HashSet, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};

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

#[derive(Debug)]
pub struct FileState {
    // Absolute path to the file
    path: PathBuf,
    size_in_bytes: u64,
    last_modified_timestamp: std::time::SystemTime,
}

impl FileState {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub struct FileSystemObjectDataSource {
    object_source_id: ObjectSourceId,
    // Always ends with exactly one slash
    mount_path: ObjectPath,
    file_system_root_path: PathBuf,
    //file_states: HashMap<PathBuf, FileState>,
    //object_locations: HashMap<ObjectId, PathBuf>,
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

fn find_asset_files(root_path: &Path) {
    let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.af"])
        .file_type(globwalk::FileType::FILE)
        .build()
        .unwrap();



    for file in walker {
        if let Ok(file) = file {
            println!("asset file {:?}", file);
            let file_uuid = path_to_uuid(root_path, file.path()).unwrap();
            let contents = std::fs::read_to_string(file.path()).unwrap();
            //nexdb::

            // let objects = crate::data_storage::json::DataStorageJsonSingleFile::load_string(
            //     edit_context,
            //     object_location.clone(),
            //     &contents,
            // );

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
        loaded_objects: &mut HashSet<ObjectId>,
        loaded_locations: &mut HashSet<ObjectLocation>,
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

        find_dir_files(&file_system_root_path);
        find_asset_files(&file_system_root_path);

        /*
        {

        }

        let walker = globwalk::GlobWalkerBuilder::from_patterns(&file_system_root_path, &["**.ta"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        let walker = globwalk::GlobWalkerBuilder::from_patterns(&file_system_root_path, &["**.ta"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        let mut file_states: HashMap<PathBuf, FileState> = Default::default();

        for file_path in walker {
            let file = file_path.unwrap();
            let metadata = std::fs::metadata(file.path()).unwrap();
            let last_modified_timestamp = metadata.modified().unwrap();
            let size_in_bytes = metadata.len();

            let file_state = FileState {
                path: file.path().to_path_buf(),
                last_modified_timestamp,
                size_in_bytes,
            };

            file_states.insert(file.path().to_path_buf(), file_state);
        }

        for (file_path, _) in &file_states {
            log::debug!("file state: {:?}", file_path);
            if let Some(extension) = file_path.extension() {
                if extension == OsStr::new("af") {
                    // asset file
                    if let Some(object_id) = path_to_uuid(&file_system_root_path, file_path).map(|x| ObjectId(x.as_u128())) {
                        log::debug!("Found file uuid {}", object_id.as_uuid());
                    } else {
                        // Warn file unexpected format?
                        continue;
                    }


                    //let location = ObjectLocation::new(object_source_id, "");

                    let contents = std::fs::read_to_string(file_path).unwrap();


                    /*
                    //TODO: Support mounting to a logical directory?
                    let object_location = Self::do_file_system_path_to_location(
                        object_source_id,
                        &mount_path,
                        &file_system_root_path,
                        file_path,
                    )
                        .unwrap();
                    let contents = std::fs::read_to_string(file_path).unwrap();

                    let objects = crate::data_storage::json::DataStorageJsonSingleFile::load_string(
                        edit_context,
                        object_location.clone(),
                        &contents,
                    );
                    // for object in objects {
                    //     object_locations.insert(object, file_path.to_path_buf());
                    // }

                    log::info!("Loaded {} objects from {:?}", objects.len(), file_path);
                    for object in objects {
                        loaded_objects.insert(object);
                    }

                    loaded_locations.insert(object_location);
                    */
                }
            }
        }
*/
        FileSystemObjectDataSource {
            object_source_id,
            mount_path,
            file_system_root_path: file_system_root_path.into(),
            //file_states,
            //object_locations
        }
    }

    fn file_system_path_to_object_id(
        object_source_id: ObjectSourceId,
        mount_path: &ObjectPath,
        file_system_root_path: &Path,
        file_path: &Path,
    ) -> Option<ObjectId> {

        path_to_uuid(file_system_root_path, file_path).map(|x| ObjectId(x.as_u128()))
        // if let Some(uuid) = uuid {
        //     let path = uuid_to_path(file_system_root_path, uuid);
        //     println!("Found file with UUID {}, regenerated path {:?}", uuid, path);
        // }

/*
        //let name = file_path.file_stem()?.to_str()?;
        println!("db file path: {:?}", file_path);


        let relative_path_from_root = file_path
            .strip_prefix(file_system_root_path)
            .ok()?;


        let components: Vec<_> = relative_path_from_root.components().collect();
        let mut path_and_name = String::new();

        if components.len() > 1 {
            for component in components[0..components.len()-1].iter() {
                path_and_name.push_str(&component.as_os_str().to_str().unwrap());
            }
        }

        if let Some(last_component) = components.last() {
            let last_str = last_component.as_os_str().to_str().unwrap();
            if let Some(extension_begin) = last_str.rfind('.') {
                //str = last_str.strip_suffix(&str[extension_begin..]).unwrap().to_string();
                path_and_name.push_str(last_str.strip_suffix(&last_str[extension_begin..]).unwrap());
            } else {
                path_and_name.push_str(last_str);
            }

            //last.as_os_str().to_str().unwrap().strip_suffix()
        }

        //path_and_name.push_str(name);
        println!("path_and_name {}", path_and_name);

        if let Some(converted) = u128::from_str_radix(&path_and_name, 16).ok() {
            let guid = Uuid::from_u128(converted);
            println!("UUID is {}", guid);
            let mut buffer = [0; 32];
            let encoded = guid.to_simple().encode_lower(&mut buffer).to_string();
            println!("encoded {}", encoded);
            let new_path = file_system_root_path.join(&encoded[0..1]).join(&encoded[1..3]).join(&encoded[3..32]);
            println!("path {}", new_path.as_os_str().to_str().unwrap());

        }


*/

/*

        let relative_path_from_root = file_path
            .strip_prefix(file_system_root_path)
            .ok()?
            .to_str()?;

        let components = relative_path_from_root.split_components();
        let mut str = String::default();
        for component in components {
            str.push_str(component);
        }

        if let Some(extension_begin) = str.rfind('.') {
            str = str.strip_suffix(&str[extension_begin..]).unwrap().to_string();
        }
*/
        //println!("db file: {}", str);

        //None

        //Some(ObjectLocation::new(object_source_id, virtual_path))
    }

    // fn do_file_system_path_to_location(
    //     object_source_id: ObjectSourceId,
    //     mount_path: &ObjectPath,
    //     file_system_root_path: &Path,
    //     file_path: &Path,
    // ) -> Option<ObjectLocation> {
    //     let relative_path_from_root = file_path
    //         .strip_prefix(file_system_root_path)
    //         .ok()?
    //         .to_str()?;
    //
    //     let components = mount_path.split_components();
    //     let mut str = String::default();
    //     for component in components {
    //         str.push_str(component);
    //     }
    //
    //
    //
    //     Some(ObjectLocation::new(object_source_id, virtual_path))
    // }

    pub fn file_system_path_to_location(
        &self,
        path: &Path,
    ) -> Option<ObjectLocation> {
        unimplemented!();
    }

    pub fn location_to_file_system_path(
        &self,
        object_location: &ObjectLocation,
    ) -> Option<PathBuf> {
        unimplemented!();
    }
}
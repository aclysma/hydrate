use crate::edit_context::EditContext;
use crate::{HashMap, ObjectLocation, ObjectPath, ObjectSourceId};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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

pub struct FileSystemDataSource {
    object_source_id: ObjectSourceId,
    mount_path: ObjectPath,
    file_system_root_path: PathBuf,
    file_states: HashMap<PathBuf, FileState>,
    //object_locations: HashMap<ObjectId, PathBuf>,
}

impl FileSystemDataSource {
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
            "Creating file system data source {:?} at mount point {:?}",
            file_system_root_path,
            mount_path
        );

        let walker = globwalk::GlobWalkerBuilder::new(&file_system_root_path, "**")
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        let mut file_states: HashMap<PathBuf, FileState> = Default::default();

        for file_path in walker {
            println!("walk path {:?}", file_path);
            let file = file_path.unwrap();
            //file.
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

        //let mut object_locations: HashMap<ObjectId, PathBuf> = Default::default();

        for (file_path, _) in &file_states {
            if let Some(extension) = file_path.extension() {
                if extension == OsStr::new("nxt") {
                    // nexdb text file

                    //TODO: Support mounting to a logical directory?
                    let object_location = Self::do_file_system_path_to_location(
                        object_source_id,
                        &mount_path,
                        &file_system_root_path,
                        file_path,
                    )
                    .unwrap();
                    let contents = std::fs::read_to_string(file_path).unwrap();

                    let objects = crate::data_storage::DataStorageJsonSingleFile::load_string(
                        edit_context,
                        object_location,
                        &contents,
                    );
                    // for object in objects {
                    //     object_locations.insert(object, file_path.to_path_buf());
                    // }

                    log::info!("Loaded {} objects from {:?}", objects.len(), file_path);
                }
            }
        }

        FileSystemDataSource {
            object_source_id,
            mount_path,
            file_system_root_path: file_system_root_path.into(),
            file_states,
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

    pub fn file_system_path_to_location(
        &self,
        path: &Path,
    ) -> Option<ObjectLocation> {
        Self::do_file_system_path_to_location(
            self.object_source_id,
            &self.mount_path,
            &self.file_system_root_path,
            path,
        )
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

    pub fn file_system_root_path(&self) -> &Path {
        &self.file_system_root_path
    }

    pub fn file_states(&self) -> &HashMap<PathBuf, FileState> {
        &self.file_states
    }

    // pub fn object_locations(&self) -> &HashMap<ObjectId, PathBuf> {
    //     &self.object_locations
    // }

    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }
}

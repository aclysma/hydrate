use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use crate::{ObjectId, HashSet, HashMap, Database};




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
    root_path: PathBuf,
    file_states: HashMap<PathBuf, FileState>,
    object_locations: HashMap<ObjectId, PathBuf>,
}

impl FileSystemDataSource {
    pub fn new<T: Into<PathBuf>>(root_path: T, db: &mut Database) -> Self {
        let root_path = root_path.into();
        log::info!(
            "Creating file system data source {:?}",
            root_path
        );

        let walker = globwalk::GlobWalkerBuilder::new(&root_path, "*")
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        let mut file_states: HashMap<PathBuf, FileState> = Default::default();

        for file_path in walker {
            println!("path {:?}", file_path);
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

        let mut object_locations: HashMap<ObjectId, PathBuf> = Default::default();

        for (k, v) in &file_states {
            if let Some(extension) = k.extension() {
                if extension == OsStr::new("nxt") {
                    // nexdb text file
                    let contents = std::fs::read_to_string(k).unwrap();
                    let objects = crate::data_storage::DataStorageJsonSingleFile::load_string(db, &contents);
                    for object in objects {
                        object_locations.insert(object, k.to_path_buf());
                    }
                }
            }
        }

        FileSystemDataSource {
            root_path: root_path.into(),
            file_states,
            object_locations
        }
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn file_states(&self) -> &HashMap<PathBuf, FileState> {
        &self.file_states
    }

    pub fn object_locations(&self) -> &HashMap<ObjectId, PathBuf> {
        &self.object_locations
    }
}

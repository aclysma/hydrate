use std::path::Path;
use std::time::SystemTime;
use nexdb::dir_tree_blob_store::path_to_uuid;
use nexdb::{HashMap, ObjectId};

pub struct FileMetadata {
    pub size: u64,
    pub modified: SystemTime,
}


struct ChangeDetector {
    //asset_hashes:
}

impl ChangeDetector {
    // pub fn scan_asset_files(root_path: &Path) {
    //     let mut asset_file_metadata = HashMap::default();
    //
    //     let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.af"])
    //         .file_type(globwalk::FileType::FILE)
    //         .build()
    //         .unwrap();
    //
    //     for file in walker {
    //         if let Ok(file) = file {
    //             println!("dir file {:?}", file);
    //             //let dir_uuid = path_to_uuid(root_path, file.path()).unwrap();
    //             //let object_id = ObjectId(dir_uuid.as_u128());
    //             let metadata = file.metadata().unwrap();
    //             let modified = metadata.modified().unwrap();
    //             let size = metadata.len();
    //
    //             asset_file_metadata.insert(file.path().to_path_buf(), FileMetadata {
    //                 size,
    //                 modified
    //             });
    //         }
    //     }
    // }
}
//use std::path::PathBuf;
//use slotmap::DenseSlotMap;
use uuid::Uuid;

pub mod dir_tree_blob_store;

// Scan files, collect last modified time/size
// Keep track of ref counts



slotmap::new_key_type! { pub struct LoadRequest; }



struct BytesHandle {

}


// struct StorageLocation {
//     path: PathBuf,
//     id: Uuid
// }



// Assets are always "rooted"
// Imported data has same ID as the asset
// Built data has same ID as asset
// Buffers are reffed by N assets, we have to walk tree to determine if they are unreferenced and
// should be cleared.. or store reference count in the buffer? Or the referencing assets in the buffer?



enum DataType {
    AssetObj,
    ImportObj,
    BuiltObj,
    Buffer,
    Directory
}

struct LoadedBytes {
    load_count: u32,
    bytes: Option<Box<[u8]>>
}

struct StorageManager {
    asset_storage: DirTreeStorageHandler,
    import_storage: DirTreeStorageHandler,
    build_storage: DirTreeStorageHandler,
}

impl StorageManager {

}

struct DirTreeStorageHandler {
    //loaded_bytes: DenseSlotMap<Uuid, LoadedBytes>,
}

impl DirTreeStorageHandler {
    // Blocking read of string data
    pub fn read_string(uuid: Uuid) -> String {
        unimplemented!();
    }

    // Blocking write of string data
    pub fn write_string(uuid: Uuid, data: &str) {
        unimplemented!();
    }

    // Blocking read of binary data
    pub fn read_bytes(uuid: Uuid) -> Box<[u8]> {
        unimplemented!();
    }

    // Blocking write of binary data
    pub fn write_bytes(uuid: Uuid, data: &[u8]) {
        unimplemented!();
    }

    pub fn load_bytes(uuid: Uuid) -> BytesHandle {
        unimplemented!();
    }
}

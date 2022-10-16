use nexdb::{Database, FileSystemDataSource, SchemaCacheSingleFile};
use std::path::PathBuf;

pub fn main() {
    let schema_cache_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/fs_data_source/schema_cache.json"
    ));
    let data_source_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/fs_data_source"
    ));

    let mut db = Database::default();

    let cache_str = std::fs::read_to_string(&schema_cache_path).unwrap();
    SchemaCacheSingleFile::load_string(&mut db, &cache_str);

    let fs = FileSystemDataSource::new(data_source_path, &mut db);

    println!("file_states {:#?}", fs.file_states());
    println!("object_locations {:#?}", fs.object_locations());
}

use nexdb::{FileSystemTreeDataSource, ObjectPath, SchemaCacheSingleFile, SchemaSet, UndoStack};
use std::path::PathBuf;
use std::sync::Arc;
use nexdb::edit_context::EditContext;

pub fn main() {
    let schema_cache_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/fs_data_source/schema_cache.json"
    ));
    let data_source_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/fs_data_source"
    ));

    let mut schema_set = SchemaSet::default();

    let cache_str = std::fs::read_to_string(&schema_cache_path).unwrap();
    SchemaCacheSingleFile::load_string(&mut schema_set, &cache_str);

    let undo_stack = UndoStack::default();
    let mut edit_context = EditContext::new(Arc::new(schema_set), &undo_stack);

    let fs = FileSystemTreeDataSource::new(data_source_path, ObjectPath::root(), &mut edit_context);

    println!("file_states {:#?}", fs.file_states());
    for (object_id, object_info) in edit_context.objects() {
        println!("{:?} {:?}", object_id, object_info.object_location());
        println!("  on disk: {:?}", fs.location_to_file_system_path(object_info.object_location()));
    }
}

mod file_system_id_based;

use std::path::Path;
pub use file_system_id_based::*;

use crate::edit_context::EditContext;
use crate::{AssetEngine, EditorModel, ObjectId};

mod file_system_path_based;
pub use file_system_path_based::*;
use crate::import_util::ImportToQueue;


trait SourceFileHandler {
    fn supported_file_extensions(&self) -> &[&'static str];

    fn generate_default_asset(
        &self,
        importable_name: Option<String>,
        edit_context: &EditContext
    ) -> Vec<ObjectId>;

    // importer also implements scan file
}

// impl SourceFileHandler {
//     fn handles_file(path: &Path) -> bool {
//
//     }
// }



pub trait DataSource {
    fn reload_all(
        &mut self,
        edit_context: &mut EditContext,
        imports_to_queue: &mut Vec<ImportToQueue>,
    );

    fn save_all_modified(
        &mut self,
        edit_context: &mut EditContext,
    );
    fn reload_all_modified(
        &mut self,
        edit_context: &mut EditContext,
    );

    // fn get_file_operations_required_to_save();
    //
    //
    //
    // fn save_objects(objects: &[ObjectId]);
}

mod file_system_id_based;
pub use file_system_id_based::*;

use crate::edit_context::EditContext;
use crate::{EditorModel, ObjectId};

mod file_system_path_based;
pub use file_system_path_based::*;

enum FileOperation {
    // add
    // delete
    // modify
}

pub trait DataSource {
    fn reload_all(
        &mut self,
        edit_context: &mut EditContext,
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

mod file_system_object;
pub use file_system_object::*;
use crate::{EditorModel, ObjectId};
use crate::edit_context::EditContext;

enum FileOperation {
    // add
    // delete
    // modify
}

pub trait DataSource {

    fn reload_all(&mut self, edit_context: &mut EditContext);

    fn save_all_modified(&mut self, edit_context: &mut EditContext);
    fn reload_all_modified(&mut self, edit_context: &mut EditContext);


    // fn get_file_operations_required_to_save();
    //
    //
    //
    // fn save_objects(objects: &[ObjectId]);
}

struct DummyDataSource {

}

impl DummyDataSource {
    fn new() {
        // Just create it, no file access here
    }

    fn load_all(editor_model: &mut EditorModel) {

    }

    fn load_some(editor_model: &mut EditorModel, objects: &[ObjectId]) {

    }

    fn save_all(editor_model: &mut EditorModel) {

    }

    fn save_some(editor_model: &mut EditorModel, objects: &[ObjectId]) {

    }

    // fn pending_vcs_locks() {
    //
    // }

    fn pending_vcs_operations() {

    }
}

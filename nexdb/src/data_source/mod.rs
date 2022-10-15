mod file_system;

use std::path::PathBuf;
use std::sync::Arc;
use serde_json::error::Category::Data;
use slotmap::DenseSlotMap;
use uuid::Uuid;
pub use file_system::*;
use crate::{Database, DataSet, HashMap, HashSet, ObjectId, SchemaSet, UndoStack};

slotmap::new_key_type! { pub struct EditContextKey; }


pub struct EditContext {
    database: Database,
    //dirty_files: HashSet<PathBuf>,
    referenced_files: HashSet<PathBuf>,
}

pub struct EditorModel {
    schema_set: Arc<SchemaSet>,
    undo_stack: UndoStack,
    root_context_key: EditContextKey,
    edit_contexts: DenseSlotMap<EditContextKey, EditContext>,
    //TODO: slot_map?
    data_sources: HashMap<PathBuf, FileSystemDataSource>
}

impl EditorModel {
    pub fn new(schema_set: Arc<SchemaSet>) -> Self {
        let undo_stack = UndoStack::default();
        let root_database = Database::new(schema_set.clone(), &undo_stack);
        let root_context = EditContext {
            database: root_database,
            //dirty_files: Default::default(),
            referenced_files: Default::default(),
        };
        let mut edit_contexts: DenseSlotMap<EditContextKey, EditContext> = Default::default();
        let root_context_key = edit_contexts.insert(root_context);

        EditorModel {
            schema_set,
            undo_stack,
            root_context_key,
            edit_contexts,
            data_sources: Default::default()
        }
    }

    fn is_object_modified(&self, object_id: ObjectId) -> bool {
        for (k, v) in self.edit_contexts {
            if v.database.is_object_modified(object_id) {
                return true;
            }
        }

        false
    }

    fn root_context(&self) -> &EditContext {
        self.edit_contexts.get(self.root_context_key).unwrap()
    }

    fn root_context_mut(&mut self) -> &mut EditContext {
        self.edit_contexts.get_mut(self.root_context_key).unwrap()
    }

    fn open_file_system_source<T: Into<PathBuf>>(&mut self, root_path: T) {
        let mut root_context = self.root_context_mut();
        let root_path = root_path.into();
        let fs = FileSystemDataSource::new(root_path.clone(), &mut root_context.database);
        self.data_sources.insert(root_path, fs);
    }

    fn close_file_system_source<T: Into<PathBuf>>(&mut self, root_path: T) {
        // kill edit contexts or fail

        // clear root_context of data from this source

        // drop the source
        let old = self.data_sources.remove(&root_path.into());
        assert!(old.is_some());
    }

    // We don't support new - expected that you create objects in root context and then open them
    fn open_edit_context(&mut self, objects: &[ObjectId]) {
        //let new_db = Database::new(self.schema_set.clone(), &self.undo_stack);
        let root_db = self.edit_contexts.get(self.root_context_key).unwrap();
        let mut data_set = DataSet::default();
        for object in objects {
            data_set.copy_from(root_db.database.data_set(), *object);
        }

        let new_db = Database::new_with_data(self.schema_set.clone(), &self.undo_stack);
    }

    fn save_edit_context() {

    }

    fn close_edit_context() {

    }

    fn undo(&mut self) {
        //self.undo_stack.undo()
    }

    fn redo() {

    }
}

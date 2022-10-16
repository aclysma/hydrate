use crate::edit_context::Database;
use crate::editor::undo::UndoStack;
use crate::{
    DataSet, FileSystemDataSource, HashMap, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId,
    SchemaSet,
};
use slotmap::DenseSlotMap;
use std::path::PathBuf;
use std::sync::Arc;
slotmap::new_key_type! { pub struct EditContextKey; }

// pub struct EditContext {
//     database: Database,
//     //dirty_files: HashSet<PathBuf>,
//     referenced_files: HashSet<PathBuf>,
// }
//
// impl EditContext {
//     // pub fn is_object_modified(&self, object_id: ObjectId) {
//     //     self.database.is_object_modified(object_id);
//     // }
//     //
//     // pub fn import_objects(&mut self, data_set: DataSet) {
//     //     self.database.import_objects(data_set);
//     // }
// }
//
// impl Deref for EditContext {
//     type Target = Database;
//
//     fn deref(&self) -> &Self::Target {
//         &self.database
//     }
// }
//
// impl DerefMut for EditContext {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.database
//     }
// }

pub struct EditorModel {
    schema_set: Arc<SchemaSet>,
    undo_stack: UndoStack,
    root_context_key: EditContextKey,
    edit_contexts: DenseSlotMap<EditContextKey, Database>,
    //TODO: slot_map?
    data_sources: HashMap<ObjectSourceId, FileSystemDataSource>,
}

impl EditorModel {
    pub fn new(schema_set: Arc<SchemaSet>) -> Self {
        let undo_stack = UndoStack::default();
        let root_edit_context = Database::new(schema_set.clone(), &undo_stack);
        let mut edit_contexts: DenseSlotMap<EditContextKey, Database> = Default::default();
        let root_context_key = edit_contexts.insert(root_edit_context);

        EditorModel {
            schema_set,
            undo_stack,
            root_context_key,
            edit_contexts,
            data_sources: Default::default(),
        }
    }

    pub fn schema_set(&self) -> &SchemaSet {
        &*self.schema_set
    }

    pub fn root_context(&self) -> &Database {
        self.edit_contexts.get(self.root_context_key).unwrap()
    }

    pub fn root_context_mut(&mut self) -> &mut Database {
        self.edit_contexts.get_mut(self.root_context_key).unwrap()
    }

    pub fn file_system_data_source(
        &mut self,
        object_source_id: ObjectSourceId,
    ) -> Option<&FileSystemDataSource> {
        self.data_sources.get(&object_source_id)
    }

    pub fn open_file_system_source<RootPathT: Into<PathBuf>>(
        &mut self,
        root_path: RootPathT,
        mount_path: ObjectPath,
    ) -> ObjectSourceId {
        let root_context = self.root_context_mut();
        let root_path = root_path.into();
        println!("MOUNT PATH {:?}", mount_path);
        let fs = FileSystemDataSource::new(root_path.clone(), mount_path, root_context);
        let object_source_id = fs.object_source_id();
        self.data_sources.insert(object_source_id, fs);
        object_source_id
    }

    // pub fn save_file_system_source<T: Into<PathBuf>>(&mut self, root_path: T) {
    //     // write to edited files
    //     for object
    //
    //     // delete files that shouldn't exist anymore
    // }

    pub fn save_root_context(&mut self) {
        let mut objects_by_location: HashMap<ObjectLocation, Vec<ObjectId>> = HashMap::default();

        let database = self.root_context();
        for (id, object) in database.objects() {
            //nexdb::
            objects_by_location
                .entry(object.object_location.clone())
                .or_default()
                .push(*id);
        }
        println!("found objects to save {:#?}", objects_by_location);

        for (location, object_ids) in objects_by_location {
            let data =
                crate::data_storage::DataStorageJsonSingleFile::store_string(database, &object_ids);
            let source = self.data_sources.get(&location.source()).unwrap();
            let file_path = source.location_to_file_system_path(&location).unwrap();
            //TODO: mark as written?

            println!("STORE DATA {:?} -> {:?}\n{}", location, file_path, data);
            std::fs::write(file_path, data).unwrap();
        }

        //crate::data_storage::DataStorageJsonSingleFile::store_string()
    }

    pub fn close_file_system_source(
        &mut self,
        object_source_id: ObjectSourceId,
    ) {
        // kill edit contexts or fail

        // clear root_context of data from this source

        // drop the source
        let old = self.data_sources.remove(&object_source_id);
        assert!(old.is_some());
    }

    // We don't support new - expected that you create objects in root context and then open them
    pub fn open_edit_context(
        &mut self,
        objects: &[ObjectId],
    ) {
        //let new_db = Database::new(self.schema_set.clone(), &self.undo_stack);
        let root_db = self.edit_contexts.get(self.root_context_key).unwrap();
        let mut data_set = DataSet::default();
        for object in objects {
            data_set.copy_from(root_db.data_set(), *object);
        }

        let new_db = Database::new_with_data(self.schema_set.clone(), &self.undo_stack);
    }

    // pub fn save_edit_context(&mut self) {
    //
    // }
    //
    // pub fn close_edit_context(&mut self) {
    //
    // }

    pub fn undo(&mut self) {
        //self.undo_stack.undo()
    }

    pub fn redo(&mut self) {}
}

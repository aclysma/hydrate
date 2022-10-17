use crate::edit_context::EditContext;
use crate::editor::undo::UndoStack;
use crate::{DataSet, FileSystemDataSource, HashMap, HashSet, LocationTree, LocationTreeNode, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId, SchemaSet};
use slotmap::DenseSlotMap;
use std::path::PathBuf;
use std::sync::Arc;
slotmap::new_key_type! { pub struct EditContextKey; }

pub struct EditorModel {
    schema_set: Arc<SchemaSet>,
    undo_stack: UndoStack,
    root_edit_context_key: EditContextKey,
    edit_contexts: DenseSlotMap<EditContextKey, EditContext>,
    //TODO: slot_map?
    data_sources: HashMap<ObjectSourceId, FileSystemDataSource>,
    location_tree: LocationTree,
}

impl EditorModel {
    pub fn new(schema_set: Arc<SchemaSet>) -> Self {
        let undo_stack = UndoStack::default();
        let mut edit_contexts: DenseSlotMap<EditContextKey, EditContext> = Default::default();

        let root_edit_context_key = edit_contexts.insert_with_key(|key| {
            EditContext::new(key, schema_set.clone(), &undo_stack)
        });

        EditorModel {
            schema_set,
            undo_stack,
            root_edit_context_key,
            edit_contexts,
            data_sources: Default::default(),
            location_tree: Default::default(),
        }
    }

    pub fn schema_set(&self) -> &SchemaSet {
        &*self.schema_set
    }

    pub fn root_edit_context(&self) -> &EditContext {
        self.edit_contexts.get(self.root_edit_context_key).unwrap()
    }

    pub fn root_edit_context_mut(&mut self) -> &mut EditContext {
        self.edit_contexts.get_mut(self.root_edit_context_key).unwrap()
    }

    pub fn file_system_data_source(
        &mut self,
        object_source_id: ObjectSourceId,
    ) -> Option<&FileSystemDataSource> {
        self.data_sources.get(&object_source_id)
    }

    pub fn add_file_system_source<RootPathT: Into<PathBuf>>(
        &mut self,
        root_path: RootPathT,
        mount_path: ObjectPath,
    ) -> ObjectSourceId {
        let root_edit_context = self.root_edit_context_mut();
        let root_path = root_path.into();
        println!("MOUNT PATH {:?}", mount_path);

        root_edit_context.commit_pending_undo_context();
        let mut loaded_objects = HashSet::default();
        let fs = FileSystemDataSource::new(root_path.clone(), mount_path, root_edit_context, &mut loaded_objects);
        for loaded_object in loaded_objects {
            root_edit_context.modified_objects.remove(&loaded_object);
        }

        let object_source_id = fs.object_source_id();
        self.data_sources.insert(object_source_id, fs);
        object_source_id
    }

    fn find_unsaved_paths(&self) -> HashSet<ObjectPath> {
        let root_edit_context = self.edit_contexts.get(self.root_edit_context_key).unwrap();
        let modified_objects =  &root_edit_context.modified_objects;

        let mut dirty_locations: HashSet<ObjectPath> = Default::default();

        for modified_object in modified_objects {
            let object_info = root_edit_context.objects().get(modified_object).unwrap();
            let location = object_info.object_location().clone();
            dirty_locations.insert(location.path().clone());
        }

        dirty_locations
    }

    pub fn save_root_edit_context(&mut self) {
        //
        // Ensure pending edits are flushed to the data set so that our modified objects list is fully up to date
        //
        let mut root_edit_context = self.edit_contexts.get_mut(self.root_edit_context_key).unwrap();
        root_edit_context.commit_pending_undo_context();

        //
        // Take the contents of the modified object list, leaving the edit context with a cleared list
        //
        let mut modified_objects = HashSet::default();
        std::mem::swap(&mut modified_objects, &mut root_edit_context.modified_objects);

        //
        // Build a list of locations (i.e. files) we need to save
        //
        let mut locations_to_save = HashMap::default();
        for modified_object in modified_objects {
            let object_info = root_edit_context.objects().get(&modified_object).unwrap();
            let location = object_info.object_location().clone();
            locations_to_save.insert(location, Vec::default());
        }

        //
        // Build a list of object IDs that need to be included in those files
        //
        for (object_id, object_info) in root_edit_context.objects() {
            let location = object_info.object_location();
            if let Some(objects) = locations_to_save.get_mut(location) {
                objects.push(*object_id);
            }
        }

        //
        // Write the files to disk, including all objects that should be present in them
        //
        for (location, object_ids) in locations_to_save {
            let data =
                crate::data_storage::DataStorageJsonSingleFile::store_string(root_edit_context, &object_ids);
            let source = self.data_sources.get(&location.source()).unwrap();
            let file_path = source.location_to_file_system_path(&location).unwrap();
            std::fs::write(file_path, data).unwrap();
        }
    }

    pub fn revert_root_edit_context(&mut self) {
        //
        // Ensure pending edits are cleared
        //
        let mut root_edit_context = self.edit_contexts.get_mut(self.root_edit_context_key).unwrap();
        root_edit_context.cancel_pending_undo_context();

        //
        // Take the contents of the modified object list, leaving the edit context with a cleared list
        //
        let mut modified_objects = HashSet::default();
        std::mem::swap(&mut modified_objects, &mut root_edit_context.modified_objects);

        //
        // Build a list of locations (i.e. files) we need to revert
        //
        let mut locations_to_revert = HashMap::default();
        for modified_object in modified_objects {
            let object_info = root_edit_context.objects().get(&modified_object).unwrap();
            let location = object_info.object_location().clone();
            locations_to_revert.insert(location, Vec::default());
        }

        //
        // Build a list of object IDs that need to be reverted (including deleting new objects
        //
        for (object_id, object_info) in root_edit_context.objects() {
            let location = object_info.object_location();
            if let Some(objects) = locations_to_revert.get_mut(location) {
                objects.push(*object_id);
            }
        }

        //
        // Reload the files from disk, deleting objects that are not present in them
        //
        for (location, object_ids) in locations_to_revert {
            let source = self.data_sources.get(&location.source()).unwrap();
            let file_path = source.location_to_file_system_path(&location).unwrap();
            let data = std::fs::read_to_string(file_path).unwrap();

            for object in object_ids {
                root_edit_context.delete_object(object);
            }

            crate::data_storage::DataStorageJsonSingleFile::load_string(root_edit_context, location, &data);

            //let source = self.data_sources.get(&location.source()).unwrap();
            //std::fs::write(file_path, data).unwrap();
        }

        //
        // Clear modified objects list since we reloaded everything from disk
        //
        //root_edit_context.cancel_pending_undo_context();
        root_edit_context.modified_objects.clear();
        root_edit_context.cancel_pending_undo_context();
        self.refresh_location_tree();
    }

    pub fn close_file_system_source(
        &mut self,
        object_source_id: ObjectSourceId,
    ) {
        unimplemented!();
        // kill edit contexts or fail

        // clear root_edit_context of data from this source

        // drop the source
        let old = self.data_sources.remove(&object_source_id);
        assert!(old.is_some());
    }

    // Spawns a separate edit context with copies of the given objects. The undo stack will be shared
    // globally, but changes will not be visible on the root context. The edit context will be flushed
    // to the root context in a single operation. Generally, we don't expect objects opened in a
    // separate edit context to change in the root context, but there is nothing that prevents it.
    pub fn open_edit_context(
        &mut self,
        objects: &[ObjectId],
    ) -> EditContextKey {
        let new_edit_context_key = self.edit_contexts.insert_with_key(|key| {
            EditContext::new_with_data(key, self.schema_set.clone(), &self.undo_stack)
        });

        let [root_edit_context, new_edit_context] = self.edit_contexts.get_disjoint_mut([self.root_edit_context_key, new_edit_context_key]).unwrap();

        for &object_id in objects {
            new_edit_context.data_set.copy_from(root_edit_context.data_set(), object_id);
        }

        new_edit_context_key
    }

    pub fn flush_edit_context_to_root(&mut self, edit_context: EditContextKey) {
        assert_ne!(edit_context, self.root_edit_context_key);
        let [root_context, context_to_flush] = self.edit_contexts.get_disjoint_mut([self.root_edit_context_key, edit_context]).unwrap();

        for &object_id in &context_to_flush.modified_objects {
            root_context.data_set.copy_from(&mut context_to_flush.data_set, object_id);
        }

        context_to_flush.modified_objects.clear();
    }

    pub fn close_edit_context(&mut self, edit_context: EditContextKey) {
        assert_ne!(edit_context, self.root_edit_context_key);
        self.edit_contexts.remove(edit_context);
    }

    pub fn undo(&mut self) {
        self.undo_stack.undo(&mut self.edit_contexts)
    }

    pub fn redo(&mut self) {
        self.undo_stack.redo(&mut self.edit_contexts)
    }

    pub fn refresh_location_tree(&mut self) {
        let mut all_paths = HashSet::default();
        for (object_id, info) in &self.edit_contexts.get(self.root_edit_context_key).unwrap().data_set.objects {
            let path = info.object_location.path();
            all_paths.insert(path.clone());
        }

        let unsaved_paths = self.find_unsaved_paths();
        self.location_tree.rebuild(&all_paths, &unsaved_paths);
    }

    pub fn cached_location_tree(&self) -> &LocationTree {
        &self.location_tree
    }
}

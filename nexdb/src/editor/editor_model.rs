use crate::edit_context::EditContext;
use crate::editor::undo::UndoStack;
use crate::{
    DataSet, DataSource, FileSystemObjectDataSource, HashMap, HashSet, LocationTree, ObjectId,
    ObjectLocation, ObjectPath, ObjectSourceId, PathNode, SchemaNamedType, SchemaSet,
};
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
    data_sources: HashMap<ObjectSourceId, Box<dyn DataSource>>,

    path_node_id_to_path: HashMap<ObjectId, ObjectPath>,
    //path_to_object_id: HashMap<ObjectPath, ObjectId>,
    location_tree: LocationTree,
}

impl EditorModel {
    pub fn new(schema_set: Arc<SchemaSet>) -> Self {
        let undo_stack = UndoStack::default();
        let mut edit_contexts: DenseSlotMap<EditContextKey, EditContext> = Default::default();

        let root_edit_context_key = edit_contexts
            .insert_with_key(|key| EditContext::new(key, schema_set.clone(), &undo_stack));

        EditorModel {
            schema_set,
            undo_stack,
            root_edit_context_key,
            edit_contexts,
            data_sources: Default::default(),
            location_tree: Default::default(),
            path_node_id_to_path: Default::default(),
        }
    }

    pub fn commit_all_pending_undo_contexts(&mut self) {
        for (_, context) in &mut self.edit_contexts {
            context.commit_pending_undo_context();
        }
    }

    pub fn cancel_all_pending_undo_contexts(&mut self) {
        for (_, context) in &mut self.edit_contexts {
            context.commit_pending_undo_context();
        }
    }

    pub fn any_edit_context_has_unsaved_changes(&self) -> bool {
        for (key, context) in &self.edit_contexts {
            if context.has_changes() {
                return true;
            }
        }

        false
    }

    pub fn schema_set(&self) -> &SchemaSet {
        &*self.schema_set
    }

    pub fn clone_schema_set(&self) -> Arc<SchemaSet> {
        self.schema_set.clone()
    }

    pub fn root_edit_context(&self) -> &EditContext {
        self.edit_contexts.get(self.root_edit_context_key).unwrap()
    }

    pub fn root_edit_context_mut(&mut self) -> &mut EditContext {
        self.edit_contexts
            .get_mut(self.root_edit_context_key)
            .unwrap()
    }

    pub fn path_node_id_to_path(
        &self,
        object_id: ObjectId,
    ) -> Option<&ObjectPath> {
        self.path_node_id_to_path.get(&object_id)
    }

    pub fn object_display_name_long(
        &self,
        object_id: ObjectId,
    ) -> String {
        let root_data_set = &self.root_edit_context().data_set;
        let location = root_data_set.object_location(object_id);
        let path = location
            .map(|x| self.path_node_id_to_path(x.path_node_id()))
            .flatten();

        if let Some(path) = path {
            let name = root_data_set.object_name(object_id);
            if let Some(name) = name.as_string() {
                path.join(name).as_str().to_string()
            } else {
                path.join(&format!("{}", object_id.as_uuid()))
                    .as_str()
                    .to_string()
            }
        } else {
            object_id.as_uuid().to_string()
        }
    }

    pub fn data_source(
        &mut self,
        object_source_id: ObjectSourceId,
    ) -> Option<&dyn DataSource> {
        self.data_sources.get(&object_source_id).map(|x| &**x)
    }

    pub fn add_file_system_object_source<RootPathT: Into<PathBuf>>(
        &mut self,
        root_path: RootPathT,
    ) -> ObjectSourceId {
        let root_edit_context = self.root_edit_context_mut();
        let root_path = root_path.into();

        root_edit_context.commit_pending_undo_context();
        let mut fs = FileSystemObjectDataSource::new(root_path.clone(), root_edit_context);
        fs.reload_all(root_edit_context);
        let object_source_id = fs.object_source_id();
        self.data_sources.insert(object_source_id, Box::new(fs));

        object_source_id
    }

    pub fn save_root_edit_context(&mut self) {
        //
        // Ensure pending edits are flushed to the data set so that our modified objects list is fully up to date
        //
        let mut root_edit_context = self
            .edit_contexts
            .get_mut(self.root_edit_context_key)
            .unwrap();
        root_edit_context.commit_pending_undo_context();

        for (id, data_source) in &mut self.data_sources {
            data_source.save_all_modified(root_edit_context);
        }

        //
        // Clear modified objects list since we saved everything to disk
        //
        root_edit_context.clear_change_tracking();
    }

    pub fn revert_root_edit_context(&mut self) {
        //
        // Ensure pending edits are cleared
        //
        let mut root_edit_context = self
            .edit_contexts
            .get_mut(self.root_edit_context_key)
            .unwrap();
        root_edit_context.cancel_pending_undo_context();

        //
        // Take the contents of the modified object list, leaving the edit context with a cleared list
        //
        let (modified_objects, modified_locations) =
            root_edit_context.take_modified_objects_and_locations();
        println!(
            "Revert:\nObjects: {:?}\nLocations: {:?}",
            modified_objects, modified_locations
        );

        for (id, data_source) in &mut self.data_sources {
            data_source.reload_all_modified(root_edit_context);
        }

        //
        // Clear modified objects list since we reloaded everything from disk.
        //
        root_edit_context.clear_change_tracking();
        //root_edit_context.cancel_pending_undo_context();

        println!("stuff");
        //self.refresh_object_path_lookups();
        //self.refresh_location_tree();
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

        let [root_edit_context, new_edit_context] = self
            .edit_contexts
            .get_disjoint_mut([self.root_edit_context_key, new_edit_context_key])
            .unwrap();

        for &object_id in objects {
            new_edit_context
                .data_set
                .copy_from(root_edit_context.data_set(), object_id);
        }

        new_edit_context_key
    }

    pub fn flush_edit_context_to_root(
        &mut self,
        edit_context: EditContextKey,
    ) {
        assert_ne!(edit_context, self.root_edit_context_key);
        let [root_context, context_to_flush] = self
            .edit_contexts
            .get_disjoint_mut([self.root_edit_context_key, edit_context])
            .unwrap();

        for &object_id in context_to_flush.modified_objects() {
            root_context
                .data_set
                .copy_from(&context_to_flush.data_set, object_id);
        }

        context_to_flush.clear_change_tracking();
    }

    pub fn close_edit_context(
        &mut self,
        edit_context: EditContextKey,
    ) {
        assert_ne!(edit_context, self.root_edit_context_key);
        self.edit_contexts.remove(edit_context);
    }

    pub fn undo(&mut self) {
        self.undo_stack.undo(&mut self.edit_contexts)
    }

    pub fn redo(&mut self) {
        self.undo_stack.redo(&mut self.edit_contexts)
    }

    fn do_populate_path(
        data_set: &DataSet,
        path_stack: &mut HashSet<ObjectId>,
        paths: &mut HashMap<ObjectId, ObjectPath>,
        path_node: ObjectId,
    ) -> ObjectPath {
        // If we already know the path for the tree node, just return it
        if let Some(parent_path) = paths.get(&path_node) {
            return parent_path.clone();
        }

        // To detect cyclical references, we accumulate visited objects into a set
        let is_cyclical_reference = !path_stack.insert(path_node);
        let source_id_and_path = if is_cyclical_reference {
            // If we detect a cycle, bail and return root path
            ObjectPath::root()
        } else {
            if let Some(object) = data_set.objects.get(&path_node) {
                if let Some(name) = object.object_name().as_string() {
                    // Parent is found, named, and not a cyclical reference
                    let mut parent = Self::do_populate_path(
                        data_set,
                        path_stack,
                        paths,
                        object.object_location.path_node_id(),
                    );
                    let path = parent.join(name);
                    path
                } else {
                    // Parent is unnamed, just treat as being at root path
                    ObjectPath::root()
                }
            } else {
                // Can't find parent, just treat as being at root path
                ObjectPath::root()
            }
        };

        paths.insert(path_node, source_id_and_path.clone());

        if !is_cyclical_reference {
            path_stack.remove(&path_node);
        }

        source_id_and_path
    }

    fn populate_paths(
        data_set: &DataSet,
        path_node_type: &SchemaNamedType,
    ) -> HashMap<ObjectId, ObjectPath> {
        let mut path_stack = HashSet::default();
        let mut paths = HashMap::<ObjectId, ObjectPath>::default();
        for (object_id, info) in &data_set.objects {
            // For objects that *are* path nodes, use their ID directly. For objects that aren't
            // path nodes, use the object ID in their location
            let path_node_id = if info.schema.fingerprint() == path_node_type.fingerprint() {
                *object_id
            } else {
                info.object_location.path_node_id()
            };

            Self::do_populate_path(data_set, &mut path_stack, &mut paths, path_node_id);
        }

        paths
    }

    pub fn refresh_tree_node_cache(&mut self) {
        let path_node_type = self
            .schema_set
            .find_named_type(PathNode::schema_name())
            .unwrap();
        let root_edit_context = self.edit_contexts.get(self.root_edit_context_key).unwrap();
        let path_node_id_to_path =
            Self::populate_paths(&root_edit_context.data_set, &path_node_type);
        self.path_node_id_to_path = path_node_id_to_path;
        self.location_tree =
            LocationTree::build(&root_edit_context.data_set, &self.path_node_id_to_path);
    }

    pub fn cached_location_tree(&self) -> &LocationTree {
        &self.location_tree
    }
}

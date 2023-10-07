use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use uuid::Uuid;

use crate::editor::undo::{CompletedUndoContextMessage, UndoContext, UndoStack};
use crate::{BuildInfo, DataObjectInfo, DataSet, DataSetDiff, DataSetResult, EditContextKey, EndContextBehavior, HashMap, HashMapKeys, HashSet, HashSetIter, ImportInfo, NullOverride, ObjectId, ObjectLocation, ObjectName, OverrideBehavior, SchemaFingerprint, SchemaNamedType, SchemaRecord, SchemaSet, Value};

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.

// Editor Context
// - Used to edit objects in isolation (for example, a node graph)
// - Expected that edited objects won't be modified by anything else
//   - Might get away with applying diffs, but may result in unintuitive behavior
// - Expected that non-edited objects *may* be modified, but not in a way that is incompatible with the edited objects
//   - Or, make a copy of non-edited objects
// - Maybe end-user needs to decide if they want to read new/old data?
//
// Undo Context
// - Used to demarcate changes that should conceptually be treated as a single operation
// - An undo context is labeled with a string. Within the context, multiple edits can be made
// - An undo context *may* be left in a "resumable" state. If we make modifications with an undo context
//   of the same name, we append to that undo context
// - If an edit is made with an undo context that doesn't match an existing "resumable" undo context,
//   we commit that context and carry on with the operation under a new undo context
// - A wrapper around dataset that understands undo contexts will produce a queue of finished undo
//   contexts, which contain revert/apply diffs
// - These undo contexts can be pushed onto a single global queue or a per-document queue

pub struct EditContext {
    edit_context_key: EditContextKey,
    schema_set: Arc<SchemaSet>,
    pub(super) data_set: DataSet,
    undo_context: UndoContext,
    completed_undo_context_tx: Sender<CompletedUndoContextMessage>,

    // We track locations separately because if an object is deleted, we don't know what location it
    // was stored at
    modified_objects: HashSet<ObjectId>,
    modified_locations: HashSet<ObjectLocation>,
}

impl EditContext {
    // Call after adding a new object
    fn track_new_object(
        &mut self,
        object_id: ObjectId,
        object_location: &ObjectLocation,
    ) {
        if self.undo_context.has_open_context() {
            // If an undo context is open, we use the diff for change tracking
            self.undo_context.track_new_object(object_id);
        } else {
            // If we don't have an undo context open, fast path to do change tracking ourselves
            self.modified_objects.insert(object_id);
            if !self.modified_locations.contains(&object_location) {
                self.modified_locations.insert(object_location.clone());
            }
        }
    }

    // Call before editing or deleting an object
    fn track_existing_object(
        &mut self,
        object_id: ObjectId,
    ) {
        if self.undo_context.has_open_context() {
            // If an undo is open, we use the diff for change tracking
            self.undo_context
                .track_existing_object(&mut self.data_set, object_id);
        } else {
            // If we don't have an undo context open, fast path to do change tracking ourselves
            self.modified_objects.insert(object_id);
            if let Some(object_info) = self.objects().get(&object_id) {
                if !self
                    .modified_locations
                    .contains(&object_info.object_location)
                {
                    self.modified_locations
                        .insert(object_info.object_location.clone());
                }
            }
        }
    }

    pub fn clear_change_tracking(&mut self) {
        self.modified_objects.clear();
        self.modified_locations.clear();
    }

    pub fn has_changes(&self) -> bool {
        !self.modified_objects.is_empty() || !self.modified_locations.is_empty()
    }

    pub fn is_object_modified(
        &self,
        object_id: ObjectId,
    ) -> bool {
        self.modified_objects.contains(&object_id)
    }

    pub fn modified_objects(&self) -> &HashSet<ObjectId> {
        &self.modified_objects
    }

    pub fn modified_locations(&self) -> &HashSet<ObjectLocation> {
        &self.modified_locations
    }

    pub fn clear_object_modified_flag(
        &mut self,
        object_id: ObjectId,
    ) {
        self.modified_objects.remove(&object_id);
    }

    pub fn clear_location_modified_flag(
        &mut self,
        object_location: &ObjectLocation,
    ) {
        self.modified_locations.remove(object_location);
    }

    pub fn take_modified_objects_and_locations(
        &mut self
    ) -> (HashSet<ObjectId>, HashSet<ObjectLocation>) {
        let mut modified_objects = HashSet::default();
        std::mem::swap(&mut modified_objects, &mut self.modified_objects);

        let mut modified_locations = HashSet::default();
        std::mem::swap(&mut modified_locations, &mut self.modified_locations);

        (modified_objects, modified_locations)
    }

    pub fn apply_diff(
        &mut self,
        diff: &DataSetDiff,
        modified_objects: &HashSet<ObjectId>,
        modified_locations: &HashSet<ObjectLocation>,
    ) {
        diff.apply(&mut self.data_set, &self.schema_set);

        // Marks the objects as changed
        self.modified_objects.extend(modified_objects);
        for modified_location in modified_locations {
            if !self.modified_locations.contains(modified_location) {
                self.modified_locations.insert(modified_location.clone());
            }
        }
    }

    pub fn new(
        edit_context_key: EditContextKey,
        schema_set: Arc<SchemaSet>,
        undo_stack: &UndoStack,
    ) -> Self {
        EditContext {
            edit_context_key,
            schema_set,
            data_set: Default::default(),
            undo_context: UndoContext::new(undo_stack, edit_context_key),
            completed_undo_context_tx: undo_stack.completed_undo_context_tx().clone(),
            modified_objects: Default::default(),
            modified_locations: Default::default(),
        }
    }

    pub fn new_with_data(
        edit_context_key: EditContextKey,
        schema_set: Arc<SchemaSet>,
        undo_stack: &UndoStack,
    ) -> Self {
        EditContext {
            edit_context_key,
            schema_set,
            data_set: Default::default(),
            undo_context: UndoContext::new(undo_stack, edit_context_key),
            completed_undo_context_tx: undo_stack.completed_undo_context_tx().clone(),
            modified_objects: Default::default(),
            modified_locations: Default::default(),
        }
    }

    pub fn with_undo_context<F: FnOnce(&mut Self) -> EndContextBehavior>(
        &mut self,
        name: &'static str,
        f: F,
    ) {
        self.undo_context.begin_context(
            &self.data_set,
            name,
            &mut self.modified_objects,
            &mut self.modified_locations,
        );
        let end_context_behavior = (f)(self);
        self.undo_context.end_context(
            &self.data_set,
            end_context_behavior,
            &mut self.modified_objects,
            &mut self.modified_locations,
        );
    }

    pub fn commit_pending_undo_context(&mut self) {
        self.undo_context.commit_context(
            &mut self.data_set,
            &mut self.modified_objects,
            &mut self.modified_locations,
        );
    }

    pub fn cancel_pending_undo_context(&mut self) {
        self.undo_context.cancel_context(&mut self.data_set);
    }

    // pub fn apply_diff(&mut self, diff: &DataSetDiff) {
    //     diff.apply(&mut self.data_set);
    // }

    //
    // Schema-related functions
    //
    pub fn schema_set(&self) -> &Arc<SchemaSet> {
        &self.schema_set
    }

    pub fn schemas(&self) -> &HashMap<SchemaFingerprint, SchemaNamedType> {
        &self.schema_set.schemas()
    }

    pub fn find_named_type(
        &self,
        name: impl AsRef<str>,
    ) -> Option<&SchemaNamedType> {
        self.schema_set.find_named_type(name)
    }

    pub fn find_named_type_by_fingerprint(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&SchemaNamedType> {
        self.schema_set.find_named_type_by_fingerprint(fingerprint)
    }
    //
    // pub fn default_value_for_schema(
    //     &self,
    //     schema: &Schema,
    // ) -> Value {
    //     self.schema_set.default_value_for_schema(schema)
    // }
    //
    // pub fn add_linked_types(
    //     &mut self,
    //     linker: SchemaLinker,
    // ) -> SchemaLinkerResult<()> {
    //     let mut schemas = (*self.schema_set).clone();
    //     schemas.add_linked_types(linker)?;
    //     self.schema_set = Arc::new(schemas);
    //     Ok(())
    // }
    //
    // pub(crate) fn restore_named_types(
    //     &mut self,
    //     named_types: Vec<SchemaNamedType>
    // ) {
    //     let mut schemas = (*self.schema_set).clone();
    //     schemas.restore_named_types(named_types);
    //     self.schema_set = Arc::new(schemas);
    // }

    //
    // Data-related functions
    //
    pub fn data_set(&self) -> &DataSet {
        &self.data_set
    }

    // pub fn data_set_mut(&mut self) -> &mut DataSet {
    //     &mut self.data_set
    // }

    // pub fn objects_with_locations(&self) -> HashMap<ObjectId, &ObjectLocation> {
    //     self.data_set.objects.iter().map(|(k, v)| {
    //         (k, &v.object_location)
    //     })
    // }
    //
    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DataObjectInfo> {
        self.data_set.all_objects()
    }

    pub fn objects(&self) -> &HashMap<ObjectId, DataObjectInfo> {
        self.data_set.objects()
    }

    pub fn has_object(
        &self,
        object_id: ObjectId,
    ) -> bool {
        self.objects().contains_key(&object_id)
    }

    // pub(crate) fn insert_object(
    //     &mut self,
    //     obj_info: DataObjectInfo,
    // ) -> ObjectId {
    //     let object_id = self.data_set.insert_object(obj_info);
    //     self.undo_context.track_new_object(object_id);
    //     object_id
    // }

    pub fn new_object(
        &mut self,
        object_name: &ObjectName,
        object_location: &ObjectLocation,
        schema: &SchemaRecord,
    ) -> ObjectId {
        let object_id =
            self.data_set
                .new_object(object_name.clone(), object_location.clone(), schema);
        self.track_new_object(object_id, object_location);
        object_id
    }

    pub fn new_object_from_prototype(
        &mut self,
        object_name: &ObjectName,
        object_location: &ObjectLocation,
        prototype: ObjectId,
    ) -> ObjectId {
        let object_id = self.data_set.new_object_from_prototype(
            object_name.clone(),
            object_location.clone(),
            prototype,
        );
        self.track_new_object(object_id, &object_location);
        object_id
    }

    pub fn restore_objects_from(
        &mut self,
        data_set: DataSet,
    ) {
        for (k, v) in data_set.objects {
            self.restore_object(
                k,
                v.object_name,
                v.object_location,
                v.import_info.clone(),
                v.build_info,
                v.prototype,
                v.schema.fingerprint(),
                v.properties,
                v.property_null_overrides,
                v.properties_in_replace_mode,
                v.dynamic_array_entries,
            );
        }
    }

    pub(crate) fn restore_object(
        &mut self,
        object_id: ObjectId,
        object_name: ObjectName,
        object_location: ObjectLocation,
        import_info: Option<ImportInfo>,
        build_info: BuildInfo,
        prototype: Option<ObjectId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
    ) {
        self.track_new_object(object_id, &object_location);
        self.data_set.restore_object(
            object_id,
            object_name,
            object_location,
            import_info,
            build_info,
            &self.schema_set,
            prototype,
            schema,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        );
    }

    pub fn delete_object(
        &mut self,
        object_id: ObjectId,
    ) {
        self.track_existing_object(object_id);
        self.data_set.delete_object(object_id);
    }

    pub fn set_object_location(
        &mut self,
        object_id: ObjectId,
        new_location: ObjectLocation,
    ) {
        self.track_existing_object(object_id);
        self.data_set.set_object_location(object_id, new_location);
        // Again so that we track the new location too
        self.track_existing_object(object_id);
    }

    pub fn set_import_info(
        &mut self,
        object_id: ObjectId,
        import_info: ImportInfo,
    ) {
        self.data_set.set_import_info(object_id, import_info);
    }

    pub fn object_name(
        &self,
        object_id: ObjectId,
    ) -> &ObjectName {
        self.data_set.object_name(object_id)
    }

    pub fn object_location(
        &self,
        object_id: ObjectId,
    ) -> Option<&ObjectLocation> {
        self.data_set.object_location(object_id)
    }

    pub fn import_info(
        &self,
        object_id: ObjectId,
    ) -> Option<&ImportInfo> {
        self.data_set.import_info(object_id)
    }

    pub fn resolve_all_file_references(
        &self,
        object_id: ObjectId,
    ) -> Option<HashMap<PathBuf, ObjectId>> {
        self.data_set.resolve_all_file_references(object_id)
    }

    pub fn get_all_file_reference_overrides(
        &mut self,
        object_id: ObjectId,
    ) -> Option<&HashMap<PathBuf, ObjectId>> {
        self.data_set.get_all_file_reference_overrides(object_id)
    }

    pub fn set_file_reference_override(
        &mut self,
        object_id: ObjectId,
        path: PathBuf,
        referenced_object_id: ObjectId,
    ) {
        self.track_existing_object(object_id);
        self.data_set
            .set_file_reference_override(object_id, path, referenced_object_id);
    }

    pub fn object_prototype(
        &self,
        object_id: ObjectId,
    ) -> Option<ObjectId> {
        self.data_set.object_prototype(object_id)
    }

    pub fn object_schema(
        &self,
        object_id: ObjectId,
    ) -> Option<&SchemaRecord> {
        self.data_set.object_schema(object_id)
    }

    pub fn get_null_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<NullOverride> {
        self.data_set
            .get_null_override(&self.schema_set, object_id, path)
    }

    pub fn set_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        self.track_existing_object(object_id);
        self.data_set
            .set_null_override(&self.schema_set, object_id, path, null_override)
    }

    pub fn remove_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        self.track_existing_object(object_id);
        self.data_set
            .remove_null_override(&self.schema_set, object_id, path)
    }

    pub fn resolve_is_null(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<bool> {
        self.data_set
            .resolve_is_null(&self.schema_set, object_id, path)
    }

    pub fn has_property_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> bool {
        self.data_set.has_property_override(object_id, path)
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        self.data_set.get_property_override(object_id, path)
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        value: Value,
    ) -> DataSetResult<()> {
        self.track_existing_object(object_id);
        self.data_set
            .set_property_override(&self.schema_set, object_id, path, value)
    }

    pub fn remove_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.track_existing_object(object_id);
        self.data_set.remove_property_override(object_id, path)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> DataSetResult<()> {
        self.track_existing_object(object_id);
        if let Some(prototype) = self.object_prototype(object_id) {
            self.track_existing_object(prototype);
        }

        self.data_set
            .apply_property_override_to_prototype(&self.schema_set, object_id, path)
    }

    pub fn resolve_property(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        self.data_set
            .resolve_property(&self.schema_set, object_id, path)
    }

    pub fn get_dynamic_array_overrides(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<HashSetIter<Uuid>> {
        self.data_set
            .get_dynamic_array_overrides(&self.schema_set, object_id, path)
    }

    pub fn add_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Uuid {
        self.track_existing_object(object_id);
        self.data_set
            .add_dynamic_array_override(&self.schema_set, object_id, path)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        self.track_existing_object(object_id);
        self.data_set
            .remove_dynamic_array_override(&self.schema_set, object_id, path, element_id)
    }

    pub fn resolve_dynamic_array(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Box<[Uuid]> {
        self.data_set
            .resolve_dynamic_array(&self.schema_set, object_id, path)
    }

    pub fn get_override_behavior(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> OverrideBehavior {
        self.data_set
            .get_override_behavior(&self.schema_set, object_id, path)
    }

    pub fn set_override_behavior(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) {
        self.track_existing_object(object_id);
        self.data_set
            .set_override_behavior(&self.schema_set, object_id, path, behavior)
    }
}

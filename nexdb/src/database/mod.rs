use std::collections::VecDeque;
use super::schema::*;
use super::{HashMap, HashSet, ObjectId, SchemaFingerprint};
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::{HashMapKeys, HashSetIter};

pub mod value;
pub use value::Value;

mod data_set;
pub use data_set::DataObjectInfo;
pub use data_set::OverrideBehavior;
pub use data_set::NullOverride;
pub use data_set::DataSet;
pub use data_set::ObjectPath;
pub use data_set::ObjectSourceId;
pub use data_set::ObjectLocation;

mod schema_set;
pub use schema_set::SchemaSet;


mod diff;
use diff::ObjectDiffSet;
use diff::DataSetDiff;
pub use diff::DataSetDiffSet;

#[cfg(test)]
mod tests;

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.




pub struct UndoStack {
    undo_chain: Vec<DataSetDiffSet>,
    competed_context_rx: Receiver<DataSetDiffSet>,
    competed_context_tx: Sender<DataSetDiffSet>,
}

impl Default for UndoStack {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        UndoStack {
            undo_chain: Default::default(),
            competed_context_tx: tx,
            competed_context_rx: rx
        }
    }
}

impl UndoStack {
    fn drain_rx(&mut self) {
        while let Ok(diff) = self.competed_context_rx.try_recv() {
            self.undo_chain.push(diff);
        }
    }

    pub fn undo(&mut self, db: &mut Database) {
        self.drain_rx();

        let popped = self.undo_chain.pop();
        if let Some(popped) = popped {
            popped.revert_diff.apply(&mut db.data_set);
        }
    }

    //TODO: Implement undo/redo properly
}



// Transaction that holds exclusive access for the data and will directly commit changes. It can
// compare directly against the original dataset for changes
pub struct UndoContext {
    before_state: DataSet,
    tracked_objects: HashSet<ObjectId>,
    context_name: Option<&'static str>,
    completed_context_tx: Sender<DataSetDiffSet>
}

impl UndoContext {
    fn new(undo_stack: &UndoStack) -> Self {
        UndoContext {
            before_state: Default::default(),
            tracked_objects: Default::default(),
            context_name: Default::default(),
            completed_context_tx: undo_stack.competed_context_tx.clone(),
        }
    }

    // Call after adding a new object
    fn track_new_object(&mut self, object_id: ObjectId) {
        if self.context_name.is_some() {
            self.tracked_objects.insert(object_id);
        }
    }

    // Call before editing or deleting an object
    fn track_existing_object(&mut self, after_state: &DataSet, object_id: ObjectId) {
        if self.context_name.is_some() {
            //TODO: Preserve sub-objects?
            if !self.tracked_objects.contains(&object_id) {
                println!("track object");
                self.tracked_objects.insert(object_id);
                self.before_state.copy_from(&after_state, object_id);
            }
        }
    }

    fn has_open_context(&self) -> bool {
        self.context_name.is_some()
    }


    fn begin_context(&mut self, after_state: &DataSet, name: &'static str, modified_objects: &mut HashSet<ObjectId>) {
        if self.context_name == Some(name) {
            // don't need to do anything, we can append to the current context
        } else {
            // commit the context that's in flight, if one exists
            if self.context_name.is_some() {
                // This won't do anything if there's nothing to send
                self.commit_context(after_state, modified_objects);
            }

            self.context_name = Some(name);
        }
    }

    fn end_context(&mut self, after_state: &DataSet, name: &'static str, allow_resume: bool, modified_objects: &mut HashSet<ObjectId>) {
        if !allow_resume {
            // This won't do anything if there's nothing to send
            self.commit_context(after_state, modified_objects);
        }
    }

    fn cancel_context(&mut self, after_state: &mut DataSet) {
        if !self.tracked_objects.is_empty() {
            // Overwrite all the objects in the new set with old data
            let mut objects = Default::default();
            std::mem::swap(&mut objects, &mut self.before_state.objects);
            for (object_id, object) in objects {
                after_state.objects.insert(object_id, object);
            }

            // Delete any tracked objects that aren't in the old data
            after_state.objects.retain(|k, v| self.tracked_objects.contains(k) && !self.before_state.objects.contains_key(k));

            self.before_state = Default::default();
            self.tracked_objects.clear();
        }
    }

    fn commit_context(&mut self, after_state: &DataSet, modified_objects: &mut HashSet<ObjectId>) {
        if !self.tracked_objects.is_empty() {
            // Make a diff and send it if it has changes
            let diff_set = DataSetDiffSet::diff_data_set(&self.before_state, &after_state, &self.tracked_objects);
            if diff_set.has_changes() {
                println!("Sending change {:#?}", diff_set);
                self.completed_context_tx.send(diff_set).unwrap();
            }

            for object in &self.tracked_objects {
                modified_objects.insert(*object);
            }

            self.before_state = Default::default();
            self.tracked_objects.clear();
        }
    }
}


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


//TODO: Rename to EditContext
pub struct Database {
    schema_set: Arc<SchemaSet>,
    data_set: DataSet,
    undo_context: UndoContext,
    completed_context_tx: Sender<DataSetDiffSet>,
    modified_objects: HashSet<ObjectId>,
}

impl Database {
    // Call after adding a new object
    fn track_new_object(&mut self, object_id: ObjectId) {
        if self.undo_context.has_open_context() {
            self.undo_context.track_new_object(object_id);
        } else {
            self.modified_objects.insert(object_id);
        }
    }

    // Call before editing or deleting an object
    fn track_existing_object(&mut self, object_id: ObjectId) {
        if self.undo_context.has_open_context() {
            self.undo_context.track_existing_object(&mut self.data_set, object_id);
        } else {
            self.modified_objects.insert(object_id);
        }
    }

    pub fn is_object_modified(&self, object_id: ObjectId) {
        self.modified_objects.contains(&object_id);
    }

    pub fn new(schema_set: Arc<SchemaSet>, undo_stack: &UndoStack) -> Self {
        Database {
            schema_set,
            data_set: Default::default(),
            undo_context: UndoContext::new(undo_stack),
            completed_context_tx: undo_stack.competed_context_tx.clone(),
            modified_objects: Default::default(),
        }
    }

    pub fn new_with_data(schema_set: Arc<SchemaSet>, undo_stack: &UndoStack) -> Self {
        Database {
            schema_set,
            data_set: Default::default(),
            undo_context: UndoContext::new(undo_stack),
            completed_context_tx: undo_stack.competed_context_tx.clone(),
            modified_objects: Default::default(),
        }
    }

    //TODO: Change to use an enum instead of bool
    pub fn with_undo_context<F: FnOnce(&mut Self) -> bool>(&mut self, name: &'static str, f: F) {
        self.undo_context.begin_context(&self.data_set, name, &mut self.modified_objects);
        let allow_resume = (f)(self);
        self.undo_context.end_context(&self.data_set, name, allow_resume, &mut self.modified_objects);
    }

    pub fn commit_pending_undo_context(&mut self) {
        self.undo_context.commit_context(&mut self.data_set, &mut self.modified_objects);
    }


    // pub fn apply_diff(&mut self, diff: &DataSetDiff) {
    //     diff.apply(&mut self.data_set);
    // }



    //
    // Schema-related functions
    //
    pub fn schema_set(&self) -> &SchemaSet {
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

    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DataObjectInfo> {
        self.data_set.all_objects()
    }

    pub(crate) fn objects(&self) -> &HashMap<ObjectId, DataObjectInfo> {
        self.data_set.objects()
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
        object_location: ObjectLocation,
        schema: &SchemaRecord,
    ) -> ObjectId {
        let object_id = self.data_set.new_object(object_location, schema);
        self.undo_context.track_new_object(object_id);
        object_id
    }

    pub fn new_object_from_prototype(
        &mut self,
        object_location: ObjectLocation,
        prototype: ObjectId,
    ) -> ObjectId {
        let object_id = self.data_set.new_object_from_prototype(object_location, prototype);
        self.undo_context.track_new_object(object_id);
        object_id
    }

    pub fn import_objects(&mut self, data_set: DataSet) {
        for (k, v) in data_set.objects {
            self.restore_object(
                k,
                v.object_location,
                v.prototype,
                v.schema.fingerprint(),
                v.properties,
                v.property_null_overrides,
                v.properties_in_replace_mode,
                v.dynamic_array_entries
            );
        }
    }

    pub(crate) fn restore_object(
        &mut self,
        object_id: ObjectId,
        object_location: ObjectLocation,
        prototype: Option<ObjectId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
    ) {
        self.data_set.restore_object(object_id, object_location, &self.schema_set, prototype, schema, properties, property_null_overrides, properties_in_replace_mode, dynamic_array_entries);
        self.undo_context.track_new_object(object_id);
    }

    pub fn object_prototype(
        &self,
        object_id: ObjectId
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
        self.data_set.get_null_override(&self.schema_set, object_id, path)
    }

    pub fn set_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.set_null_override(&self.schema_set, object_id, path, null_override)
    }

    pub fn remove_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.remove_null_override(&self.schema_set, object_id, path)
    }

    pub fn resolve_is_null(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<bool> {
        self.data_set.resolve_is_null(&self.schema_set, object_id, path)
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
    ) -> bool {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.set_property_override(&self.schema_set, object_id, path, value)
    }

    pub fn remove_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.remove_property_override(object_id, path)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.apply_property_override_to_prototype(&self.schema_set, object_id, path)
    }

    pub fn resolve_property(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.data_set.resolve_property(&self.schema_set, object_id, path)
    }

    pub fn get_dynamic_array_overrides(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<HashSetIter<Uuid>> {
        self.data_set.get_dynamic_array_overrides(&self.schema_set, object_id, path)
    }

    pub fn add_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Uuid {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.add_dynamic_array_override(&self.schema_set, object_id, path)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.remove_dynamic_array_override(&self.schema_set, object_id, path, element_id)
    }

    pub fn resolve_dynamic_array(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Box<[Uuid]> {
        self.data_set.resolve_dynamic_array(&self.schema_set, object_id, path)
    }

    pub fn get_override_behavior(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> OverrideBehavior {
        self.data_set.get_override_behavior(&self.schema_set, object_id, path)
    }

    pub fn set_override_behavior(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) {
        self.undo_context.track_existing_object(&self.data_set, object_id);
        self.data_set.set_override_behavior(&self.schema_set, object_id, path, behavior)
    }
}

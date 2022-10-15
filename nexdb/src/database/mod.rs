use super::schema::*;
use super::{HashMap, HashSet, ObjectId, SchemaFingerprint};
use std::io::BufRead;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::{BufferId, HashMapKeys, HashSetIter, SchemaLinker, SchemaLinkerResult};

pub mod value;
pub use value::Value;
use crate::database::schema_set::SchemaSet;

mod data_set;
pub use data_set::DataObjectInfo;
pub use data_set::OverrideBehavior;
pub use data_set::NullOverride;
pub use data_set::DataSet;

mod diff;
use diff::ObjectDiffSet;
pub use diff::DataSetDiffSet;



mod schema_set;

#[cfg(test)]
mod tests;

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that


// Transaction that holds exclusive access for the data and will directly commit changes. It can
// compare directly against the original dataset for changes
pub struct UndoContext {
    before_state: DataSet,
    tracked_objects: HashSet<ObjectId>,
}

impl UndoContext {
    fn track_new_object(&mut self, object_id: ObjectId) {
        self.tracked_objects.insert(object_id);
    }

    fn track_existing_object(&mut self, after_state: &DataSet, object_id: ObjectId) {
        //TODO: Preserve sub-objects?
        if !self.tracked_objects.contains(&object_id) {
            self.tracked_objects.insert(object_id);
            self.before_state.copy_from(&after_state, object_id);
        }
    }

    pub fn revert(self, after_state: &mut DataSet) {
        // Overwrite all the objects in the new set with our data
        for (object_id, object) in self.before_state.objects {
            after_state.objects.insert(object_id, object);
        }
    }

    pub fn commit(self, after_state: &DataSet) -> DataSetDiffSet {
        DataSetDiffSet::diff_data_set(&self.before_state, &after_state, &self.tracked_objects)
    }

    pub fn new_object(
        &mut self,
        schema: &SchemaRecord,
        after_state: &mut DataSet
    ) -> ObjectId {
        let object_id = after_state.new_object(schema);
        self.track_new_object(object_id);
        object_id
    }

    pub fn new_object_from_prototype(
        &mut self,
        after_state: &mut DataSet,
        prototype: ObjectId,
    ) -> ObjectId {
        let object_id = after_state.new_object_from_prototype(prototype);
        self.tracked_objects.insert(object_id);
        object_id
    }

    pub fn delete_object(
        &mut self,
        after_state: &mut DataSet,
        object_id: ObjectId
    ) {
        //TODO: Deleting object may requires more objects to be touched to remove references to it
        self.preserve_object(after_state, object_id);
        after_state.delete_object(object_id);
    }

    pub(crate) fn restore_object(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        prototype: Option<ObjectId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
    ) {
        self.preserve_object(after_state, object_id);
        after_state.restore_object(schema_set, object_id, prototype, schema, properties, property_null_overrides, properties_in_replace_mode, dynamic_array_entries);
    }

    pub fn set_null_override(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        self.preserve_object(after_state, object_id);
        after_state.set_null_override(schema_set, object_id, path, null_override)
    }

    pub fn remove_null_override(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        self.preserve_object(after_state, object_id);
        after_state.remove_null_override(&schema_set, object_id, path)
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        value: Value,
    ) -> bool {
        self.preserve_object(after_state, object_id);
        let schema_set = self.schema_set.clone();
        after_state.set_property_override(&schema_set, object_id, path, value)
    }

    pub fn remove_property_override(
        &mut self,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.preserve_object(after_state, object_id);
        after_state.remove_property_override(object_id, path)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        self.preserve_object(after_state, object_id);
        after_state.apply_property_override_to_prototype(schema_set, object_id, path)
    }

    pub fn add_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Uuid {
        self.preserve_object(after_state, object_id);
        after_state.add_dynamic_array_override(schema_set, object_id, path)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        self.preserve_object(after_state, object_id);
        after_state.remove_dynamic_array_override(schema_set, object_id, path, element_id)
    }

    pub fn set_override_behavior(
        &mut self,
        schema_set: &SchemaSet,
        after_state: &mut DataSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) {
        self.preserve_object(after_state, object_id);
        after_state.set_override_behavior(schema_set, object_id, path, behavior)
    }
}















// pub struct UndoContext {
//     before_state: DataSet,
//     tracked_objects: HashSet<ObjectId>,
// }
//







#[derive(Default)]
pub struct Database {
    schema_set: Arc<SchemaSet>,
    data_set: DataSet,
}

impl Database {
    // pub fn create_immediate_transaction(&self) -> ImmediateTransaction {
    //     ImmediateTransaction {
    //         schema_set: self.schema_set.clone(),
    //         before: &self.data_set,
    //         after: Default::default(),
    //         tracked_objects: Default::default()
    //     }
    // }

    pub fn begin_transaction() {

    }

    pub fn revert_transaction() {
        unimplemented!();
    }

    pub fn commit_transaction() -> DataSetDiffSet {
        unimplemented!();
    }

    pub fn apply_diff() {

    }




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

    pub fn default_value_for_schema(
        &self,
        schema: &Schema,
    ) -> Value {
        self.schema_set.default_value_for_schema(schema)
    }

    pub fn add_linked_types(
        &mut self,
        linker: SchemaLinker,
    ) -> SchemaLinkerResult<()> {
        let mut schemas = (*self.schema_set).clone();
        schemas.add_linked_types(linker)?;
        self.schema_set = Arc::new(schemas);
        Ok(())
    }

    pub(crate) fn restore_named_types(
        &mut self,
        named_types: Vec<SchemaNamedType>
    ) {
        let mut schemas = (*self.schema_set).clone();
        schemas.restore_named_types(named_types);
        self.schema_set = Arc::new(schemas);
    }

    //
    // Data-related functions
    //
    pub fn data_set(&self) -> &DataSet {
        &self.data_set
    }

    pub fn data_set_mut(&mut self) -> &mut DataSet {
        &mut self.data_set
    }

    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DataObjectInfo> {
        self.data_set.all_objects()
    }

    pub(crate) fn objects(&self) -> &HashMap<ObjectId, DataObjectInfo> {
        self.data_set.objects()
    }

    pub(crate) fn insert_object(
        &mut self,
        obj_info: DataObjectInfo,
    ) -> ObjectId {
        self.data_set.insert_object(obj_info)
    }

    pub fn new_object(
        &mut self,
        schema: &SchemaRecord,
    ) -> ObjectId {
        self.data_set.new_object(schema)
    }

    pub fn new_object_from_prototype(
        &mut self,
        prototype: ObjectId,
    ) -> ObjectId {
        self.data_set.new_object_from_prototype(prototype)
    }

    pub(crate) fn restore_object(
        &mut self,
        object_id: ObjectId,
        prototype: Option<ObjectId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
    ) {
        self.data_set.restore_object(&self.schema_set, object_id, prototype, schema, properties, property_null_overrides, properties_in_replace_mode, dynamic_array_entries)
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
        self.data_set.set_null_override(&self.schema_set, object_id, path, null_override)
    }

    pub fn remove_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
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
        self.data_set.set_property_override(&self.schema_set, object_id, path, value)
    }

    pub fn remove_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.data_set.remove_property_override(object_id, path)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
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
        self.data_set.add_dynamic_array_override(&self.schema_set, object_id, path)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
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
        self.data_set.set_override_behavior(&self.schema_set, object_id, path, behavior)
    }
}

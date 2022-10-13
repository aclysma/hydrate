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
use crate::database::data_set::{DataSet, ObjectDiff, ObjectDiffSet};

mod schema_set;

#[cfg(test)]
mod tests;

//TODO: Delete unused property data when path ancestor is null or in replace mode

pub struct TransactionDiffs {

}

pub struct Transaction {
    schema_set: Arc<SchemaSet>,
    before: DataSet,
    after: DataSet,
}

impl Transaction {
    pub fn add_object(&mut self, baseline: &DataSet) {

    }

    pub fn create_diffs() {

    }
}

// Delta
// Added
// Removed


//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

#[derive(Default, Debug)]
pub struct TransactionDiff {
    creates: Vec<DataObjectInfo>,
    deletes: Vec<ObjectId>,
    changes: HashMap<ObjectId, ObjectDiff>
}

impl TransactionDiff {
    pub fn has_changes(&self) -> bool {
        !self.creates.is_empty() || !self.deletes.is_empty() || !self.changes.is_empty()
    }

    pub fn apply(&self, data_set: &mut DataSet) {
        for create in &self.creates {
            data_set.insert_object(create.clone());
        }

        for delete in &self.deletes {
            data_set.delete_object(*delete);
        }

        for (object_id, v) in &self.changes {
            if let Some(object) = data_set.objects_mut().get_mut(object_id) {
                v.apply(object);
            }
        }
    }
}


#[derive(Debug)]
pub struct TransactionDiffSet {
    pub apply_diff: TransactionDiff,
    pub revert_diff: TransactionDiff,
}

impl TransactionDiffSet {
    pub fn has_changes(&self) -> bool {
        // assume if apply has no changes, neither does revert
        self.apply_diff.has_changes()
    }

    pub fn diff_data_set(before: &DataSet, after: &DataSet, tracked_objects: &HashSet<ObjectId>) -> Self {
        let mut apply_diff = TransactionDiff::default();
        let mut revert_diff = TransactionDiff::default();

        // Check for created objects
        for &object_id in tracked_objects {
            let existed_before = before.objects().contains_key(&object_id);
            let existed_after = after.objects().contains_key(&object_id);
            if existed_before {
                if existed_after {
                    // changed
                    let diff = ObjectDiffSet::diff_objects(before, object_id,  &after, object_id);
                    if diff.has_changes() {
                        apply_diff.changes.insert(object_id, diff.apply_diff);
                        revert_diff.changes.insert(object_id, diff.revert_diff);
                    }
                } else {
                    // deleted
                    apply_diff.deletes.push(object_id);
                    revert_diff.creates.push(before.objects().get(&object_id).unwrap().clone());
                }
            } else if existed_after {
                // created
                apply_diff.creates.push(after.objects().get(&object_id).unwrap().clone());
                revert_diff.deletes.push(object_id);
            }
        }

        TransactionDiffSet {
            apply_diff,
            revert_diff
        }
    }
}

pub struct DeferredTransaction {
    schema_set: Arc<SchemaSet>,
    after: DataSet,
    tracked_objects: HashSet<ObjectId>,
}

impl DeferredTransaction {
    pub fn resume(self, data_set: &DataSet) -> ImmediateTransaction {
        ImmediateTransaction {
            schema_set: self.schema_set,
            before: data_set,
            after: self.after,
            tracked_objects: self.tracked_objects
        }
    }
}


// Transaction that holds exclusive access for the data and will directly commit changes. It can
// compare directly against the original dataset for changes
pub struct ImmediateTransaction<'a> {
    schema_set: Arc<SchemaSet>,
    before: &'a DataSet,
    after: DataSet,
    tracked_objects: HashSet<ObjectId>,
}

impl<'a> ImmediateTransaction<'a> {
    pub fn defer(self) -> DeferredTransaction {
        DeferredTransaction {
            schema_set: self.schema_set,
            after: self.after,
            tracked_objects: self.tracked_objects
        }
    }

    pub fn create_diff_set(&self) -> TransactionDiffSet {
        TransactionDiffSet::diff_data_set(self.before, &self.after, &self.tracked_objects)
        // let mut apply_diff = TransactionDiff::default();
        // let mut revert_diff = TransactionDiff::default();
        //
        // // Check for created objects
        // for &object_id in &self.tracked_objects {
        //     let existed_before = self.before.objects().contains_key(&object_id);
        //     let existed_after = self.after.objects().contains_key(&object_id);
        //     if existed_before {
        //         if existed_after {
        //             // changed
        //             let diff = ObjectDiffSet::diff_objects(self.before, object_id,  &self.after, object_id);
        //             if diff.has_changes() {
        //                 apply_diff.changes.insert(object_id, diff.apply_diff);
        //                 revert_diff.changes.insert(object_id, diff.revert_diff);
        //             }
        //         } else {
        //             // deleted
        //             apply_diff.deletes.push(object_id);
        //             revert_diff.creates.push(self.before.objects().get(&object_id).unwrap().clone());
        //         }
        //     } else if existed_after {
        //         // created
        //         apply_diff.creates.push(self.after.objects().get(&object_id).unwrap().clone());
        //         revert_diff.deletes.push(object_id);
        //     }
        // }
        //
        // TransactionDiffSet {
        //     apply_diff,
        //     revert_diff
        // }
    }

    fn data_set_for_read(&self, object_id: ObjectId) -> &DataSet {
        if !self.tracked_objects.contains(&object_id) {
            &self.before
        } else {
            &self.after
        }
    }

    fn data_set_for_write(&mut self, object_id: ObjectId) -> &mut DataSet {
        if !self.tracked_objects.contains(&object_id) {
            self.after.copy_from(self.before, object_id);
            self.tracked_objects.insert(object_id);
        }

        if let Some(prototype) = self.after.objects().get(&object_id).unwrap().prototype {
            // add the prototype
            self.data_set_for_write(prototype);
        }

        &mut self.after
    }











    //
    // Schema-related functions
    //
    pub(crate) fn schema_set(&self) -> &SchemaSet {
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







    pub fn new_object(
        &mut self,
        schema: &SchemaRecord,
    ) -> ObjectId {
        let object_id = self.after.new_object(schema);
        self.tracked_objects.insert(object_id);
        object_id
    }

    pub fn new_object_from_prototype(
        &mut self,
        prototype: ObjectId,
    ) -> ObjectId {
        let object_id = self.after.new_object_from_prototype(prototype);
        self.tracked_objects.insert(object_id);
        object_id
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
        self.after.restore_object(&self.schema_set, object_id, prototype, schema, properties, property_null_overrides, properties_in_replace_mode, dynamic_array_entries);
        self.tracked_objects.insert(object_id);
    }

    pub fn object_prototype(
        &self,
        object_id: ObjectId
    ) -> Option<ObjectId> {
        self.data_set_for_read(object_id).object_prototype(object_id)
    }

    pub fn object_schema(
        &self,
        object_id: ObjectId,
    ) -> Option<&SchemaRecord> {
        self.data_set_for_read(object_id).object_schema(object_id)
    }

    pub fn get_null_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<NullOverride> {
        self.data_set_for_read(object_id).get_null_override(&self.schema_set, object_id, path)
    }

    pub fn set_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).set_null_override(&schema_set, object_id, path, null_override)
    }

    pub fn remove_null_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).remove_null_override(&schema_set, object_id, path)
    }

    pub fn resolve_is_null(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<bool> {
        self.data_set_for_read(object_id).resolve_is_null(&self.schema_set, object_id, path)
    }

    pub fn has_property_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> bool {
        self.data_set_for_read(object_id).has_property_override(object_id, path)
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        self.data_set_for_read(object_id).get_property_override(object_id, path)
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        value: Value,
    ) -> bool {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).set_property_override(&schema_set, object_id, path, value)
    }

    pub fn remove_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.data_set_for_write(object_id).remove_property_override(object_id, path)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).apply_property_override_to_prototype(&schema_set, object_id, path)
    }

    pub fn resolve_property(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.data_set_for_read(object_id).resolve_property(&self.schema_set, object_id, path)
    }

    pub fn get_dynamic_array_overrides(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<HashSetIter<Uuid>> {
        self.data_set_for_read(object_id).get_dynamic_array_overrides(&self.schema_set, object_id, path)
    }

    pub fn add_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Uuid {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).add_dynamic_array_override(&schema_set, object_id, path)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).remove_dynamic_array_override(&schema_set, object_id, path, element_id)
    }

    pub fn resolve_dynamic_array(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Box<[Uuid]> {
        self.data_set_for_read(object_id).resolve_dynamic_array(&self.schema_set, object_id, path)
    }

    pub fn get_override_behavior(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> OverrideBehavior {
        self.data_set_for_read(object_id).get_override_behavior(&self.schema_set, object_id, path)
    }

    pub fn set_override_behavior(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) {
        let schema_set = self.schema_set.clone();
        self.data_set_for_write(object_id).set_override_behavior(&schema_set, object_id, path, behavior)
    }
}






















#[derive(Default)]
pub struct Database {
    schema_set: Arc<SchemaSet>,
    data_set: DataSet,
}

impl Database {
    pub fn create_immediate_transaction(&self) -> ImmediateTransaction {
        ImmediateTransaction {
            schema_set: self.schema_set.clone(),
            before: &self.data_set,
            after: Default::default(),
            tracked_objects: Default::default()
        }
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

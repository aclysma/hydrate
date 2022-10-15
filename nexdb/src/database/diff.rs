use uuid::Uuid;
use crate::{DataObjectInfo, DataSet, HashMap, HashSet, NullOverride, ObjectId, Value};
use crate::value::PropertyValue;

#[derive(Debug)]
pub struct DynamicArrayEntryDelta {
    key: String,
    add: Vec<Uuid>,
    remove: Vec<Uuid>
}

#[derive(Default, Debug)]
pub struct ObjectDiff {
    set_prototype: Option<Option<ObjectId>>,
    set_properties: Vec<(String, PropertyValue)>,
    remove_properties: Vec<String>,
    set_null_overrides: Vec<(String, NullOverride)>,
    remove_null_overrides: Vec<String>,
    add_properties_in_replace_mode: Vec<String>,
    remove_properties_in_replace_mode: Vec<String>,
    dynamic_array_entry_deltas: Vec<DynamicArrayEntryDelta>,
}

impl ObjectDiff {
    pub fn has_changes(&self) -> bool {
        self.set_prototype.is_some() ||
            !self.set_properties.is_empty() ||
            !self.remove_properties.is_empty() ||
            !self.set_null_overrides.is_empty() ||
            !self.remove_null_overrides.is_empty() ||
            !self.add_properties_in_replace_mode.is_empty() ||
            !self.remove_properties_in_replace_mode.is_empty() ||
            !self.dynamic_array_entry_deltas.is_empty()
    }

    pub fn apply(&self, object: &mut DataObjectInfo) {
        if let Some(set_prototype) = self.set_prototype {
            object.prototype = set_prototype;
        }

        for (k, v) in &self.set_properties {
            object.properties.insert(k.clone(), v.as_value());
        }

        for k in &self.remove_properties {
            object.properties.remove(k);
        }

        for (k, v) in &self.set_null_overrides {
            object.property_null_overrides.insert(k.clone(), *v);
        }

        for k in &self.remove_properties {
            object.property_null_overrides.remove(k);
        }

        for k in &self.add_properties_in_replace_mode {
            object.properties_in_replace_mode.insert(k.clone());
        }

        for k in &self.remove_properties_in_replace_mode {
            object.properties_in_replace_mode.remove(k);
        }

        for delta in &self.dynamic_array_entry_deltas {
            if !delta.add.is_empty() {
                //
                // Path where we add keys: We may need to create the entry in the map. Won't need to remove it
                //
                let existing_entries = if let Some(existing_entries) = object.dynamic_array_entries.get_mut(&delta.key) {
                    existing_entries
                } else {
                    object.dynamic_array_entries.entry(delta.key.clone()).or_default()
                };

                for k in &delta.add {
                    existing_entries.insert(*k);
                }

                for k in &delta.remove {
                    existing_entries.remove(k);
                }
            } else if !delta.remove.is_empty() {
                //
                // Path where we don't add keys but we remove keys: We may need to delete the entry in the map. Won't need to add it
                //
                if let Some(existing_entries) = object.dynamic_array_entries.get_mut(&delta.key) {
                    for k in &delta.remove {
                        existing_entries.remove(k);
                    }

                    if existing_entries.is_empty() {
                        object.dynamic_array_entries.remove(&delta.key);
                    }
                }
            }
        }
    }
}

pub struct ObjectDiffSet {
    pub apply_diff: ObjectDiff,
    pub revert_diff: ObjectDiff,
}

impl ObjectDiffSet {
    pub fn has_changes(&self) -> bool {
        // assume if apply has no changes, neither does revert
        self.apply_diff.has_changes()
    }

    pub fn diff_objects(
        before_data_set: &DataSet,
        before_object_id: ObjectId,
        after_data_set: &DataSet,
        after_object_id: ObjectId,
    ) -> Self {
        let before_obj = before_data_set.objects.get(&before_object_id).unwrap();
        let after_obj = after_data_set.objects.get(&after_object_id).unwrap();

        assert_eq!(before_obj.schema.fingerprint(), after_obj.schema.fingerprint());

        let mut apply_diff = ObjectDiff::default();
        let mut revert_diff = ObjectDiff::default();

        //
        // Prototype
        //
        if before_obj.prototype != after_obj.prototype {
            apply_diff.set_prototype = Some(after_obj.prototype);
            revert_diff.set_prototype = Some(before_obj.prototype);
        }

        //
        // Properties
        //
        for (key, before_value) in &before_obj.properties {
            if let Some(after_value) = after_obj.properties.get(key) {
                if !Value::are_matching_property_values(before_value, after_value) {
                    // Value was changed
                    apply_diff.set_properties.push((key.clone(), after_value.as_property_value().unwrap()));
                    revert_diff.set_properties.push((key.clone(), before_value.as_property_value().unwrap()));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_properties.push(key.clone());
                revert_diff.set_properties.push((key.clone(), before_value.as_property_value().unwrap()));
            }
        }

        for (key, after_value) in &after_obj.properties {
            if !before_obj.properties.contains_key(key) {
                // Property was added
                apply_diff.set_properties.push((key.clone(), after_value.as_property_value().unwrap()));
                revert_diff.remove_properties.push(key.clone());
            }
        }


        //
        // Null Overrides
        //
        for (key, &before_value) in &before_obj.property_null_overrides {
            if let Some(after_value) = after_obj.property_null_overrides.get(key).copied() {
                if before_value != after_value {
                    // Value was changed
                    apply_diff.set_null_overrides.push((key.clone(), after_value));
                    revert_diff.set_null_overrides.push((key.clone(), before_value));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_null_overrides.push(key.clone());
                revert_diff.set_null_overrides.push((key.clone(), before_value));
            }
        }

        for (key, &after_value) in &after_obj.property_null_overrides {
            if !before_obj.property_null_overrides.contains_key(key) {
                // Property was added
                apply_diff.set_null_overrides.push((key.clone(), after_value));
                revert_diff.remove_null_overrides.push(key.clone());
            }
        }

        //
        // Properties in replace mode
        //
        for replace_mode_property in &before_obj.properties_in_replace_mode {
            if !after_obj.properties_in_replace_mode.contains(replace_mode_property) {
                // Replace mode disabled
                apply_diff.remove_properties_in_replace_mode.push(replace_mode_property.clone());
                revert_diff.add_properties_in_replace_mode.push(replace_mode_property.clone());
            }
        }

        for replace_mode_property in &after_obj.properties_in_replace_mode {
            if !before_obj.properties_in_replace_mode.contains(replace_mode_property) {
                // Replace mode enabled
                apply_diff.add_properties_in_replace_mode.push(replace_mode_property.clone());
                revert_diff.remove_properties_in_replace_mode.push(replace_mode_property.clone());
            }
        }

        //
        // Dynamic Array Entries
        //
        for (key, old_entries) in &before_obj.dynamic_array_entries {
            if let Some(new_entries) = after_obj.dynamic_array_entries.get(key) {
                // Diff the hashes
                let mut added_entries = Vec::default();
                let mut removed_entries = Vec::default();

                for old_entry in old_entries {
                    if !new_entries.contains(&old_entry) {
                        removed_entries.push(*old_entry);
                    }
                }

                for new_entry in new_entries {
                    if !old_entries.contains(&new_entry) {
                        added_entries.push(*new_entry);
                    }
                }

                if !added_entries.is_empty() || !removed_entries.is_empty() {
                    apply_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: added_entries.clone(),
                        remove: removed_entries.clone(),
                    });
                    revert_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: removed_entries,
                        remove: added_entries,
                    });
                }
            } else {
                if !old_entries.is_empty() {
                    // All of them were removed
                    apply_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: Default::default(),
                        remove: old_entries.iter().copied().collect(),
                    });
                    revert_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: old_entries.iter().copied().collect(),
                        remove: Default::default(),
                    });
                }
            }
        }

        for (key, new_entries) in &after_obj.dynamic_array_entries {
            if !new_entries.is_empty() {
                if !before_obj.dynamic_array_entries.contains_key(key) {
                    // All of them were added
                    apply_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: new_entries.iter().copied().collect(),
                        remove: Default::default(),
                    });
                    revert_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: Default::default(),
                        remove: new_entries.iter().copied().collect(),
                    });
                }
            }
        }

        ObjectDiffSet {
            apply_diff,
            revert_diff,
        }
    }
}

#[derive(Default, Debug)]
pub struct DataSetDiff {
    creates: Vec<DataObjectInfo>,
    deletes: Vec<ObjectId>,
    changes: HashMap<ObjectId, ObjectDiff>
}

impl DataSetDiff {
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
pub struct DataSetDiffSet {
    pub apply_diff: DataSetDiff,
    pub revert_diff: DataSetDiff,
}

impl DataSetDiffSet {
    pub fn has_changes(&self) -> bool {
        // assume if apply has no changes, neither does revert
        self.apply_diff.has_changes()
    }

    pub fn diff_data_set(before: &DataSet, after: &DataSet, tracked_objects: &HashSet<ObjectId>) -> Self {
        let mut apply_diff = DataSetDiff::default();
        let mut revert_diff = DataSetDiff::default();

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

        DataSetDiffSet {
            apply_diff,
            revert_diff
        }
    }
}
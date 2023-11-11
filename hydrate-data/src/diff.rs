use crate::value::PropertyValue;
use crate::{
    DataAssetInfo, DataSet, HashSet, NullOverride, AssetId, AssetLocation, AssetName,
    SchemaSet, Value,
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
pub struct DynamicArrayEntryDelta {
    key: String,
    add: Vec<Uuid>,
    remove: Vec<Uuid>,
}

#[derive(Default, Debug)]
pub struct AssetDiff {
    set_name: Option<AssetName>,
    set_location: Option<AssetLocation>,
    set_prototype: Option<Option<AssetId>>,
    set_properties: Vec<(String, PropertyValue)>,
    remove_properties: Vec<String>,
    set_null_overrides: Vec<(String, NullOverride)>,
    remove_null_overrides: Vec<String>,
    add_properties_in_replace_mode: Vec<String>,
    remove_properties_in_replace_mode: Vec<String>,
    dynamic_array_entry_deltas: Vec<DynamicArrayEntryDelta>,
    set_file_references: Vec<(PathBuf, AssetId)>,
    remove_file_references: Vec<PathBuf>,
}

impl AssetDiff {
    pub fn has_changes(&self) -> bool {
        self.set_name.is_some()
            || self.set_location.is_some()
            || self.set_prototype.is_some()
            || !self.set_properties.is_empty()
            || !self.remove_properties.is_empty()
            || !self.set_null_overrides.is_empty()
            || !self.remove_null_overrides.is_empty()
            || !self.add_properties_in_replace_mode.is_empty()
            || !self.remove_properties_in_replace_mode.is_empty()
            || !self.dynamic_array_entry_deltas.is_empty()
            || !self.set_file_references.is_empty()
            || !self.remove_file_references.is_empty()
    }

    pub fn apply(
        &self,
        asset: &mut DataAssetInfo,
    ) {
        if let Some(set_name) = &self.set_name {
            asset.asset_name = set_name.clone();
        }

        if let Some(set_location) = &self.set_location {
            asset.asset_location = set_location.clone();
        }

        if let Some(set_prototype) = self.set_prototype {
            asset.prototype = set_prototype;
        }

        for (k, v) in &self.set_properties {
            asset.properties.insert(k.clone(), v.as_value());
        }

        for k in &self.remove_properties {
            asset.properties.remove(k);
        }

        for (k, v) in &self.set_null_overrides {
            asset.property_null_overrides.insert(k.clone(), *v);
        }

        for k in &self.remove_properties {
            asset.property_null_overrides.remove(k);
        }

        for k in &self.add_properties_in_replace_mode {
            asset.properties_in_replace_mode.insert(k.clone());
        }

        for k in &self.remove_properties_in_replace_mode {
            asset.properties_in_replace_mode.remove(k);
        }

        for delta in &self.dynamic_array_entry_deltas {
            if !delta.add.is_empty() {
                //
                // Path where we add keys: We may need to create the entry in the map. Won't need to remove it
                //
                let existing_entries = if let Some(existing_entries) =
                    asset.dynamic_array_entries.get_mut(&delta.key)
                {
                    existing_entries
                } else {
                    asset
                        .dynamic_array_entries
                        .entry(delta.key.clone())
                        .or_default()
                };

                for k in &delta.add {
                    existing_entries.try_insert_at_end(*k);
                }

                for k in &delta.remove {
                    existing_entries.remove(k);
                }
            } else if !delta.remove.is_empty() {
                //
                // Path where we don't add keys but we remove keys: We may need to delete the entry in the map. Won't need to add it
                //
                if let Some(existing_entries) = asset.dynamic_array_entries.get_mut(&delta.key) {
                    for k in &delta.remove {
                        existing_entries.remove(k);
                    }

                    if existing_entries.is_empty() {
                        asset.dynamic_array_entries.remove(&delta.key);
                    }
                }
            }
        }

        for (k, v) in &self.set_file_references {
            asset
                .build_info
                .file_reference_overrides
                .insert(k.clone(), *v);
        }

        for k in &self.remove_file_references {
            asset.build_info.file_reference_overrides.remove(k);
        }
    }
}

pub struct AssetDiffSet {
    pub apply_diff: AssetDiff,
    pub revert_diff: AssetDiff,
}

impl AssetDiffSet {
    pub fn has_changes(&self) -> bool {
        // assume if apply has no changes, neither does revert
        self.apply_diff.has_changes()
    }

    pub fn diff_assets(
        before_data_set: &DataSet,
        before_asset_id: AssetId,
        after_data_set: &DataSet,
        after_asset_id: AssetId,
        modified_locations: &mut HashSet<AssetLocation>,
    ) -> Self {
        let before_obj = before_data_set.assets().get(&before_asset_id).unwrap();
        let after_obj = after_data_set.assets().get(&after_asset_id).unwrap();

        assert_eq!(
            before_obj.schema().fingerprint(),
            after_obj.schema().fingerprint()
        );

        let mut apply_diff = AssetDiff::default();
        let mut revert_diff = AssetDiff::default();

        if before_obj.asset_name != after_obj.asset_name {
            apply_diff.set_name = Some(after_obj.asset_name.clone());
            revert_diff.set_name = Some(before_obj.asset_name.clone());
        }

        if before_obj.asset_location != after_obj.asset_location {
            apply_diff.set_location = Some(after_obj.asset_location.clone());
            revert_diff.set_location = Some(before_obj.asset_location.clone());
        }

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
                    apply_diff
                        .set_properties
                        .push((key.clone(), after_value.as_property_value().unwrap()));
                    revert_diff
                        .set_properties
                        .push((key.clone(), before_value.as_property_value().unwrap()));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_properties.push(key.clone());
                revert_diff
                    .set_properties
                    .push((key.clone(), before_value.as_property_value().unwrap()));
            }
        }

        for (key, after_value) in &after_obj.properties {
            if !before_obj.properties.contains_key(key) {
                // Property was added
                apply_diff
                    .set_properties
                    .push((key.clone(), after_value.as_property_value().unwrap()));
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
                    apply_diff
                        .set_null_overrides
                        .push((key.clone(), after_value));
                    revert_diff
                        .set_null_overrides
                        .push((key.clone(), before_value));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_null_overrides.push(key.clone());
                revert_diff
                    .set_null_overrides
                    .push((key.clone(), before_value));
            }
        }

        for (key, &after_value) in &after_obj.property_null_overrides {
            if !before_obj.property_null_overrides.contains_key(key) {
                // Property was added
                apply_diff
                    .set_null_overrides
                    .push((key.clone(), after_value));
                revert_diff.remove_null_overrides.push(key.clone());
            }
        }

        //
        // Properties in replace mode
        //
        for replace_mode_property in &before_obj.properties_in_replace_mode {
            if !after_obj
                .properties_in_replace_mode
                .contains(replace_mode_property)
            {
                // Replace mode disabled
                apply_diff
                    .remove_properties_in_replace_mode
                    .push(replace_mode_property.clone());
                revert_diff
                    .add_properties_in_replace_mode
                    .push(replace_mode_property.clone());
            }
        }

        for replace_mode_property in &after_obj.properties_in_replace_mode {
            if !before_obj
                .properties_in_replace_mode
                .contains(replace_mode_property)
            {
                // Replace mode enabled
                apply_diff
                    .add_properties_in_replace_mode
                    .push(replace_mode_property.clone());
                revert_diff
                    .remove_properties_in_replace_mode
                    .push(replace_mode_property.clone());
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
                    apply_diff
                        .dynamic_array_entry_deltas
                        .push(DynamicArrayEntryDelta {
                            key: key.clone(),
                            add: added_entries.clone(),
                            remove: removed_entries.clone(),
                        });
                    revert_diff
                        .dynamic_array_entry_deltas
                        .push(DynamicArrayEntryDelta {
                            key: key.clone(),
                            add: removed_entries,
                            remove: added_entries,
                        });
                }
            } else {
                if !old_entries.is_empty() {
                    // All of them were removed
                    apply_diff
                        .dynamic_array_entry_deltas
                        .push(DynamicArrayEntryDelta {
                            key: key.clone(),
                            add: Default::default(),
                            remove: old_entries.iter().copied().collect(),
                        });
                    revert_diff
                        .dynamic_array_entry_deltas
                        .push(DynamicArrayEntryDelta {
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
                    apply_diff
                        .dynamic_array_entry_deltas
                        .push(DynamicArrayEntryDelta {
                            key: key.clone(),
                            add: new_entries.iter().copied().collect(),
                            remove: Default::default(),
                        });
                    revert_diff
                        .dynamic_array_entry_deltas
                        .push(DynamicArrayEntryDelta {
                            key: key.clone(),
                            add: Default::default(),
                            remove: new_entries.iter().copied().collect(),
                        });
                }
            }
        }

        //
        // File References
        //
        for (key, &before_value) in &before_obj.build_info.file_reference_overrides {
            if let Some(&after_value) = after_obj.build_info.file_reference_overrides.get(key) {
                if before_value != after_value {
                    // Value was changed
                    apply_diff
                        .set_file_references
                        .push((key.clone(), after_value));
                    revert_diff
                        .set_file_references
                        .push((key.clone(), before_value));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_file_references.push(key.clone());
                revert_diff
                    .set_file_references
                    .push((key.clone(), before_value));
            }
        }

        for (key, &after_value) in &after_obj.build_info.file_reference_overrides {
            if !before_obj
                .build_info
                .file_reference_overrides
                .contains_key(key)
            {
                // Property was added
                apply_diff
                    .set_file_references
                    .push((key.clone(), after_value));
                revert_diff.remove_file_references.push(key.clone());
            }
        }

        // we only flag the location as modified if we make an edit
        // (if apply_diff doesn't have changes, before_diff doesn't either)
        if apply_diff.has_changes() {
            if !modified_locations.contains(&after_obj.asset_location) {
                modified_locations.insert(after_obj.asset_location.clone());
            }

            // Also save the old location so that in moves, the "from" location is marked as changed too
            if before_obj.asset_location != after_obj.asset_location {
                if !modified_locations.contains(&before_obj.asset_location) {
                    modified_locations.insert(before_obj.asset_location.clone());
                }
            }
        }

        AssetDiffSet {
            apply_diff,
            revert_diff,
        }
    }
}

#[derive(Default, Debug)]
pub struct DataSetDiff {
    creates: Vec<(AssetId, DataAssetInfo)>,
    deletes: Vec<AssetId>,
    changes: Vec<(AssetId, AssetDiff)>,
}

impl DataSetDiff {
    pub fn has_changes(&self) -> bool {
        !self.creates.is_empty() || !self.deletes.is_empty() || !self.changes.is_empty()
    }

    pub fn apply(
        &self,
        data_set: &mut DataSet,
        schema_set: &SchemaSet,
    ) {
        for delete in &self.deletes {
            data_set.delete_asset(*delete);
        }

        for (id, create) in &self.creates {
            data_set.restore_asset(
                *id,
                create.asset_name.clone(),
                create.asset_location.clone(),
                create.import_info.clone(),
                create.build_info.clone(),
                schema_set,
                create.prototype,
                create.schema().fingerprint(),
                create.properties.clone(),
                create.property_null_overrides.clone(),
                create.properties_in_replace_mode.clone(),
                create.dynamic_array_entries.clone(),
            );
        }

        for (asset_id, v) in &self.changes {
            if let Some(asset) = data_set.assets_mut().get_mut(asset_id) {
                v.apply(asset);
            }
        }
    }

    pub fn get_modified_assets(
        &self,
        modified_assets: &mut HashSet<AssetId>,
    ) {
        for (id, _) in &self.creates {
            modified_assets.insert(*id);
        }

        for id in &self.deletes {
            modified_assets.insert(*id);
        }

        for (id, _) in &self.changes {
            modified_assets.insert(*id);
        }
    }
}

#[derive(Debug)]
pub struct DataSetDiffSet {
    pub apply_diff: DataSetDiff,
    pub revert_diff: DataSetDiff,
    pub modified_assets: HashSet<AssetId>,
    pub modified_locations: HashSet<AssetLocation>,
}

impl DataSetDiffSet {
    pub fn has_changes(&self) -> bool {
        // assume if apply has no changes, neither does revert
        self.apply_diff.has_changes()
    }

    pub fn diff_data_set(
        before: &DataSet,
        after: &DataSet,
        tracked_assets: &HashSet<AssetId>,
    ) -> Self {
        let mut apply_diff = DataSetDiff::default();
        let mut revert_diff = DataSetDiff::default();
        let mut modified_assets: HashSet<AssetId> = Default::default();
        let mut modified_locations: HashSet<AssetLocation> = Default::default();

        // Check for created assets
        for &asset_id in tracked_assets {
            let existed_before = before.assets().contains_key(&asset_id);
            let existed_after = after.assets().contains_key(&asset_id);
            if existed_before {
                if existed_after {
                    // Asset was modified
                    let diff = AssetDiffSet::diff_assets(
                        before,
                        asset_id,
                        &after,
                        asset_id,
                        &mut modified_locations,
                    );
                    if diff.has_changes() {
                        modified_assets.insert(asset_id);
                        apply_diff.changes.push((asset_id, diff.apply_diff));
                        revert_diff.changes.push((asset_id, diff.revert_diff));
                    }
                } else {
                    // Asset was deleted
                    let before_asset_info = before.assets().get(&asset_id).unwrap().clone();
                    modified_assets.insert(asset_id);
                    if !modified_locations.contains(&before_asset_info.asset_location) {
                        modified_locations.insert(before_asset_info.asset_location.clone());
                    }

                    // deleted
                    apply_diff.deletes.push(asset_id);
                    revert_diff
                        .creates
                        .push((asset_id, before_asset_info.clone()));
                }
            } else if existed_after {
                // Asset was created
                let after_asset_info = after.assets().get(&asset_id).unwrap();
                if !modified_locations.contains(&after_asset_info.asset_location) {
                    modified_locations.insert(after_asset_info.asset_location.clone());
                }

                // created
                apply_diff
                    .creates
                    .push((asset_id, after_asset_info.clone()));
                revert_diff.deletes.push(asset_id);
            }
        }

        DataSetDiffSet {
            apply_diff,
            revert_diff,
            modified_locations,
            modified_assets,
        }
    }
}

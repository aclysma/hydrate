use hydrate_data::{OrderedSet, SingleObject};
use std::path::PathBuf;
use uuid::Uuid;

use crate::editor::undo::{UndoContext, UndoStack};
use crate::{
    BuildInfo, DataSetAssetInfo, DataSet, DataSetDiff, DataSetResult, EditContextKey,
    EndContextBehavior, HashMap, HashSet, ImportInfo, NullOverride,
    AssetId, AssetLocation, AssetName, OverrideBehavior, SchemaFingerprint, SchemaNamedType,
    SchemaRecord, SchemaSet, Value,
};

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.

// Editor Context
// - Used to edit assets in isolation (for example, a node graph)
// - Expected that edited assets won't be modified by anything else
//   - Might get away with applying diffs, but may result in unintuitive behavior
// - Expected that non-edited assets *may* be modified, but not in a way that is incompatible with the edited assets
//   - Or, make a copy of non-edited assets
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
    schema_set: SchemaSet,
    pub(super) data_set: DataSet,
    undo_context: UndoContext,

    // We track locations separately because if an asset is deleted, we don't know what location it
    // was stored at
    modified_assets: HashSet<AssetId>,
    modified_locations: HashSet<AssetLocation>,
}

impl EditContext {
    // Call after adding a new asset
    fn track_new_asset(
        &mut self,
        asset_id: AssetId,
        asset_location: &AssetLocation,
    ) {
        if self.undo_context.has_open_context() {
            // If an undo context is open, we use the diff for change tracking
            self.undo_context.track_new_asset(asset_id);
        } else {
            // If we don't have an undo context open, fast path to do change tracking ourselves
            self.modified_assets.insert(asset_id);
            if !self.modified_locations.contains(&asset_location) {
                self.modified_locations.insert(asset_location.clone());
            }
        }
    }

    // Call before editing or deleting an asset
    fn track_existing_asset(
        &mut self,
        asset_id: AssetId,
    ) -> DataSetResult<()> {
        if self.undo_context.has_open_context() {
            // If an undo is open, we use the diff for change tracking
            self.undo_context
                .track_existing_asset(&mut self.data_set, asset_id)?;
        } else {
            // If we don't have an undo context open, fast path to do change tracking ourselves
            self.modified_assets.insert(asset_id);
            if let Some(asset_info) = self.assets().get(&asset_id) {
                if !self
                    .modified_locations
                    .contains(&asset_info.asset_location())
                {
                    self.modified_locations
                        .insert(asset_info.asset_location().clone());
                }
            }
        }

        Ok(())
    }

    pub fn clear_change_tracking(&mut self) {
        self.modified_assets.clear();
        self.modified_locations.clear();
    }

    pub fn has_changes(&self) -> bool {
        !self.modified_assets.is_empty() || !self.modified_locations.is_empty()
    }

    pub fn is_asset_modified(
        &self,
        asset_id: AssetId,
    ) -> bool {
        self.modified_assets.contains(&asset_id)
    }

    pub fn modified_assets(&self) -> &HashSet<AssetId> {
        &self.modified_assets
    }

    pub fn modified_locations(&self) -> &HashSet<AssetLocation> {
        &self.modified_locations
    }

    pub fn clear_asset_modified_flag(
        &mut self,
        asset_id: AssetId,
    ) {
        self.modified_assets.remove(&asset_id);
    }

    pub fn clear_location_modified_flag(
        &mut self,
        asset_location: &AssetLocation,
    ) {
        self.modified_locations.remove(asset_location);
    }

    pub fn take_modified_assets_and_locations(
        &mut self
    ) -> (HashSet<AssetId>, HashSet<AssetLocation>) {
        let mut modified_assets = HashSet::default();
        std::mem::swap(&mut modified_assets, &mut self.modified_assets);

        let mut modified_locations = HashSet::default();
        std::mem::swap(&mut modified_locations, &mut self.modified_locations);

        (modified_assets, modified_locations)
    }

    pub fn apply_diff(
        &mut self,
        diff: &DataSetDiff,
        modified_assets: &HashSet<AssetId>,
        modified_locations: &HashSet<AssetLocation>,
    ) -> DataSetResult<()> {
        diff.apply(&mut self.data_set, &self.schema_set)?;

        // Marks the assets as changed
        self.modified_assets.extend(modified_assets);
        for modified_location in modified_locations {
            if !self.modified_locations.contains(modified_location) {
                self.modified_locations.insert(modified_location.clone());
            }
        }

        Ok(())
    }

    pub fn new(
        edit_context_key: EditContextKey,
        schema_set: SchemaSet,
        undo_stack: &UndoStack,
    ) -> Self {
        EditContext {
            schema_set,
            data_set: Default::default(),
            undo_context: UndoContext::new(undo_stack, edit_context_key),
            modified_assets: Default::default(),
            modified_locations: Default::default(),
        }
    }

    pub fn new_with_data(
        edit_context_key: EditContextKey,
        schema_set: SchemaSet,
        undo_stack: &UndoStack,
    ) -> Self {
        EditContext {
            schema_set,
            data_set: Default::default(),
            undo_context: UndoContext::new(undo_stack, edit_context_key),
            modified_assets: Default::default(),
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
            &mut self.modified_assets,
            &mut self.modified_locations,
        );
        let end_context_behavior = (f)(self);
        self.undo_context.end_context(
            &self.data_set,
            end_context_behavior,
            &mut self.modified_assets,
            &mut self.modified_locations,
        );
    }

    pub fn commit_pending_undo_context(&mut self) {
        self.undo_context.commit_context(
            &mut self.data_set,
            &mut self.modified_assets,
            &mut self.modified_locations,
        );
    }

    pub fn cancel_pending_undo_context(&mut self) -> DataSetResult<()> {
        self.undo_context.cancel_context(&mut self.data_set)
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

    // pub fn assets_with_locations(&self) -> HashMap<AssetId, &AssetLocation> {
    //     self.data_set.assets.iter().map(|(k, v)| {
    //         (k, &v.asset_location)
    //     })
    // }
    //

    pub fn assets(&self) -> &HashMap<AssetId, DataSetAssetInfo> {
        self.data_set.assets()
    }

    pub fn has_asset(
        &self,
        asset_id: AssetId,
    ) -> bool {
        self.assets().contains_key(&asset_id)
    }

    // pub(crate) fn insert_asset(
    //     &mut self,
    //     obj_info: DataAssetInfo,
    // ) -> AssetId {
    //     let asset_id = self.data_set.insert_asset(obj_info);
    //     self.undo_context.track_new_asset(asset_id);
    //     asset_id
    // }

    pub fn new_asset_with_id(
        &mut self,
        asset_id: AssetId,
        asset_name: &AssetName,
        asset_location: &AssetLocation,
        schema: &SchemaRecord,
    ) -> DataSetResult<()> {
        self.data_set.new_asset_with_id(
            asset_id,
            asset_name.clone(),
            asset_location.clone(),
            schema,
        )?;
        self.track_new_asset(asset_id, asset_location);
        Ok(())
    }

    pub fn new_asset(
        &mut self,
        asset_name: &AssetName,
        asset_location: &AssetLocation,
        schema: &SchemaRecord,
    ) -> AssetId {
        let asset_id =
            self.data_set
                .new_asset(asset_name.clone(), asset_location.clone(), schema);
        self.track_new_asset(asset_id, asset_location);
        asset_id
    }

    pub fn new_asset_from_prototype(
        &mut self,
        asset_name: &AssetName,
        asset_location: &AssetLocation,
        prototype: AssetId,
    ) -> DataSetResult<AssetId> {
        let asset_id = self.data_set.new_asset_from_prototype(
            asset_name.clone(),
            asset_location.clone(),
            prototype,
        )?;
        self.track_new_asset(asset_id, &asset_location);
        Ok(asset_id)
    }

    pub fn init_from_single_object(
        &mut self,
        asset_id: AssetId,
        single_object: &SingleObject,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .copy_from_single_object(asset_id, single_object)
    }

    pub fn restore_assets_from(
        &mut self,
        data_set: DataSet,
    ) -> DataSetResult<()> {
        for (k, v) in data_set.take_assets() {
            self.restore_asset(
                k,
                v.asset_name().clone(),
                v.asset_location().clone(),
                v.import_info().clone().clone(),
                v.build_info().clone(),
                v.prototype(),
                v.schema().fingerprint(),
                v.properties().clone(),
                v.property_null_overrides().clone(),
                v.properties_in_replace_mode().clone(),
                v.dynamic_array_entries().clone(),
            )?;
        }

        Ok(())
    }

    pub(crate) fn restore_asset(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
        asset_location: AssetLocation,
        import_info: Option<ImportInfo>,
        build_info: BuildInfo,
        prototype: Option<AssetId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, OrderedSet<Uuid>>,
    ) -> DataSetResult<()> {
        self.track_new_asset(asset_id, &asset_location);
        self.data_set.restore_asset(
            asset_id,
            asset_name,
            asset_location,
            import_info,
            build_info,
            &self.schema_set,
            prototype,
            schema,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        )
    }

    pub fn delete_asset(
        &mut self,
        asset_id: AssetId,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set.delete_asset(asset_id)
    }

    pub fn set_asset_location(
        &mut self,
        asset_id: AssetId,
        new_location: AssetLocation,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set.set_asset_location(asset_id, new_location)?;
        // Again so that we track the new location too
        self.track_existing_asset(asset_id)?;
        Ok(())
    }

    pub fn set_import_info(
        &mut self,
        asset_id: AssetId,
        import_info: ImportInfo,
    ) -> DataSetResult<()> {
        self.data_set.set_import_info(asset_id, import_info)
    }

    pub fn asset_name(
        &self,
        asset_id: AssetId,
    ) -> Option<&AssetName> {
        self.data_set.asset_name(asset_id)
    }

    pub fn set_asset_name(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set.set_asset_name(asset_id, asset_name)
    }

    pub fn asset_location(
        &self,
        asset_id: AssetId,
    ) -> Option<&AssetLocation> {
        self.data_set.asset_location(asset_id)
    }

    pub fn asset_location_chain(
        &self,
        asset_id: AssetId,
    ) -> DataSetResult<Vec<AssetLocation>> {
        self.data_set.asset_location_chain(asset_id)
    }

    pub fn import_info(
        &self,
        asset_id: AssetId,
    ) -> Option<&ImportInfo> {
        self.data_set.import_info(asset_id)
    }

    pub fn resolve_all_file_references(
        &self,
        asset_id: AssetId,
    ) -> DataSetResult<HashMap<PathBuf, AssetId>> {
        self.data_set.resolve_all_file_references(asset_id)
    }

    pub fn get_all_file_reference_overrides(
        &mut self,
        asset_id: AssetId,
    ) -> Option<&HashMap<PathBuf, AssetId>> {
        self.data_set.get_all_file_reference_overrides(asset_id)
    }

    pub fn set_file_reference_override(
        &mut self,
        asset_id: AssetId,
        path: PathBuf,
        referenced_asset_id: AssetId,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .set_file_reference_override(asset_id, path, referenced_asset_id)
    }

    pub fn asset_prototype(
        &self,
        asset_id: AssetId,
    ) -> Option<AssetId> {
        self.data_set.asset_prototype(asset_id)
    }

    pub fn asset_schema(
        &self,
        asset_id: AssetId,
    ) -> Option<&SchemaRecord> {
        self.data_set.asset_schema(asset_id)
    }

    pub fn get_null_override(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.data_set
            .get_null_override(&self.schema_set, asset_id, path)
    }

    pub fn set_null_override(
        &mut self,
        asset_id: AssetId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .set_null_override(&self.schema_set, asset_id, path, null_override)
    }

    pub fn resolve_null_override(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.data_set
            .resolve_null_override(&self.schema_set, asset_id, path)
    }

    pub fn has_property_override(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<bool> {
        self.data_set.has_property_override(asset_id, path)
    }

    // Just gets if this asset has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<Option<&Value>> {
        self.data_set.get_property_override(asset_id, path)
    }

    // Just sets a property on this asset, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        asset_id: AssetId,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .set_property_override(&self.schema_set, asset_id, path, value)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        if let Some(prototype) = self.asset_prototype(asset_id) {
            self.track_existing_asset(prototype)?;
        }

        self.data_set
            .apply_property_override_to_prototype(&self.schema_set, asset_id, path)
    }

    pub fn resolve_property(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        self.data_set
            .resolve_property(&self.schema_set, asset_id, path)
    }

    pub fn get_dynamic_array_overrides(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<std::slice::Iter<Uuid>> {
        self.data_set
            .get_dynamic_array_overrides(&self.schema_set, asset_id, path)
    }

    pub fn add_dynamic_array_override(
        &mut self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .add_dynamic_array_override(&self.schema_set, asset_id, path)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        asset_id: AssetId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) -> DataSetResult<bool> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .remove_dynamic_array_override(&self.schema_set, asset_id, path, element_id)
    }

    pub fn resolve_dynamic_array(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.data_set
            .resolve_dynamic_array(&self.schema_set, asset_id, path)
    }

    pub fn get_override_behavior(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        self.data_set
            .get_override_behavior(&self.schema_set, asset_id, path)
    }

    pub fn set_override_behavior(
        &mut self,
        asset_id: AssetId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) -> DataSetResult<()> {
        self.track_existing_asset(asset_id)?;
        self.data_set
            .set_override_behavior(&self.schema_set, asset_id, path, behavior)
    }
}

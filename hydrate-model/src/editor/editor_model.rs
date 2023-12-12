use crate::edit_context::EditContext;
use crate::editor::undo::UndoStack;
use crate::{
    AssetId, AssetPath, AssetPathCache, AssetSourceId, DataSet, DataSource,
    FileSystemIdBasedDataSource, FileSystemPathBasedDataSource, HashMap,
    PathNode, PathNodeRoot, SchemaNamedType, SchemaSet,
};
use hydrate_data::{AssetLocation, AssetName, CanonicalPathReference, DataSetError, DataSetResult, ImportInfo, PathReferenceHash, SingleObject};
use hydrate_pipeline::{import_util::ImportToQueue, DynEditorModel, ImporterRegistry, HydrateProjectConfiguration};
use hydrate_schema::{SchemaFingerprint, SchemaRecord};
use slotmap::DenseSlotMap;
use std::path::PathBuf;
use uuid::Uuid;
slotmap::new_key_type! { pub struct EditContextKey; }

pub struct EditorModel {
    project_config: HydrateProjectConfiguration,
    schema_set: SchemaSet,
    undo_stack: UndoStack,
    root_edit_context_key: EditContextKey,
    edit_contexts: DenseSlotMap<EditContextKey, EditContext>,
    //TODO: slot_map?
    data_sources: HashMap<AssetSourceId, Box<dyn DataSource>>,

    //asset_path_cache: AssetPathCache,
    //location_tree: LocationTree,
    path_node_schema: SchemaNamedType,
    path_node_root_schema: SchemaNamedType,
}

pub struct EditorModelWithCache<'a> {
    pub asset_path_cache: &'a AssetPathCache,
    pub editor_model: &'a mut EditorModel,
}

impl<'a> DynEditorModel for EditorModelWithCache<'a> {
    fn schema_set(&self) -> &SchemaSet {
        self.editor_model.schema_set()
    }

    fn handle_import_complete(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
        asset_location: AssetLocation,
        default_asset: &SingleObject,
        replace_with_default_asset: bool,
        import_info: ImportInfo,
        canonical_path_references: &HashMap<CanonicalPathReference, AssetId>,
        _path_references: &HashMap<PathReferenceHash, CanonicalPathReference>,
    ) -> DataSetResult<()> {
        //
        // If the asset is supposed to be regenerated, stomp the existing asset
        //
        let edit_context = self.editor_model.root_edit_context_mut();
        if replace_with_default_asset {
            edit_context.init_from_single_object(
                asset_id,
                asset_name,
                asset_location,
                default_asset,
            )?;
        }

        //
        // Whether it is regenerated or not, update import data
        //
        edit_context.set_import_info(asset_id, import_info)?;
        for (path_reference, referenced_asset_id) in canonical_path_references {
            edit_context.set_path_reference_override(
                asset_id,
                path_reference.clone(),
                *referenced_asset_id,
            )?;
        }

        Ok(())
    }

    fn data_set(&self) -> &DataSet {
        self.editor_model.root_edit_context().data_set()
    }

    fn is_path_node_or_root(
        &self,
        schema_record: &SchemaRecord,
    ) -> bool {
        self.editor_model
            .is_path_node_or_root(schema_record.fingerprint())
    }

    fn asset_display_name_long(
        &self,
        asset_id: AssetId,
    ) -> String {
        self.editor_model
            .asset_display_name_long(asset_id, &self.asset_path_cache)
    }
}

impl EditorModel {
    pub fn new(project_config: HydrateProjectConfiguration, schema_set: SchemaSet) -> Self {
        let undo_stack = UndoStack::default();
        let mut edit_contexts: DenseSlotMap<EditContextKey, EditContext> = Default::default();

        let root_edit_context_key = edit_contexts
            .insert_with_key(|key| EditContext::new(&project_config, key, schema_set.clone(), &undo_stack));

        let path_node_root_schema = schema_set
            .find_named_type(PathNodeRoot::schema_name())
            .unwrap()
            .clone();

        let path_node_schema = schema_set
            .find_named_type(PathNode::schema_name())
            .unwrap()
            .clone();

        EditorModel {
            project_config,
            schema_set,
            undo_stack,
            root_edit_context_key,
            edit_contexts,
            data_sources: Default::default(),
            //location_tree: Default::default(),
            //asset_path_cache: AssetPathCache::empty(),
            path_node_root_schema,
            path_node_schema,
        }
    }

    pub fn path_node_schema(&self) -> &SchemaNamedType {
        &self.path_node_schema
    }

    pub fn path_node_root_schema(&self) -> &SchemaNamedType {
        &self.path_node_root_schema
    }

    pub fn data_sources(&self) -> &HashMap<AssetSourceId, Box<dyn DataSource>> {
        &self.data_sources
    }

    pub fn is_path_node_or_root(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> bool {
        self.path_node_schema.fingerprint() == fingerprint
            || self.path_node_root_schema.fingerprint() == fingerprint
    }

    pub fn is_generated_asset(
        &self,
        asset_id: AssetId,
    ) -> bool {
        for data_source in self.data_sources.values() {
            if data_source.is_generated_asset(asset_id) {
                return true;
            }
        }

        false
    }

    pub fn persist_generated_asset(
        &mut self,
        asset_id: AssetId,
    ) {
        for (_, data_source) in &mut self.data_sources {
            let root_edit_context = self
                .edit_contexts
                .get_mut(self.root_edit_context_key)
                .unwrap();

            data_source.persist_generated_asset(root_edit_context, asset_id);
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
        for (_key, context) in &self.edit_contexts {
            if context.has_changes() {
                return true;
            }
        }

        false
    }

    pub fn schema_set(&self) -> &SchemaSet {
        &self.schema_set
    }

    pub fn clone_schema_set(&self) -> SchemaSet {
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

    pub fn asset_path(
        &self,
        asset_id: AssetId,
        asset_path_cache: &AssetPathCache,
    ) -> Option<AssetPath> {
        let root_data_set = &self.root_edit_context().data_set;
        let location = root_data_set.asset_location(asset_id);

        // Look up the location, if we don't find it just assume the asset is at the root. This
        // allows some degree of robustness even when data is in a bad state (like cyclical references)
        let path = location
            .map(|x| asset_path_cache.path_to_id_lookup().get(&x.path_node_id()))
            .flatten()
            .cloned()?;

        let name = root_data_set
            .asset_name(asset_id)
            .unwrap()
            .as_string();
        if let Some(name) = name {
            Some(path.join(name))
        } else {
            Some(path.join(&format!("{}", asset_id.as_uuid())))
        }
    }

    pub fn asset_display_name_long(
        &self,
        asset_id: AssetId,
        asset_path_cache: &AssetPathCache,
    ) -> String {
        self.asset_path(asset_id, asset_path_cache)
            .map(|x| x.as_str().to_string())
            .unwrap_or_else(|| format!("{}", asset_id.as_uuid()))
    }

    pub fn data_source(
        &mut self,
        asset_source_id: AssetSourceId,
    ) -> Option<&dyn DataSource> {
        self.data_sources.get(&asset_source_id).map(|x| &**x)
    }

    pub fn is_a_root_asset(
        &self,
        asset_id: AssetId,
    ) -> bool {
        for source in self.data_sources.keys() {
            if *source.uuid() == asset_id.as_uuid() {
                return true;
            }
        }

        false
    }

    pub fn add_file_system_id_based_asset_source<RootPathT: Into<PathBuf>>(
        &mut self,
        project_config: &HydrateProjectConfiguration,
        data_source_name: &str,
        file_system_root_path: RootPathT,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> AssetSourceId {
        let file_system_root_path = dunce::canonicalize(&file_system_root_path.into()).unwrap();
        let path_node_root_schema = self.path_node_root_schema.as_record().unwrap().clone();
        let root_edit_context = self.root_edit_context_mut();

        // Commit any pending changes so we have a clean change tracking state
        root_edit_context.commit_pending_undo_context();

        //
        // Create the PathNodeRoot asset that acts as the root location for all assets in this DS
        //
        let asset_source_id = AssetSourceId::new();
        let root_asset_id = AssetId::from_uuid(*asset_source_id.uuid());
        root_edit_context
            .new_asset_with_id(
                root_asset_id,
                &AssetName::new(data_source_name),
                &AssetLocation::null(),
                &path_node_root_schema,
            )
            .unwrap();

        // Clear change tracking so that the new root asset we just added doesn't appear as a unsaved change.
        // (It should never serialize)
        root_edit_context.clear_change_tracking();

        //
        // Create the data source and force full reload of it
        //
        let mut fs = FileSystemIdBasedDataSource::new(
            file_system_root_path.clone(),
            root_edit_context,
            asset_source_id,
        );
        fs.load_from_storage(project_config, root_edit_context, imports_to_queue);

        self.data_sources.insert(asset_source_id, Box::new(fs));

        asset_source_id
    }

    pub fn add_file_system_path_based_data_source<RootPathT: Into<PathBuf>>(
        &mut self,
        project_config: &HydrateProjectConfiguration,
        data_source_name: &str,
        file_system_root_path: RootPathT,
        importer_registry: &ImporterRegistry,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> AssetSourceId {
        let file_system_root_path = dunce::canonicalize(&file_system_root_path.into()).unwrap();
        let path_node_root_schema = self.path_node_root_schema.as_record().unwrap().clone();
        let root_edit_context = self.root_edit_context_mut();

        // Commit any pending changes so we have a clean change tracking state
        root_edit_context.commit_pending_undo_context();

        //
        // Create the PathNodeRoot asset that acts as the root location for all assets in this DS
        //
        let asset_source_id = AssetSourceId::new();
        let root_asset_id = AssetId::from_uuid(*asset_source_id.uuid());
        root_edit_context
            .new_asset_with_id(
                root_asset_id,
                &AssetName::new(data_source_name),
                &AssetLocation::null(),
                &path_node_root_schema,
            )
            .unwrap();

        // Clear change tracking so that the new root asset we just added doesn't appear as a unsaved change.
        // (It should never serialize)
        root_edit_context.clear_change_tracking();

        //
        // Create the data source and force full reload of it
        //
        let mut fs = FileSystemPathBasedDataSource::new(
            file_system_root_path.clone(),
            root_edit_context,
            asset_source_id,
            importer_registry,
        );
        fs.load_from_storage(project_config, root_edit_context, imports_to_queue);

        self.data_sources.insert(asset_source_id, Box::new(fs));

        asset_source_id
    }

    pub fn save_root_edit_context(&mut self) {
        //
        // Ensure pending edits are flushed to the data set so that our modified assets list is fully up to date
        //
        let root_edit_context = self
            .edit_contexts
            .get_mut(self.root_edit_context_key)
            .unwrap();
        root_edit_context.commit_pending_undo_context();

        for (_id, data_source) in &mut self.data_sources {
            data_source.flush_to_storage(root_edit_context);
        }

        //
        // Clear modified assets list since we saved everything to disk
        //
        root_edit_context.clear_change_tracking();
    }

    pub fn revert_root_edit_context(
        &mut self,
        project_config: &HydrateProjectConfiguration,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) {
        //
        // Ensure pending edits are cleared
        //
        let root_edit_context = self
            .edit_contexts
            .get_mut(self.root_edit_context_key)
            .unwrap();
        root_edit_context.cancel_pending_undo_context().unwrap();

        //
        // Take the contents of the modified asset list, leaving the edit context with a cleared list
        //
        let (modified_assets, modified_locations) =
            root_edit_context.take_modified_assets_and_locations();
        println!(
            "Revert:\nAssets: {:?}\nLocations: {:?}",
            modified_assets, modified_locations
        );

        for (_id, data_source) in &mut self.data_sources {
            data_source.load_from_storage(project_config, root_edit_context, imports_to_queue);
        }

        //
        // Clear modified assets list since we reloaded everything from disk.
        //
        root_edit_context.clear_change_tracking();
        //root_edit_context.cancel_pending_undo_context();

        //self.refresh_asset_path_lookups();
        //self.refresh_location_tree();
    }

    pub fn close_file_system_source(
        &mut self,
        _asset_source_id: AssetSourceId,
    ) {
        unimplemented!();
        // kill edit contexts or fail

        // clear root_edit_context of data from this source

        // drop the source
        //let old = self.data_sources.remove(&asset_source_id);
        //assert!(old.is_some());
    }

    // Spawns a separate edit context with copies of the given assets. The undo stack will be shared
    // globally, but changes will not be visible on the root context. The edit context will be flushed
    // to the root context in a single operation. Generally, we don't expect assets opened in a
    // separate edit context to change in the root context, but there is nothing that prevents it.
    pub fn open_edit_context(
        &mut self,
        assets: &[AssetId],
    ) -> DataSetResult<EditContextKey> {
        let new_edit_context_key = self.edit_contexts.insert_with_key(|key| {
            EditContext::new_with_data(&self.project_config, key, self.schema_set.clone(), &self.undo_stack)
        });

        let [root_edit_context, new_edit_context] = self
            .edit_contexts
            .get_disjoint_mut([self.root_edit_context_key, new_edit_context_key])
            .unwrap();

        for asset in assets {
            if !root_edit_context.assets().contains_key(asset) {
                return Err(DataSetError::AssetNotFound);
            }
        }

        for &asset_id in assets {
            new_edit_context
                .data_set
                .copy_from(root_edit_context.data_set(), asset_id)
                .expect("Could not copy asset to newly created edit context");
        }

        Ok(new_edit_context_key)
    }

    pub fn flush_edit_context_to_root(
        &mut self,
        edit_context: EditContextKey,
    ) -> DataSetResult<()> {
        assert_ne!(edit_context, self.root_edit_context_key);
        let [root_context, context_to_flush] = self
            .edit_contexts
            .get_disjoint_mut([self.root_edit_context_key, edit_context])
            .unwrap();

        // In the case of failure we want to flush as much as we can, so keep the error around and
        // return it after trying to flush all the assetsa
        let mut first_error = None;
        for &asset_id in context_to_flush.modified_assets() {
            if let Err(e) = root_context
                .data_set
                .copy_from(&context_to_flush.data_set, asset_id)
            {
                if first_error.is_none() {
                    first_error = Some(Err(e));
                }
            }
        }

        context_to_flush.clear_change_tracking();

        first_error.unwrap_or(Ok(()))
    }

    pub fn close_edit_context(
        &mut self,
        edit_context: EditContextKey,
    ) {
        assert_ne!(edit_context, self.root_edit_context_key);
        self.edit_contexts.remove(edit_context);
    }

    pub fn undo(&mut self) -> DataSetResult<()> {
        self.undo_stack.undo(&mut self.edit_contexts)
    }

    pub fn redo(&mut self) -> DataSetResult<()> {
        self.undo_stack.redo(&mut self.edit_contexts)
    }

    // pub fn refresh_tree_node_cache(&mut self) {
    //     let asset_path_cache = AssetPathCache::new(self);
    //
    //     let location_tree = LocationTree::build(
    //         &self,
    //         &asset_path_cache
    //     );
    //
    //     self.asset_path_cache = asset_path_cache;
    //     self.location_tree = location_tree;
    // }
    //
    // pub fn cached_location_tree(&self) -> &LocationTree {
    //     &self.location_tree
    // }
}

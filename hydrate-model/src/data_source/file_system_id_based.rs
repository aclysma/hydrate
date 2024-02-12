use crate::edit_context::EditContext;
use crate::{AssetId, AssetSourceId, DataSource, PathNodeRoot, PendingFileOperations};
use hydrate_base::hashing::HashMap;
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};
use hydrate_data::{AssetLocation, HashObjectMode};
use hydrate_pipeline::{HydrateProjectConfiguration, ImportJobToQueue};
use hydrate_schema::SchemaNamedType;
use std::path::PathBuf;

struct FileMetadata {
    // size_in_bytes: u64,
    // last_modified_time: Option<SystemTime>,
}

impl FileMetadata {
    pub fn new(_metadata: &std::fs::Metadata) -> Self {
        FileMetadata {
            // size_in_bytes: metadata.len(),
            // last_modified_time: metadata.modified().ok()
        }
    }

    // pub fn has_changed(&self, metadata: &std::fs::Metadata) -> bool {
    //     self.size_in_bytes != metadata.len() || self.last_modified_time != metadata.modified().ok()
    // }
}

struct AssetDiskState {
    object_hash: u64,
    _file_metadata: FileMetadata,
}

pub struct FileSystemIdBasedDataSource {
    asset_source_id: AssetSourceId,
    file_system_root_path: PathBuf,

    // Any asset ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    assets_disk_state: HashMap<AssetId, AssetDiskState>,

    path_node_root_schema: SchemaNamedType,
}

impl FileSystemIdBasedDataSource {
    fn is_asset_owned_by_this_data_source(
        &self,
        edit_context: &EditContext,
        asset_id: AssetId,
    ) -> bool {
        if edit_context.asset_schema(asset_id).unwrap().fingerprint()
            == self.path_node_root_schema.fingerprint()
        {
            return false;
        }

        //TODO: is_null means we default to using this source
        let root_location = edit_context
            .asset_location_chain(asset_id)
            .unwrap_or_default()
            .last()
            .cloned()
            .unwrap_or_else(AssetLocation::null);
        root_location.path_node_id().as_uuid() == *self.asset_source_id.uuid()
            || root_location.is_null()
    }

    pub fn asset_source_id(&self) -> AssetSourceId {
        self.asset_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
        asset_source_id: AssetSourceId,
    ) -> Self {
        let path_node_root_schema = edit_context
            .schema_set()
            .find_named_type(PathNodeRoot::schema_name())
            .unwrap()
            .clone();

        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system asset data source {:?}",
            file_system_root_path,
        );

        FileSystemIdBasedDataSource {
            asset_source_id,
            file_system_root_path: file_system_root_path.into(),
            assets_disk_state: Default::default(),
            path_node_root_schema,
        }
    }

    fn path_for_asset(
        &self,
        asset_id: AssetId,
    ) -> PathBuf {
        uuid_to_path(&self.file_system_root_path, asset_id.as_uuid(), "af")
    }
}

impl DataSource for FileSystemIdBasedDataSource {
    fn is_generated_asset(
        &self,
        _asset_id: AssetId,
    ) -> bool {
        // this data source does not contain source files so can't have generated assets
        false
    }

    // fn asset_symbol_name(&self, asset_id: AssetId) -> Option<String> {
    //     None
    // }

    fn persist_generated_asset(
        &mut self,
        _edit_context: &mut EditContext,
        _asset_id: AssetId,
    ) {
        // this data source does not contain source files so can't have generated assets
    }

    #[profiling::function]
    fn load_from_storage(
        &mut self,
        _project_config: &HydrateProjectConfiguration,
        edit_context: &mut EditContext,
        _import_job_to_queue: &mut ImportJobToQueue,
    ) {
        profiling::scope!(&format!(
            "load_from_storage {:?}",
            self.file_system_root_path
        ));

        //
        // Delete all assets from the database owned by this data source
        //
        let mut assets_to_delete = Vec::default();
        for (asset_id, _) in edit_context.assets() {
            if self.is_asset_owned_by_this_data_source(edit_context, *asset_id) {
                assets_to_delete.push(*asset_id);
            }
        }

        for asset_to_delete in assets_to_delete {
            edit_context.delete_asset(asset_to_delete).unwrap();
        }

        self.assets_disk_state.clear();

        //
        // Recreate all assets from storage
        //
        let walker =
            globwalk::GlobWalkerBuilder::from_patterns(&self.file_system_root_path, &["**.af"])
                .file_type(globwalk::FileType::FILE)
                .build()
                .unwrap();

        for file in walker {
            if let Ok(file) = file {
                let file = dunce::canonicalize(&file.path()).unwrap();

                let asset_file_metadata = FileMetadata::new(&std::fs::metadata(&file).unwrap());

                //println!("asset file {:?}", file);
                let file_uuid = path_to_uuid(&self.file_system_root_path, &file).unwrap();
                let contents = std::fs::read_to_string(&file).unwrap();
                let default_asset_location =
                    AssetLocation::new(AssetId(*self.asset_source_id.uuid()));

                let schema_set = edit_context.schema_set().clone();
                crate::json_storage::AssetJson::load_asset_from_string(
                    edit_context,
                    &schema_set,
                    Some(file_uuid),
                    default_asset_location,
                    None,
                    &contents,
                )
                .unwrap();
                let asset_id = AssetId::from_uuid(file_uuid);

                let object_hash = edit_context
                    .data_set()
                    .hash_object(asset_id, HashObjectMode::FullObjectWithLocationId)
                    .unwrap();

                let old = self.assets_disk_state.insert(
                    asset_id,
                    AssetDiskState {
                        object_hash: object_hash,
                        _file_metadata: asset_file_metadata,
                    },
                );
                assert!(old.is_none());
            }
        }
    }

    fn flush_to_storage(
        &mut self,
        edit_context: &mut EditContext,
    ) {
        profiling::scope!(&format!(
            "flush_to_storage {:?}",
            self.file_system_root_path
        ));

        let mut pending_deletes = Vec::<AssetId>::default();
        let mut pending_writes = Vec::<AssetId>::default();

        for &asset_id in edit_context.assets().keys() {
            if self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
                match self.assets_disk_state.get(&asset_id) {
                    None => {
                        // There is a newly created asset that has never been saved
                        pending_writes.push(asset_id);
                    }
                    Some(asset_disk_state) => {
                        let object_hash = edit_context
                            .data_set()
                            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationId)
                            .unwrap();
                        if asset_disk_state.object_hash != object_hash {
                            // The object has been modified and no longer matches disk state
                            pending_writes.push(asset_id);
                        }
                    }
                }
            }
        }

        // Is there anything that's been deleted?
        for (&asset_id, _) in &self.assets_disk_state {
            if !edit_context.has_asset(asset_id)
                || !self.is_asset_owned_by_this_data_source(edit_context, asset_id)
            {
                // There is an asset that no longer exists, but the file is still on disk
                pending_deletes.push(asset_id);
            }
        }

        //
        // Save any created/updated assets
        //
        for asset_id in pending_writes {
            if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                // never save the root asset
                continue;
            }

            let asset_info = edit_context.data_set().assets().get(&asset_id).unwrap();

            // If the asset doesn't have a location set or is set to the root of this data
            // source, serialize with a null location
            let asset_location = if asset_info.asset_location().is_null()
                || asset_info.asset_location().path_node_id().as_uuid()
                    == *self.asset_source_id.uuid()
            {
                None
            } else {
                Some(asset_info.asset_location())
            };

            let data = crate::json_storage::AssetJson::save_asset_to_string(
                edit_context.schema_set(),
                edit_context.assets(),
                asset_id,
                false, //don't include ID because we assume it by file name
                asset_location,
            );
            let file_path = self.path_for_asset(asset_id);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }

            std::fs::write(&file_path, data).unwrap();

            let object_hash = edit_context
                .data_set()
                .hash_object(asset_id, HashObjectMode::FullObjectWithLocationId)
                .unwrap();
            let asset_file_metadata = FileMetadata::new(&std::fs::metadata(&file_path).unwrap());

            self.assets_disk_state.insert(
                asset_id,
                AssetDiskState {
                    object_hash,
                    _file_metadata: asset_file_metadata,
                },
            );
        }

        //
        // Delete assets that no longer exist
        //
        for asset_id in pending_deletes {
            let file_path = self.path_for_asset(asset_id);
            std::fs::remove_file(&file_path).unwrap();
            self.assets_disk_state.remove(&asset_id);

            //TODO: Clean up empty parent dirs?
        }
    }

    fn edit_context_has_unsaved_changes(
        &self,
        edit_context: &EditContext,
    ) -> bool {
        for &asset_id in edit_context.assets().keys() {
            if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                // ignore the root asset
                continue;
            }

            if self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
                match self.assets_disk_state.get(&asset_id) {
                    None => {
                        // There is a newly created asset that has never been saved
                        return true;
                    }
                    Some(asset_disk_state) => {
                        let object_hash = edit_context
                            .data_set()
                            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationId)
                            .unwrap();
                        if asset_disk_state.object_hash != object_hash {
                            // The object has been modified and no longer matches disk state
                            return true;
                        }
                    }
                }
            }
        }

        // Is there anything that's been deleted?
        for (&asset_id, _) in &self.assets_disk_state {
            if !edit_context.has_asset(asset_id)
                || !self.is_asset_owned_by_this_data_source(edit_context, asset_id)
            {
                // There is an asset that no longer exists, but the file is still on disk
                return true;
            }
        }

        return false;
    }

    fn append_pending_file_operations(
        &self,
        edit_context: &EditContext,
        pending_file_operations: &mut PendingFileOperations,
    ) {
        for &asset_id in edit_context.assets().keys() {
            if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                // ignore the root asset
                continue;
            }

            if self.is_asset_owned_by_this_data_source(edit_context, asset_id) {
                match self.assets_disk_state.get(&asset_id) {
                    None => {
                        // There is a newly created asset that has never been saved
                        let file_path = self.path_for_asset(asset_id);
                        pending_file_operations
                            .create_operations
                            .push((asset_id, file_path));
                    }
                    Some(asset_disk_state) => {
                        let object_hash = edit_context
                            .data_set()
                            .hash_object(asset_id, HashObjectMode::FullObjectWithLocationId)
                            .unwrap();
                        if asset_disk_state.object_hash != object_hash {
                            // The object has been modified and no longer matches disk state
                            let file_path = self.path_for_asset(asset_id);
                            pending_file_operations
                                .modify_operations
                                .push((asset_id, file_path));
                        }
                    }
                }
            }
        }

        // Is there anything that's been deleted?
        for (&asset_id, _) in &self.assets_disk_state {
            if !edit_context.has_asset(asset_id)
                || !self.is_asset_owned_by_this_data_source(edit_context, asset_id)
            {
                // There is an asset that no longer exists, but the file is still on disk
                let file_path = self.path_for_asset(asset_id);
                pending_file_operations
                    .delete_operations
                    .push((asset_id, file_path));
            }
        }
    }
}

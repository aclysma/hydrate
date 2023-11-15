use crate::edit_context::EditContext;
use crate::import_util::ImportToQueue;
use crate::{DataSource, HashSet, AssetId, AssetSourceId, PathNodeRoot};
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};
use hydrate_data::AssetLocation;
use hydrate_schema::SchemaNamedType;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn load_asset_files(
    edit_context: &mut EditContext,
    root_path: &Path,
    asset_source_id: AssetSourceId,
    all_asset_ids_on_disk: &mut HashSet<AssetId>,
) {
    let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.af"])
        .file_type(globwalk::FileType::FILE)
        .build()
        .unwrap();

    for file in walker {
        if let Ok(file) = file {
            //println!("asset file {:?}", file);
            let file_uuid = path_to_uuid(root_path, file.path()).unwrap();
            let contents = std::fs::read_to_string(file.path()).unwrap();
            crate::json_storage::EditContextAssetJson::load_edit_context_asset_from_string(
                edit_context,
                Some(file_uuid),
                asset_source_id,
                None,
                &contents,
            );
            let asset_id = AssetId::from_uuid(file_uuid);
            let asset_location = edit_context
                .assets()
                .get(&asset_id)
                .unwrap()
                .asset_location()
                .clone();
            edit_context.clear_asset_modified_flag(asset_id);
            edit_context.clear_location_modified_flag(&asset_location);
            all_asset_ids_on_disk.insert(asset_id);
        }
    }
}

pub struct FileSystemIdBasedDataSource {
    asset_source_id: AssetSourceId,
    file_system_root_path: PathBuf,

    // Any asset ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_asset_ids_on_disk: HashSet<AssetId>,

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
            all_asset_ids_on_disk: Default::default(),
            path_node_root_schema,
        }
    }

    fn find_all_modified_assets(
        &self,
        edit_context: &EditContext,
    ) -> HashSet<AssetId> {
        // We need to handle assets that were moved into this data source that weren't previous in it
        let mut modified_assets = edit_context.modified_assets().clone();

        for asset_id in edit_context.assets().keys() {
            if self.is_asset_owned_by_this_data_source(edit_context, *asset_id) {
                if !self.all_asset_ids_on_disk.contains(asset_id) {
                    modified_assets.insert(*asset_id);
                }
            }
        }

        modified_assets
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
        edit_context: &mut EditContext,
        _imports_to_queue: &mut Vec<ImportToQueue>,
    ) {
        profiling::scope!(&format!("load_from_storage {:?}", self.file_system_root_path));

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
            edit_context.delete_asset(asset_to_delete);
        }

        //
        // Recreate all assets from storage
        //
        load_asset_files(
            edit_context,
            &self.file_system_root_path,
            self.asset_source_id,
            &mut self.all_asset_ids_on_disk,
        );
    }

    fn flush_to_storage(
        &mut self,
        edit_context: &mut EditContext,
    ) {
        profiling::scope!(&format!("flush_to_storage {:?}", self.file_system_root_path));

        // Delete files for assets that were deleted
        let modified_assets = self.find_all_modified_assets(edit_context);
        for asset_id in &modified_assets {
            if self.all_asset_ids_on_disk.contains(&asset_id)
                && !edit_context.has_asset(*asset_id)
            {
                //TODO: delete the asset file
                self.all_asset_ids_on_disk.remove(&asset_id);
            }
        }

        for asset_id in &modified_assets {
            if let Some(asset_info) = edit_context.assets().get(asset_id) {
                if self.is_asset_owned_by_this_data_source(edit_context, *asset_id) {
                    if asset_id.as_uuid() == *self.asset_source_id.uuid() {
                        // never save the root asset
                        continue;
                    }

                    let parent_dir = asset_info.asset_location().path_node_id().as_uuid();
                    let parent_dir = if parent_dir == Uuid::nil()
                        || parent_dir == *self.asset_source_id.uuid()
                    {
                        None
                    } else {
                        Some(parent_dir)
                    };

                    let data = crate::json_storage::EditContextAssetJson::save_edit_context_asset_to_string(
                        edit_context,
                        *asset_id,
                        false, //don't include ID because we assume it by file name
                        parent_dir
                    );
                    let file_path =
                        uuid_to_path(&self.file_system_root_path, asset_id.as_uuid(), "af");
                    self.all_asset_ids_on_disk.insert(*asset_id);

                    if let Some(parent) = file_path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }

                    std::fs::write(file_path, data).unwrap();
                }
            }
        }
    }

    //TODO: revert_some(asset_id_list)
    // - Delete any asset in the list
    // - Load from file any asset in the list
}

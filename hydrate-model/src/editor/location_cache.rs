use crate::{AssetPath, EditorModel};
use hydrate_base::hashing::HashMap;
use hydrate_base::AssetId;
use hydrate_data::DataSet;
use hydrate_schema::{DataSetResult, SchemaNamedType};

pub fn build_path_lookup(
    data_set: &DataSet,
    path_node_type: &SchemaNamedType,
    path_node_root_type: &SchemaNamedType,
) -> DataSetResult<HashMap<AssetId, AssetPath>> {
    let mut paths = HashMap::<AssetId, AssetPath>::default();
    for (asset_id, info) in data_set.assets() {
        if info.schema().fingerprint() == path_node_root_type.fingerprint() {
            // Special case for path node roots
            if let Some(name) = info.asset_name().as_string() {
                paths.insert(*asset_id, AssetPath::new_root(name));
            }
        } else {
            // For assets that *are* path nodes, use their ID directly. For assets that aren't
            // path nodes, use their location asset ID
            let path_node_id = if info.schema().fingerprint() == path_node_type.fingerprint()
                || info.schema().fingerprint() == path_node_root_type.fingerprint()
            {
                *asset_id
            } else {
                // We could process assets so that if for some reason the parent nodes don't exist, we can still
                // generate path lookups for them. Instead we will consider a parent not being found as
                // the asset being at the root level. We could also have a "lost and found" UI.
                //info.asset_location().path_node_id()
                continue;
            };

            // We will walk up the location chain and cache the path_node_id/path pairs. (We resolve
            // the parents recursively going all the way up to the root, and then appending the
            // current node to it's parent's resolved path.)
            let mut chain = data_set.asset_location_chain(path_node_id)?;
            if !chain.is_empty() {
                let name = data_set.asset_name(chain.pop().unwrap().path_node_id())?;
                if let Some(name) = name.as_string() {
                    let mut asset_path = AssetPath::new_root(name);

                    let mut path_is_valid = true;
                    for i in (0..chain.len()).rev() {
                        let name = data_set.asset_name(chain[i].path_node_id())?;
                        if let Some(name) = name.as_string() {
                            asset_path = asset_path.join(name);
                        } else {
                            path_is_valid = false;
                            break;
                        }
                    }

                    if path_is_valid {
                        let name = data_set.asset_name(*asset_id)?;
                        if let Some(name) = name.as_string() {
                            asset_path = asset_path.join(name);
                            paths.insert(*asset_id, asset_path);
                        }
                    }
                }
            }
        }
    }

    Ok(paths)
}

pub struct AssetPathCache {
    path_to_id_lookup: HashMap<AssetId, AssetPath>,
}

impl AssetPathCache {
    pub fn empty() -> Self {
        AssetPathCache {
            path_to_id_lookup: Default::default(),
        }
    }

    pub fn build(editor_model: &EditorModel) -> DataSetResult<Self> {
        let path_to_id_lookup = build_path_lookup(
            editor_model.root_edit_context().data_set(),
            editor_model.path_node_schema(),
            editor_model.path_node_root_schema(),
        )?;

        Ok(AssetPathCache { path_to_id_lookup })
    }

    pub fn path_to_id_lookup(&self) -> &HashMap<AssetId, AssetPath> {
        &self.path_to_id_lookup
    }
}

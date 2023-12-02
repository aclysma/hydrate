use crate::{AssetPath, EditorModel};
use hydrate_base::hashing::{HashMap, HashSet};
use hydrate_base::AssetId;
use hydrate_data::DataSet;
use hydrate_schema::SchemaNamedType;

fn do_populate_path(
    data_set: &DataSet,
    path_stack: &mut HashSet<AssetId>,
    paths: &mut HashMap<AssetId, AssetPath>,
    path_node: AssetId,
) -> AssetPath {
    if path_node.is_null() {
        return AssetPath::root();
    }

    // If we already know the path for the tree node, just return it
    if let Some(parent_path) = paths.get(&path_node) {
        return parent_path.clone();
    }

    // To detect cyclical references, we accumulate visited assets into a set
    let is_cyclical_reference = !path_stack.insert(path_node);
    let source_id_and_path = if is_cyclical_reference {
        // If we detect a cycle, bail and return root path
        AssetPath::root()
    } else {
        if let Some(asset) = data_set.assets().get(&path_node) {
            if let Some(name) = asset.asset_name().as_string() {
                // Parent is found, named, and not a cyclical reference
                let parent = do_populate_path(
                    data_set,
                    path_stack,
                    paths,
                    asset.asset_location().path_node_id(),
                );
                let path = parent.join(name);
                path
            } else {
                // Parent is unnamed, just treat as being at root path
                AssetPath::root()
            }
        } else {
            // Can't find parent, just treat as being at root path
            AssetPath::root()
        }
    };

    paths.insert(path_node, source_id_and_path.clone());

    if !is_cyclical_reference {
        path_stack.remove(&path_node);
    }

    source_id_and_path
}

pub fn build_path_lookup(
    data_set: &DataSet,
    path_node_type: &SchemaNamedType,
    path_node_root_type: &SchemaNamedType,
) -> HashMap<AssetId, AssetPath> {
    let mut path_stack = HashSet::default();
    let mut paths = HashMap::<AssetId, AssetPath>::default();
    for (asset_id, info) in data_set.assets() {
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
        do_populate_path(data_set, &mut path_stack, &mut paths, path_node_id);
    }

    paths
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

    pub fn build(editor_model: &EditorModel) -> Self {
        let path_to_id_lookup = build_path_lookup(
            editor_model.root_edit_context().data_set(),
            editor_model.path_node_schema(),
            editor_model.path_node_root_schema(),
        );

        AssetPathCache { path_to_id_lookup }
    }

    pub fn path_to_id_lookup(&self) -> &HashMap<AssetId, AssetPath> {
        &self.path_to_id_lookup
    }
}

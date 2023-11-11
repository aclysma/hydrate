use crate::{DataSet, DataSource, HashMap, AssetId, AssetLocation, AssetPath, AssetSourceId};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct LocationTreeNodeKey {
    name: String,
    location: AssetLocation,
}

impl LocationTreeNodeKey {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn location(&self) -> &AssetLocation {
        &self.location
    }
}

impl PartialOrd<Self> for LocationTreeNodeKey {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocationTreeNodeKey {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        match self.location.cmp(&other.location) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.name.cmp(&other.name),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

#[derive(Debug)]
pub struct LocationTreeNode {
    //pub path: AssetPath,
    pub location: AssetLocation,
    pub location_root: AssetLocation,
    pub children: BTreeMap<LocationTreeNodeKey, LocationTreeNode>,
    pub has_changes: bool,
}

#[derive(Debug)]
pub struct LocationTree {
    pub root_nodes: BTreeMap<LocationTreeNodeKey, LocationTreeNode>,
}

impl Default for LocationTree {
    fn default() -> Self {
        LocationTree {
            root_nodes: Default::default(),
        }
    }
}

impl LocationTree {
    pub fn create_node(
        &mut self,
        data_set: &DataSet,
        tree_node_id: AssetId,
    ) {
        let mut path_asset_stack = vec![AssetLocation::new(tree_node_id)];
        path_asset_stack.append(&mut data_set.asset_location_chain(tree_node_id));

        //
        // Get the node key for the first element of the path. It should already exist because we create
        // nodes for the data sources.
        //
        let root_location = path_asset_stack.last().cloned().unwrap(); //.unwrap_or(AssetLocation::new(tree_node_id));
        let root_location_path_node_id = root_location.path_node_id();

        let root_tree_node_key = LocationTreeNodeKey {
            location: root_location.clone(),
            name: data_set
                .asset_name(root_location_path_node_id)
                .as_string()
                .cloned()
                .unwrap_or_default(),
        };

        path_asset_stack.pop();

        if let Some(mut tree_node) = self.root_nodes.get_mut(&root_tree_node_key) {
            while let Some(node_object) = path_asset_stack.pop() {
                // Unnamed assets can't be paths
                //let node_location = AssetLocation::new(node_asset);
                //let location_chain = data_set.asset_location_chain(node_asset.path_node_id());

                let node_name = data_set
                    .asset_name(node_object.path_node_id())
                    .as_string()
                    .cloned()
                    .unwrap(); //.unwrap_or_else(|| node_asset.as_uuid().to_string());

                let node_key = LocationTreeNodeKey {
                    name: node_name,
                    location: node_object.clone(),
                };

                tree_node = tree_node.children.entry(node_key).or_insert_with(|| {
                    //let path = paths.get(&node_asset).unwrap().clone();
                    //let node_location = AssetLocation::new(source, location.parent_tree_node());
                    //let location = AssetLocation::new(nod)
                    let has_changes = false; //unsaved_paths.contains(&node_location);
                    LocationTreeNode {
                        //path,
                        //source: node_location.source(),
                        location: node_object,
                        location_root: root_location.clone(),
                        children: Default::default(),
                        has_changes,
                    }
                });
            }
        } else {
            //TODO: Handle this
        }
    }

    pub fn build(
        data_sources: &HashMap<AssetSourceId, Box<dyn DataSource>>,
        data_set: &DataSet,
        paths: &HashMap<AssetId, AssetPath>,
    ) -> Self {
        // Create root nodes for all the data sources
        let mut root_nodes: BTreeMap<LocationTreeNodeKey, LocationTreeNode> = Default::default();
        for (source_id, _data_source) in data_sources {
            let location = AssetLocation::new(AssetId::from_uuid(*source_id.uuid()));
            let name = data_set.asset_name(location.path_node_id());
            root_nodes.insert(
                LocationTreeNodeKey {
                    location: location.clone(),
                    name: name.as_string().cloned().unwrap_or_default(),
                },
                LocationTreeNode {
                    location,
                    location_root: AssetLocation::null(),
                    children: Default::default(),
                    has_changes: false,
                },
            );
        }

        let mut tree = LocationTree { root_nodes };

        // Iterate all known paths and ensure a node exists in the tree for each segment of each path
        for (tree_node_id, _path) in paths {
            // Skip the root component since it is our root node
            tree.create_node(data_set, *tree_node_id);
        }

        tree
    }
}

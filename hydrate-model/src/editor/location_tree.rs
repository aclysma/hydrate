use crate::{DataSet, DataSource, HashMap, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct LocationTreeNodeKey {
    name: String,
    location: ObjectLocation,
}

impl LocationTreeNodeKey {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn location(&self) -> &ObjectLocation {
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
    //pub path: ObjectPath,
    pub location: ObjectLocation,
    pub location_root: ObjectLocation,
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
        tree_node_id: ObjectId,
    ) {
        let mut path_object_stack = vec![ObjectLocation::new(tree_node_id)];
        path_object_stack.append(&mut data_set.object_location_chain(tree_node_id));

        //
        // Get the node key for the first element of the path. It should already exist because we create
        // nodes for the data sources.
        //
        let root_location = path_object_stack.last().cloned().unwrap(); //.unwrap_or(ObjectLocation::new(tree_node_id));
        let root_location_path_node_id = root_location.path_node_id();

        let root_tree_node_key = LocationTreeNodeKey {
            location: root_location.clone(),
            name: data_set
                .object_name(root_location_path_node_id)
                .as_string()
                .cloned()
                .unwrap_or_default(),
        };

        path_object_stack.pop();

        if let Some(mut tree_node) = self.root_nodes.get_mut(&root_tree_node_key) {
            while let Some(node_object) = path_object_stack.pop() {
                // Unnamed objects can't be paths
                //let node_location = ObjectLocation::new(node_object);
                //let location_chain = data_set.object_location_chain(node_object.path_node_id());

                let node_name = data_set
                    .object_name(node_object.path_node_id())
                    .as_string()
                    .cloned()
                    .unwrap(); //.unwrap_or_else(|| node_object.as_uuid().to_string());

                let node_key = LocationTreeNodeKey {
                    name: node_name,
                    location: node_object.clone(),
                };

                tree_node = tree_node.children.entry(node_key).or_insert_with(|| {
                    //let path = paths.get(&node_object).unwrap().clone();
                    //let node_location = ObjectLocation::new(source, location.parent_tree_node());
                    //let location = ObjectLocation::new(nod)
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
        data_sources: &HashMap<ObjectSourceId, Box<dyn DataSource>>,
        data_set: &DataSet,
        paths: &HashMap<ObjectId, ObjectPath>,
    ) -> Self {
        // Create root nodes for all the data sources
        let mut root_nodes: BTreeMap<LocationTreeNodeKey, LocationTreeNode> = Default::default();
        for (source_id, _data_source) in data_sources {
            let location = ObjectLocation::new(ObjectId::from_uuid(*source_id.uuid()));
            let name = data_set.object_name(location.path_node_id());
            root_nodes.insert(
                LocationTreeNodeKey {
                    location: location.clone(),
                    name: name.as_string().cloned().unwrap_or_default(),
                },
                LocationTreeNode {
                    location,
                    location_root: ObjectLocation::null(),
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

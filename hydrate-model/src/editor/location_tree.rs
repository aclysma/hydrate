use crate::{DataSet, HashMap, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};
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
    pub root_node: LocationTreeNode,
}

impl Default for LocationTree {
    fn default() -> Self {
        LocationTree {
            root_node: LocationTreeNode {
                //source: ObjectSourceId::null(),
                //path: ObjectPath::root(),
                location: ObjectLocation::new(ObjectId::null()),
                location_root: ObjectLocation::new(ObjectId::null()),
                children: Default::default(),
                has_changes: false,
            },
        }
    }
}

impl LocationTree {
    pub fn create_node(
        &mut self,
        data_set: &DataSet,
        paths: &HashMap<ObjectId, ObjectPath>,
        tree_node_id: ObjectId,
    ) {
        // let mut path_object_stack = Vec::default();
        //
        // // Walk up the path, pushing the object id for each path component onto a stack
        // let mut obj_iter = tree_node_id;
        // while !obj_iter.is_null() && !path_object_stack.contains(&obj_iter) {
        //     path_object_stack.push(obj_iter);
        //     obj_iter = if let Some(location) = data_set.object_location(tree_node_id) {
        //         location.path_node_id()
        //     } else {
        //         ObjectId::null()
        //     };
        // }

        let mut path_object_stack = data_set.object_location_chain(tree_node_id);

        let mut tree_node = &mut self.root_node;

        while let Some(node_object) = path_object_stack.pop() {
            // Unnamed objects can't be paths
            //let node_location = ObjectLocation::new(node_object);
            let location_chain = data_set.object_location_chain(node_object.path_node_id());
            let location = location_chain.first().cloned().unwrap_or_else(ObjectLocation::null);
            let location_root = location_chain.last().cloned().unwrap_or_else(ObjectLocation::null);

            let node_name = data_set
                .object_name(node_object.path_node_id())
                .as_string()
                .cloned()
                .unwrap(); //.unwrap_or_else(|| node_object.as_uuid().to_string());

            let node_key = LocationTreeNodeKey {
                name: node_name,
                location: location.clone(),
            };

            tree_node = tree_node.children.entry(node_key).or_insert_with(|| {
                //let path = paths.get(&node_object).unwrap().clone();
                //let node_location = ObjectLocation::new(source, location.parent_tree_node());
                //let location = ObjectLocation::new(nod)
                let has_changes = false; //unsaved_paths.contains(&node_location);
                LocationTreeNode {
                    //path,
                    //source: node_location.source(),
                    location,
                    location_root,
                    children: Default::default(),
                    has_changes,
                }
            });
        }
    }

    pub fn build(
        data_set: &DataSet,
        paths: &HashMap<ObjectId, ObjectPath>,
    ) -> Self {
        //TODO: Change this to use the root objects as nodes

        let root_node = LocationTreeNode {
            //path: ObjectPath::root(),
            //source: ObjectSourceId::null(),
            location: ObjectLocation::null(),
            location_root: ObjectLocation::null(),
            children: Default::default(),
            has_changes: false,
        };

        let mut tree = LocationTree { root_node };

        for (tree_node_id, path) in paths {
            let components = path.split_components();
            if !components.is_empty() {
                //println!("source {:?}", object_location.path());

                // Skip the root component since it is our root node
                tree.create_node(data_set, paths, *tree_node_id);
            }
        }

        tree
    }
}

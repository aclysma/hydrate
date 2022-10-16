use std::collections::BTreeMap;
use crate::{HashSet, ObjectLocation, ObjectPath};

pub struct LocationTreeNodeKey {
    name: String,
}


#[derive(Debug)]
pub struct LocationTreeNode {
    pub path: ObjectPath,
    pub children: BTreeMap<String, LocationTreeNode>,
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
                path: ObjectPath::root(),
                children: Default::default(),
                has_changes: false
            }
        }
    }
}

impl LocationTree {
    fn get_or_create_path(&mut self, path_components: &[&str], unsaved_paths: &HashSet<ObjectPath>) -> &mut LocationTreeNode {
        let mut tree_node = &mut self.root_node;

        let mut node_path = ObjectPath::root();
        for path_component in path_components {
            node_path = node_path.join(path_component);
            tree_node = tree_node.children.entry(path_component.to_string()).or_insert_with(|| {
                LocationTreeNode {
                    path: node_path.clone(),
                    children: Default::default(),
                    has_changes: unsaved_paths.contains(&node_path)
                }
            });

            //assert!(tree_node.is_directory);
        }

        tree_node
    }

    pub fn rebuild(&mut self, object_paths: &HashSet<ObjectPath>, unsaved_paths: &HashSet<ObjectPath>) {
        self.root_node = LocationTreeNode {
            path: ObjectPath::root(),
            children: Default::default(),
            has_changes: false
        };

        for path in object_paths {
            let components = path.split_components();
            if components.len() > 1 {
                // Skip the root component since it is our root node
                self.get_or_create_path(&components[1..], unsaved_paths);
            }
        }
    }
}

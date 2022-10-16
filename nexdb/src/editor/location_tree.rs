use std::collections::BTreeMap;
use crate::{HashSet, ObjectPath};

pub struct LocationTreeNodeKey {
    name: String,
}


#[derive(Debug)]
pub struct LocationTreeNode {
    pub path: ObjectPath,
    pub children: BTreeMap<String, LocationTreeNode>,
    //explicitly_added: bool,
    //is_directory: bool
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
                children: Default::default()
            }
        }
    }
}

impl LocationTree {
    fn get_or_create_path(&mut self, path_components: &[&str]) -> &mut LocationTreeNode {
        let mut tree_node = &mut self.root_node;

        let mut node_path = ObjectPath::root();
        for path_component in path_components {
            node_path = node_path.join(path_component);
            tree_node = tree_node.children.entry(path_component.to_string()).or_insert_with(|| {
                LocationTreeNode {
                    path: node_path.clone(),
                    children: Default::default(),
                }
            });

            //assert!(tree_node.is_directory);
        }

        tree_node
    }

    pub fn rebuild(&mut self, object_paths: &HashSet<ObjectPath>) {
        self.root_node = LocationTreeNode {
            path: ObjectPath::root(),
            children: Default::default()
        };

        for path in object_paths {
            let components = path.split_components();
            if components.len() > 1 {
                // Skip the root component since it is our root node
                self.get_or_create_path(&components[1..]);
            }
        }
    }
}



/*
struct LocationTreeNode {
    path: ObjectPath,
    children: BTreeMap<String, LocationTreeNode>,
    explicitly_added: bool,
    is_directory: bool
}

struct LocationTree {
    root_node: LocationTreeNode,
}

impl LocationTree {
    fn get_path(&mut self, path_segments: &[&str]) -> Option<&mut LocationTreeNode> {
        let mut tree_node = &mut self.root_node;

        for path_component in path_components {
            if let Some(child) = tree_node.children.get_mut(path_component) {
                tree_node = child;
            } else {
                return None;
            }
        }

        Some(tree_node)
    }

    fn get_or_create_path(&mut self, path_segments: &[&str]) -> &mut LocationTreeNode {
        let mut tree_node = &mut self.root_node;

        let mut node_path = ObjectPath::root();
        for path_component in path_components {
            node_path = node_path.join(path_component);
            tree_node = tree_node.children.entry(path_component.to_string()).or_insert_with(|| {
                LocationTreeNode {
                    path: node_path.clone(),
                    children: Default::default(),
                    is_directory: true,
                    explicitly_added: false,
                }
            });

            assert!(tree_node.is_directory);
        }

        tree_node
    }

    fn insert_file(&mut self, path: ObjectPath) {
        let mut path_components = path.split_components();
        let last = path_components.pop();

        let mut tree_node = self.get_or_create_path(&path_components);

        // insert last
        if let Some(last) = last {
            node_path = node_path.join(last);
            tree_node.children.insert(last.to_string(), LocationTreeNode {
                path: node_path,
                children: Default::default(),
                is_directory: false,
                explicitly_added: true
            });
        }
    }

    fn remove_file(&mut self, path: ObjectPath) {
        let mut path_components = path.split_components();
        let last = path_components.pop();

        let mut tree_node = &mut self.root_node;
        for path_component in path_components {
            if let Some(child) = tree_node.children.get_mut(path_component) {
                tree_node = child;
            } else {
                return;
            }
        }

        if let Some(last) = last {
            tree_node.children.remove(last);
        }
    }
}
*/
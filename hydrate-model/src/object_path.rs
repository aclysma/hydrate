// assumed to end with /. We don't just use / to make it clear that it's not a file path
const ROOT_PATH_STR: &str = "db:/";
const ROOT_PATH: ObjectPath = ObjectPath(None);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPath(Option<String>);

impl ObjectPath {
    pub fn new(s: &str) -> Self {
        // We assume all paths are absolute
        if !s.starts_with(ROOT_PATH_STR) {
            panic!("Invalid object path str");
        }

        if s.len() == ROOT_PATH_STR.len() {
            ObjectPath(None)
        } else {
            ObjectPath(Some(s.to_string()))
        }
    }

    pub fn root_ref() -> &'static Self {
        &ROOT_PATH
    }

    pub fn root() -> Self {
        ObjectPath(None)
    }

    pub fn join(
        &self,
        rhs: &str,
    ) -> ObjectPath {
        if rhs.is_empty() {
            return self.clone();
        }

        // Joining an absolute path to an absolute path is not allowed
        assert!(!rhs.starts_with(ROOT_PATH_STR));
        assert!(!rhs.contains("/"));

        match &self.0 {
            Some(x) => {
                if x.ends_with("/") {
                    ObjectPath(Some(format!("{}{}", x, rhs)))
                } else {
                    ObjectPath(Some(format!("{}/{}", x, rhs)))
                }
            }
            None => ObjectPath(Some(format!("{}{}", ROOT_PATH_STR, rhs))),
        }
    }

    // pub fn strip_prefix(
    //     &self,
    //     prefix: &ObjectPath,
    // ) -> Option<ObjectPath> {
    //     match self.0 {
    //         Some(x) => {
    //             x.strip_prefix(&prefix.0).ma
    //         }
    //     }
    //
    //
    //     self.0.as_ref().unwrap_or(ROOT_PATH_STR)
    //         .strip_prefix(&prefix.0)
    //         .map(|x| ObjectPath(x.to_string()))
    // }

    // pub fn parent_path(&self) -> Option<Self> {
    //     match &self.0 {
    //         None => None, // Parent of root path does not exist
    //         Some(path) => {
    //             if let Some(index) = path.rfind("/") {
    //                 if index >= ROOT_PATH_STR.len() {
    //                     // We have a parent path that isn't root
    //                     Some(ObjectPath(Some(path[0..index].to_string())))
    //                 } else {
    //                     // Parent path is root
    //                     Some(ObjectPath(None))
    //                 }
    //             } else {
    //                 // Path with no slash should not exist
    //                 unimplemented!()
    //             }
    //         }
    //     }
    // }

    pub fn parent_path_and_name(&self) -> Option<(Self, String)> {
        match &self.0 {
            None => None, // Parent of root path does not exist
            Some(path) => {
                if let Some(index) = path.rfind("/") {
                    if index >= ROOT_PATH_STR.len() {
                        // We have a parent path that isn't root
                        let parent = ObjectPath(Some(path[0..index].to_string()));
                        let name = path[index + 1..].to_string();
                        Some((parent, name))
                    } else {
                        // Parent path is root
                        let parent = ObjectPath(None);
                        let name = path[ROOT_PATH_STR.len()..].to_string();
                        Some((parent, name))
                    }
                } else {
                    // Path with no slash should not exist
                    unimplemented!()
                }
            }
        }
    }

    pub fn is_root_path(&self) -> bool {
        return self.0.is_none();
    }

    pub fn split_components(&self) -> Vec<&str> {
        match &self.0 {
            Some(x) => x.split("/").skip(1).collect(),
            None => vec![],
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref().map(|x| x.as_str()).unwrap_or(ROOT_PATH_STR)
    }

    pub fn starts_with(
        &self,
        other: &ObjectPath,
    ) -> bool {
        self.as_str().starts_with(other.as_str())
    }
}

impl From<&str> for ObjectPath {
    fn from(s: &str) -> Self {
        ObjectPath::new(s)
    }
}

impl From<String> for ObjectPath {
    fn from(s: String) -> Self {
        ObjectPath::new(&s)
    }
}

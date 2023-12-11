use crate::ImportableName;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use hydrate_schema::{DataSetError, DataSetResult};

pub trait PathReferenceNamespaceResolver {
    // Given the namespace, return the path associated with it
    fn namespace_root(&self, namespace: &str) -> Option<PathBuf>;

    // Given the canonicalized absolute path, if it can be expressed as a namespace and path within the namepsace, return that
    fn simplify_path(&self, path: &Path) -> Option<(String, PathBuf)>;
}


//TODO: I want to change this to be optionally using something like db:/ with the data source name instead of "db"
//TODO: Remove PartialEq, Eq, Hash
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathReference {
    namespace: String,
    path: String,
    importable_name: ImportableName,
}

impl Display for PathReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.namespace.is_empty() {
            if let Some(importable_name) = self.importable_name.name() {
                write!(f, "{}#{}", self.path, importable_name)
            } else {
                write!(f, "{}", self.path)
            }
        } else {
            if let Some(importable_name) = self.importable_name.name() {
                write!(f, "{}://{}#{}", self.namespace, self.path, importable_name)
            } else {
                write!(f, "{}://{}", self.namespace, self.path)
            }
        }
    }
}

impl PathReference {
    pub fn new(
        namespace: String,
        path: String,
        importable_name: ImportableName,
    ) -> Self {
        PathReference {
            namespace,
            path,
            importable_name,
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn importable_name(&self) -> &ImportableName {
        &self.importable_name
    }

    pub fn canonicalized_absolute_path(
        &self,
        namespace_resolver: &dyn PathReferenceNamespaceResolver,
        source_file_path: &Path,
    ) -> DataSetResult<PathReference> {
        let canonical_absolute_path = if self.namespace.is_empty() {
            if Path::new(&self.path).is_relative() {
                source_file_path
                    .parent()
                    .unwrap()
                    .join(Path::new(&self.path))
                    .canonicalize()
                    .unwrap()
            } else {
                PathBuf::from(&self.path).canonicalize().unwrap()
            }
        } else {
            let namespace_root = namespace_resolver.namespace_root(&self.namespace).ok_or(DataSetError::UnknownPathNamespace)?;
            namespace_root.join(&self.path).canonicalize().unwrap()
        };

        Ok(PathReference {
            namespace: "".to_string(),
            path: canonical_absolute_path.to_string_lossy().to_string(),
            importable_name: self.importable_name.clone(),
        })
    }

    pub fn simplify(
        self,
        namespace_resolver: &dyn PathReferenceNamespaceResolver
    ) -> PathReference {
        if !self.namespace.is_empty() {
            // If it has a namespace it can't be simplified
            self
        } else if Path::new(&self.path).is_relative() {
            // If it's a relative path it can't be simplified
            self
        } else {
            // If it's an absolute path, see if it is in a namespace, if it is, we can return a PathReference relative
            // to the namespace
            let canonicalized_path = PathBuf::from(&self.path).canonicalize().unwrap();

            if let Some((namespace, prefix)) = namespace_resolver.simplify_path(&canonicalized_path) {
                PathReference {
                    namespace,
                    path: prefix.to_string_lossy().to_string(),
                    importable_name: self.importable_name
                }
            } else {
                self
            }
        }
    }
}

impl From<&str> for PathReference {
    fn from(s: &str) -> PathReference {
        let namespace_delimeter_position = s.rfind("://");
        let importable_name_delimeter_position = s.rfind('#');

        let (path_start_position, namespace) = if let Some(namespace_delimeter_position) = namespace_delimeter_position {
            (namespace_delimeter_position + 3, s[0..namespace_delimeter_position].to_string())
        } else {
            (0, String::default())
        };

        let (path, importable_name) = if let Some(importable_name_delimeter_position) = importable_name_delimeter_position {
            let path = s[path_start_position..importable_name_delimeter_position].to_string();
            let importable_name = &s[importable_name_delimeter_position + 1..];
            let importable_name = if !importable_name.is_empty() {
                ImportableName::new(importable_name.to_string())
            } else {
                ImportableName::default()
            };
            (path, importable_name)
        } else {
            (s[path_start_position..].to_string(), ImportableName::default())
        };

        PathReference {
            namespace,
            path,
            importable_name
        }
    }
}

impl From<String> for PathReference {
    fn from(path: String) -> PathReference {
        let str: &str = &path;
        PathReference::from(str)
    }
}

impl From<&String> for PathReference {
    fn from(path: &String) -> PathReference {
        let str: &str = &path;
        PathReference::from(str)
    }
}

impl From<&Path> for PathReference {
    fn from(path: &Path) -> PathReference {
        let str: &str = path.to_str().unwrap();
        PathReference::from(str)
    }
}

impl From<&PathBuf> for PathReference {
    fn from(path: &PathBuf) -> PathReference {
        let str: &str = path.to_str().unwrap();
        PathReference::from(str)
    }
}

impl From<PathBuf> for PathReference {
    fn from(path: PathBuf) -> PathReference {
        let str: &str = path.to_str().unwrap();
        PathReference::from(str)
    }
}

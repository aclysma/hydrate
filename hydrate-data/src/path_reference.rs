use crate::ImportableName;
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathReference {
    pub path: String,
    pub importable_name: ImportableName,
}

impl PathReference {
    pub fn new(path: String, importable_name: ImportableName) -> Self {
        PathReference {
            path,
            importable_name
        }
    }

    pub fn is_relative(&self) -> bool {
        Path::new(&self.path).is_relative()
    }

    pub fn canonicalize_relative(
        source_file_path: &Path,
        referenced: &PathReference,
    ) -> PathReference {
        let referenced_file_absolute_path = if Path::new(&referenced.path).is_relative() {
            source_file_path
                .parent()
                .unwrap()
                .join(Path::new(&referenced.path))
                .canonicalize()
                .unwrap()
        } else {
            PathBuf::from(&referenced.path)
        };

        PathReference {
            path: referenced_file_absolute_path.to_string_lossy().to_string(),
            importable_name: referenced.importable_name.clone(),
        }
    }
}

impl From<&str> for PathReference {
    fn from(s: &str) -> PathReference {
        let delimeter_position = s.rfind('#');
        if let Some(delimeter_position) = delimeter_position {
            let path = s[..delimeter_position].to_string();
            let name = &s[delimeter_position + 1..];
            let importable_name = if !name.is_empty() {
                ImportableName::new(name.to_string())
            } else {
                ImportableName::default()
            };

            PathReference {
                path,
                importable_name,
            }
        } else {
            PathReference {
                path: s.to_string(),
                importable_name: ImportableName::default(),
            }
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

impl Display for PathReference {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let str = if let Some(importable_name) = &self.importable_name.name() {
            format!("{}#{}", self.path, importable_name)
        } else {
            self.path.clone()
        };
        write!(f, "{}", str)
    }
}

use std::error::Error;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use hydrate_data::PathReferenceNamespaceResolver;
use hydrate_schema::SchemaCacheSingleFile;

#[derive(Serialize, Deserialize)]
pub struct NamePathPairJson {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SchemaCodegenJobsJson {
    name: String,
    schema_path: String,
    included_schema_paths: Vec<String>,
    outfile: String,
}

#[derive(Serialize, Deserialize)]
pub struct HydrateProjectConfigurationJson {
    pub schema_def_paths: Vec<String>,
    pub schema_cache_file_path: String,
    pub import_data_path: String,
    pub build_data_path: String,
    pub job_data_path: String,
    pub id_based_asset_sources: Vec<NamePathPairJson>,
    pub path_based_asset_sources: Vec<NamePathPairJson>,
    pub source_file_locations: Vec<NamePathPairJson>,
    pub schema_codegen_jobs: Vec<SchemaCodegenJobsJson>,
}

#[derive(Debug, Clone)]
pub struct NamePathPair {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SchemaCodegenJobs {
    pub name: String,
    pub schema_path: PathBuf,
    pub included_schema_paths: Vec<PathBuf>,
    pub outfile: PathBuf,
}

#[derive(Debug, Clone)]
pub struct HydrateProjectConfiguration {
    // Directories to all schema files that should be used
    pub schema_def_paths: Vec<PathBuf>,

    // Path to a json file used to cache every schema that ever existed
    // Unlike the other paths, this is a path to a FILE (hence the name "*_file_path)
    pub schema_cache_file_path: PathBuf,

    // Path to where all import data will be stored (this is bulk data extracted from source files)
    pub import_data_path: PathBuf,

    // Path to where all built data will be stored (this is what the game consumes)
    pub build_data_path: PathBuf,

    // Unused for now, but it will be a cache for intermediate build data later
    pub job_data_path: PathBuf,

    // Asset storage location that uses file system paths for names/asset references
    pub id_based_asset_sources: Vec<NamePathPair>,
    // Asset storage location that uses IDs for file names/asset references
    pub path_based_asset_sources: Vec<NamePathPair>,
    // When importing data, if it is coming from within one of these paths on disk the location of
    // the source file will be tracked relative to that path
    pub source_file_locations: Vec<NamePathPair>,

    pub schema_codegen_jobs: Vec<SchemaCodegenJobs>,
}

impl PathReferenceNamespaceResolver for HydrateProjectConfiguration {
    fn namespace_root(&self, namespace: &str) -> Option<PathBuf> {
        for src in &self.id_based_asset_sources {
            if src.name == namespace {
                return Some(src.path.clone());
            }
        }

        for src in &self.path_based_asset_sources {
            if src.name == namespace {
                return Some(src.path.clone());
            }
        }

        for src in &self.source_file_locations {
            if src.name == namespace {
                return Some(src.path.clone());
            }
        }

        None
    }

    fn simplify_path(&self, path: &Path) -> Option<(String, PathBuf)> {
        for src in &self.id_based_asset_sources {
            if let Ok(path) = path.strip_prefix(&src.path) {
                return Some((src.name.clone(), path.to_path_buf()));
            }
        }

        for src in &self.path_based_asset_sources {
            if let Ok(path) = path.strip_prefix(&src.path) {
                return Some((src.name.clone(), path.to_path_buf()));
            }
        }

        for src in &self.source_file_locations {
            if let Ok(path) = path.strip_prefix(&src.path) {
                return Some((src.name.clone(), path.to_path_buf()));
            }
        }

        None
    }
}

impl HydrateProjectConfiguration {
    pub fn unverified_absolute_path(root_path: &Path, json_path: &str) -> PathBuf {
        if Path::new(json_path).is_absolute() {
            PathBuf::from(json_path)
        } else {
            root_path.join(json_path)
        }
    }

    // root_path is the path the json file is in, json_path is the string in json that is meant
    // to be parsed/converted to a canonicalized path
    pub fn parse_schema_cache_file_path(root_path: &Path, json_path: &str) -> Result<PathBuf, Box<dyn Error>> {
        // If it's not an absolute path, join it onto the path containing the project file
        let joined_path = Self::unverified_absolute_path(root_path, json_path);

        // Create an empty file (and its parent dirs) if it doesn't exist
        if !joined_path.exists() {
            // Create the containing dir
            let parent_path = joined_path.parent().ok_or_else(|| "Parent of project file path could not be found".to_string())?;
            std::fs::create_dir_all(parent_path)?;

            // Create the file, it has to exist in order to get the canonical path. Since it will be a json
            // file that's an array of cached types, fill it with "[]"
            let default_contents = serde_json::to_string_pretty(&SchemaCacheSingleFile::default())?;
            std::fs::write(&joined_path, default_contents)?;
        }

        // Canonicalize the path
        Ok(dunce::canonicalize(&joined_path).map_err(|e| e.to_string())?)
    }

    // root_path is the path the json file is in, json_path is the string in json that is meant
    // to be parsed/converted to a canonicalized path
    pub fn parse_dir_path(root_path: &Path, json_path: &str) -> Result<PathBuf, Box<dyn Error>> {
        // If it's not an absolute path, join it onto the path containing the project file
        let joined_path = Self::unverified_absolute_path(root_path, json_path);

        // Create the dir (and it's parent dirs) if it doesn't exist
        if !joined_path.exists() {
            std::fs::create_dir_all(&joined_path)?;
        }

        // Canonicalize the path
        Ok(dunce::canonicalize(&joined_path).map_err(|e| e.to_string())?)
    }

    pub fn read_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        let root_path = dunce::canonicalize(path.parent().ok_or_else(|| "Parent of project file path could not be found".to_string())?)?;
        let file_contents = std::fs::read_to_string(path)?;
        let project_file: HydrateProjectConfigurationJson = serde_json::from_str(&file_contents)?;

        let schema_cache_file_path = Self::parse_schema_cache_file_path(&root_path, &project_file.schema_cache_file_path)?;
        let import_data_path = Self::parse_dir_path(&root_path, &project_file.import_data_path)?;
        let build_data_path = Self::parse_dir_path(&root_path, &project_file.build_data_path)?;
        let job_data_path = Self::parse_dir_path(&root_path, &project_file.job_data_path)?;

        let mut schema_def_paths = Vec::default();
        for path in &project_file.schema_def_paths {
            schema_def_paths.push(Self::parse_dir_path(&root_path, path)?)
        }

        let mut id_based_asset_sources = Vec::default();
        for pair in project_file.id_based_asset_sources {
            id_based_asset_sources.push(NamePathPair {
                name: pair.name,
                path: Self::parse_dir_path(&root_path, &pair.path)?
            });
        }

        let mut path_based_asset_sources = Vec::default();
        for pair in project_file.path_based_asset_sources {
            path_based_asset_sources.push(NamePathPair {
                name: pair.name,
                path: Self::parse_dir_path(&root_path, &pair.path)?
            });
        }

        let mut source_file_locations = Vec::default();
        for pair in project_file.source_file_locations {
            source_file_locations.push(NamePathPair {
                name: pair.name,
                path: Self::parse_dir_path(&root_path, &pair.path)?
            });
        }

        // We don't canonicalize/verify the codegen paths
        let mut schema_codegen_jobs = Vec::default();
        for schema_codegen_job in project_file.schema_codegen_jobs {
            let mut included_schema_paths = Vec::default();
            for included_schema_path in &schema_codegen_job.included_schema_paths {
                included_schema_paths.push(Self::unverified_absolute_path(&root_path, included_schema_path));
            }

            schema_codegen_jobs.push(SchemaCodegenJobs {
                name: schema_codegen_job.name,
                schema_path: Self::unverified_absolute_path(&root_path, &schema_codegen_job.schema_path),
                included_schema_paths,
                outfile: Self::unverified_absolute_path(&root_path, &schema_codegen_job.outfile),
            })
        }

        Ok(HydrateProjectConfiguration {
            schema_def_paths,
            schema_cache_file_path,
            import_data_path,
            build_data_path,
            job_data_path,
            id_based_asset_sources,
            path_based_asset_sources,
            source_file_locations,
            schema_codegen_jobs
        })
    }

    pub fn locate_project_file(search_location: &Path) -> Result<Self, Box<dyn Error>> {
        let mut path = Some(search_location.to_path_buf());
        while let Some(p) = path {
            let joined_path = p.join("hydrate_project.json");
            if joined_path.exists() {
                log::info!("Using project configuration at {:?}", joined_path);
                return Self::read_from_path(&joined_path);
            }

            path = p.parent().map(|x| x.to_path_buf());
        }

        Err(format!("hydrate_project.json could not be located at {:?} or in any of its parent directories", search_location))?
    }
}

use std::hash::{Hash, Hasher};
use std::path::Path;
use siphasher::sip128::Hasher128;
use uuid::Uuid;
use super::{SchemaFromFileTypeRef, SchemaFromFileNamedType, SchemaFromFileDynamicArray, SchemaFromFileRecordField, SchemaFromFileRecord};
use crate::{HashMap, HashSet, SchemaFromFileParserError};

#[derive(Debug)]
pub enum SchemaLoaderError {
    Str(&'static str),
    String(String),
}

impl From<SchemaFromFileParserError> for SchemaLoaderError {
    fn from(err: SchemaFromFileParserError) -> Self {
        match err {
            SchemaFromFileParserError::Str(x) => SchemaLoaderError::Str(x),
            SchemaFromFileParserError::String(x) => SchemaLoaderError::String(x)
        }
    }
}

pub type SchemaLoaderResult<T> = Result<T, SchemaLoaderError>;


#[derive(Default)]
pub struct SchemaLoader {
    types: HashMap<String, SchemaFromFileNamedType>,
    type_aliases: HashMap<String, String>,
    //records: Vec<SchemaFromFileRecord>,
    // enums
    // fixed
    // union?
}

impl SchemaLoader {
    pub fn add_source_dir<PathT: AsRef<Path>, PatternT: AsRef<str>>(&mut self, path: PathT, pattern: PatternT) -> SchemaLoaderResult<()> {
        log::info!("Adding schema source dir {:?} with pattern {:?}", path.as_ref(), pattern.as_ref());
        let walker = globwalk::GlobWalkerBuilder::new(path.as_ref(), pattern.as_ref())
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            let file = file.unwrap();
            log::debug!("Parsing schema file {}", file.path().display());
            let schema_str = std::fs::read_to_string(file.path()).unwrap();
            let json_value: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
            //println!("VALUE {:#?}", value);

            let json_objects = json_value.as_array().ok_or_else(|| SchemaLoaderError::Str("Schema file must be an array of json objects"))?;

            for json_object in json_objects {
                let named_type = super::schema_from_file::parse_json_schema(&json_object, &format!("[{}]", file.path().display()))?;

                if self.types.contains_key(named_type.type_name()) {
                    Err(SchemaLoaderError::String(format!("A type named {} has already been parsed", named_type.type_name())))?;
                }

                if self.type_aliases.contains_key(named_type.type_name()) {
                    Err(SchemaLoaderError::String(format!("A type named {} has already been parsed", named_type.type_name())))?;
                }

                for alias in named_type.aliases() {
                    if self.types.contains_key(alias) {
                        Err(SchemaLoaderError::String(format!("A type named {} has already been parsed", alias)))?;
                    }

                    if self.type_aliases.contains_key(alias) {
                        Err(SchemaLoaderError::String(format!("A type named {} has already been parsed", alias)))?;
                    }
                }

                for alias in named_type.aliases() {
                    self.type_aliases.insert(alias.to_string(), named_type.type_name().to_string());
                }
                self.types.insert(named_type.type_name().to_string(), named_type);
            }
        }

        Ok(())
    }

    pub fn finish(&mut self) {
        // Apply aliases
        for (_, named_type) in &mut self.types {
            named_type.apply_type_aliases(&self.type_aliases);
        }

        let mut partial_hashes = HashMap::default();
        for (type_name, named_type) in &self.types {
            let mut hasher = siphasher::sip128::SipHasher::default();
            named_type.partial_hash(&mut hasher);
            let partial_fingerprint = hasher.finish128().as_u128();
            partial_hashes.insert(type_name, partial_fingerprint);
        }

        // Hash each thing
        for (type_name, named_type) in &self.types {
            //TODO: Continue calling collect on all types in list until no new types are added?
            let mut related_types = HashSet::default();
            related_types.insert(type_name.clone());

            loop {
                // We make a copy because otherwise we would be iterating the HashSet while we are appending to it
                let before_copy: Vec<_> = related_types.iter().cloned().collect();
                for related_type in &before_copy {
                    let related_type = self.types.get(related_type).unwrap();
                    related_type.collect_all_related_types(&mut related_types);
                }

                if before_copy.len() == related_types.len() {
                    break;
                }
            }

            named_type.collect_all_related_types(&mut related_types);

            println!("{} relies on {:?}", named_type.type_name(), related_types);

            let mut related_types: Vec<_> = related_types.into_iter().collect();
            related_types.sort();

            let mut hasher = siphasher::sip128::SipHasher::default();
            for related_type in &related_types {
                //let related_type = self.types.get(related_type);
                let partial_hash = partial_hashes.get(related_type).unwrap();
                partial_hash.hash(&mut hasher);
            }
            let fingerprint = hasher.finish128().as_u128();

            println!("type {} fingerprint is {}", type_name, fingerprint);
        }
    }
}

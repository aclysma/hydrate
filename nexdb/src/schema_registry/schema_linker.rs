use super::enum_type_builder::*;
use super::fixed_type_builder::*;
use super::record_type_builder::*;
use super::schema_def::*;
use crate::{HashMap, HashSet, SchemaDefParserError, SchemaFingerprint, SchemaNamedType};
use siphasher::sip128::Hasher128;
use std::hash::{Hash, Hasher};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug)]
pub enum SchemaLinkerError {
    Str(&'static str),
    String(String),
    ValidationError(SchemaDefValidationError),
}

impl From<SchemaDefParserError> for SchemaLinkerError {
    fn from(err: SchemaDefParserError) -> Self {
        match err {
            SchemaDefParserError::Str(x) => SchemaLinkerError::Str(x),
            SchemaDefParserError::String(x) => SchemaLinkerError::String(x),
            SchemaDefParserError::ValidationError(x) => SchemaLinkerError::ValidationError(x),
        }
    }
}

impl From<SchemaDefValidationError> for SchemaLinkerError {
    fn from(err: SchemaDefValidationError) -> Self {
        SchemaLinkerError::ValidationError(err)
    }
}

pub type SchemaLinkerResult<T> = Result<T, SchemaLinkerError>;

#[derive(Default)]
pub struct SchemaLinker {
    types: HashMap<String, SchemaDefNamedType>,
    type_aliases: HashMap<String, String>,
    //records: Vec<SchemaFromFileRecord>,
    // enums
    // fixed
    // union?
}

impl SchemaLinker {
    fn add_named_type(
        &mut self,
        named_type: SchemaDefNamedType,
    ) -> SchemaLinkerResult<()> {
        log::debug!("Adding type {}", named_type.type_name());
        if self.types.contains_key(named_type.type_name()) {
            Err(SchemaLinkerError::String(format!(
                "Type name {} has already been used",
                named_type.type_name()
            )))?;
        }

        if self.type_aliases.contains_key(named_type.type_name()) {
            Err(SchemaLinkerError::String(format!(
                "Type name {} has already been used",
                named_type.type_name()
            )))?;
        }

        for alias in named_type.aliases() {
            if self.types.contains_key(alias) {
                Err(SchemaLinkerError::String(format!(
                    "Type name {} has already been used",
                    alias
                )))?;
            }

            if self.type_aliases.contains_key(alias) {
                Err(SchemaLinkerError::String(format!(
                    "Type name {} has already been used",
                    alias
                )))?;
            }
        }

        for alias in named_type.aliases() {
            self.type_aliases
                .insert(alias.to_string(), named_type.type_name().to_string());
        }
        //let schema_def = SchemaDefType::NamedType(named_type.type_name().to_string());
        self.types
            .insert(named_type.type_name().to_string(), named_type);
        Ok(())
    }

    pub fn add_source_dir<PathT: AsRef<Path>, PatternT: AsRef<str>>(
        &mut self,
        path: PathT,
        pattern: PatternT,
    ) -> SchemaLinkerResult<()> {
        log::info!(
            "Adding schema source dir {:?} with pattern {:?}",
            path.as_ref(),
            pattern.as_ref()
        );
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

            let json_objects = json_value.as_array().ok_or_else(|| {
                SchemaLinkerError::Str("Schema file must be an array of json objects")
            })?;

            for json_object in json_objects {
                let named_type = super::schema_def::parse_json_schema_def(
                    &json_object,
                    &format!("[{}]", file.path().display()),
                )?;
                self.add_named_type(named_type)?;
            }
        }

        Ok(())
    }

    pub fn register_record_type<F: Fn(&mut RecordTypeBuilder)>(
        &mut self,
        name: impl Into<String>,
        f: F,
    ) -> SchemaLinkerResult<()> {
        let mut builder = RecordTypeBuilder::default();
        (f)(&mut builder);

        let mut fields = Vec::with_capacity(builder.fields.len());
        for builder_field in builder.fields {
            fields.push(SchemaDefRecordField::new(
                builder_field.name,
                builder_field.aliases,
                builder_field.field_type,
            )?);
        }

        let name = name.into();
        let schema_record = SchemaDefRecord::new(name.clone(), builder.aliases, fields)?;
        let named_type = SchemaDefNamedType::Record(schema_record);
        self.add_named_type(named_type)
    }

    pub fn register_enum_type<F: Fn(&mut EnumTypeBuilder)>(
        &mut self,
        name: impl Into<String>,
        f: F,
    ) -> SchemaLinkerResult<()> {
        let mut builder = EnumTypeBuilder::default();
        (f)(&mut builder);

        let mut symbols = Vec::with_capacity(builder.symbols.len());
        for builder_field in builder.symbols {
            symbols.push(SchemaDefEnumSymbol::new(
                builder_field.name,
                builder_field.aliases,
                builder_field.value,
            )?);
        }

        symbols.sort_by_key(|x| x.value);

        let name = name.into();
        let schema_enum = SchemaDefEnum::new(name.clone(), builder.aliases, symbols)?;

        let named_type = SchemaDefNamedType::Enum(schema_enum);
        self.add_named_type(named_type)
    }

    pub fn register_fixed_type<F: Fn(&mut FixedTypeBuilder)>(
        &mut self,
        name: impl Into<String>,
        length: usize,
        f: F,
    ) -> SchemaLinkerResult<()> {
        let mut builder = FixedTypeBuilder::default();
        (f)(&mut builder);

        let name = name.into();
        let schema_fixed = SchemaDefFixed::new(name.clone(), builder.aliases, length)?;

        let named_type = SchemaDefNamedType::Fixed(schema_fixed);
        self.add_named_type(named_type)
    }

    pub(crate) fn finish(mut self) -> SchemaLinkerResult<LinkedSchemas> {
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

        let mut schemas_by_name: HashMap<String, SchemaFingerprint> = Default::default();
        let mut schemas: HashMap<SchemaFingerprint, SchemaNamedType> = Default::default();

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

            let mut related_types: Vec<_> = related_types.into_iter().collect();
            related_types.sort();

            let mut hasher = siphasher::sip128::SipHasher::default();
            for related_type in &related_types {
                //let related_type = self.types.get(related_type);
                let partial_hash = partial_hashes.get(related_type).unwrap();
                partial_hash.hash(&mut hasher);
            }
            let fingerprint = SchemaFingerprint(hasher.finish128().as_u128());

            log::debug!("type {} fingerprint is {}", type_name, fingerprint.as_uuid());
            schemas_by_name.insert(type_name.to_string(), fingerprint);
        }

        for (_type_name, named_type) in self.types {
            let fingerprint = schemas_by_name.get(named_type.type_name()).unwrap();
            let schema = named_type.to_schema(&schemas_by_name);
            schemas.insert(*fingerprint, schema);
        }

        Ok(LinkedSchemas {
            schemas_by_name,
            schemas,
        })
    }
}

pub(crate) struct LinkedSchemas {
    pub(crate) schemas_by_name: HashMap<String, SchemaFingerprint>,
    pub(crate) schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
}

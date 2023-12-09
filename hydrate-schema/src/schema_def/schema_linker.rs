use super::enum_type_builder::*;
use super::record_type_builder::*;
use super::schema_def::*;
use crate::{HashMap, HashSet, SchemaDefParserError, SchemaFingerprint, SchemaNamedType};
use siphasher::sip128::Hasher128;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::path::Path;

#[derive(Debug)]
pub enum SchemaLinkerError {
    Str(&'static str),
    String(String),
    ValidationError(SchemaDefValidationError),
}

impl Display for SchemaLinkerError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "Error linking schema: {:?}", self)
    }
}

impl Error for SchemaLinkerError {}

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

/// Acccumulates schema definitions defined in code or by json. Once schemas have been loaded, they
/// are "linked", producing read-only schemas that are hashed and may cyclically reference each
/// other. The individual schemas are also very cheap to clone as they are stored in Arc<T>s.
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
    pub fn unlinked_type_names(&self) -> Vec<String> {
        self.types.keys().cloned().collect()
    }

    fn add_named_type(
        &mut self,
        named_type: SchemaDefNamedType,
    ) -> SchemaLinkerResult<()> {
        log::trace!("Adding type {}", named_type.type_name());
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
            log::trace!("Parsing schema file {}", file.path().display());
            let schema_str = std::fs::read_to_string(file.path()).unwrap();
            let json_value: serde_json::Value = {
                profiling::scope!("serde_json::from_str");
                serde_json::from_str(&schema_str).unwrap()
            };
            //println!("VALUE {:#?}", value);

            let json_objects = json_value.as_array().ok_or_else(|| {
                SchemaLinkerError::Str("Schema file must be an array of json objects")
            })?;

            for json_object in json_objects {
                let named_type = super::json_schema::parse_json_schema_def(
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
                builder_field.markup,
            )?);
        }

        let name = name.into();
        let schema_record =
            SchemaDefRecord::new(name.clone(), builder.aliases, fields, builder.markup)?;
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
            )?);
        }

        symbols.sort_by(|a, b| a.symbol_name.cmp(&b.symbol_name));

        let name = name.into();
        let schema_enum = SchemaDefEnum::new(name.clone(), builder.aliases, symbols)?;

        let named_type = SchemaDefNamedType::Enum(schema_enum);
        self.add_named_type(named_type)
    }

    fn validate_schema(
        schema: &SchemaDefType,
        named_types: &HashMap<String, SchemaDefNamedType>,
        validated_types: &mut HashSet<String>,
    ) -> SchemaLinkerResult<()> {
        match schema {
            // For nullables we just need to make sure their inner type is validated
            SchemaDefType::Nullable(def) => {
                Self::validate_schema(&*def, named_types, validated_types)
            }
            // These value types don't need any validation
            SchemaDefType::Boolean => Ok(()),
            SchemaDefType::I32 => Ok(()),
            SchemaDefType::I64 => Ok(()),
            SchemaDefType::U32 => Ok(()),
            SchemaDefType::U64 => Ok(()),
            SchemaDefType::F32 => Ok(()),
            SchemaDefType::F64 => Ok(()),
            SchemaDefType::Bytes => Ok(()),
            SchemaDefType::String => Ok(()),
            // For arrays we just need to make sure their inner type is validated
            SchemaDefType::StaticArray(def) => {
                Self::validate_schema(&*def.item_type, named_types, validated_types)
            }
            SchemaDefType::DynamicArray(def) => {
                Self::validate_schema(&*def.item_type, named_types, validated_types)
            }
            // For maps we need to validate the key/value types, and that the key type is allowed to be used as a key
            SchemaDefType::Map(def) => {
                // If we update this, update the similar logic in parse_json_schema_type_ref()
                match &*def.key_type {
                    SchemaDefType::Boolean
                    | SchemaDefType::I32
                    | SchemaDefType::I64
                    | SchemaDefType::U32
                    | SchemaDefType::U64
                    | SchemaDefType::String
                    | SchemaDefType::AssetRef(_) => {
                        // valid keys
                        Ok(())
                    }
                    SchemaDefType::Nullable(_)
                    | SchemaDefType::F32
                    | SchemaDefType::F64
                    | SchemaDefType::Bytes
                    | SchemaDefType::StaticArray(_)
                    | SchemaDefType::DynamicArray(_)
                    | SchemaDefType::Map(_) => {
                        // Invalid schema, we don't support these types as keys
                        Err(SchemaDefValidationError::InvalidMapKeyType)
                    }
                    SchemaDefType::NamedType(key_named_type) => {
                        match named_types.get(key_named_type) {
                            Some(SchemaDefNamedType::Record(_)) => {
                                // Records are not valid map key types
                                Err(SchemaDefValidationError::InvalidMapKeyType).into()
                            }
                            Some(SchemaDefNamedType::Enum(_)) => {
                                // Enums are ok as map key types
                                Ok(())
                            }
                            None => {
                                // Could not find the referenced named type
                                Err(SchemaDefValidationError::ReferencedNamedTypeNotFound).into()
                            }
                        }
                    }
                }?;
                Self::validate_schema(&*def.value_type, named_types, validated_types)?;
                Self::validate_schema(&*def.value_type, named_types, validated_types)?;
                Ok(())
            }
            // For assets we verify they point at a record
            SchemaDefType::AssetRef(def) => {
                match named_types.get(def) {
                    Some(SchemaDefNamedType::Record(_)) => {
                        // Asset ref points to a record in the named_types map, we're good
                        Ok(())
                    }
                    Some(SchemaDefNamedType::Enum(_)) => {
                        // Asset refs can't point at enums
                        Err(SchemaDefValidationError::InvalidAssetRefInnerType.into())
                    }
                    None => Err(SchemaDefValidationError::ReferencedNamedTypeNotFound.into()),
                }
            }
            // For named types, we validate the fields. However, we need to handle cyclical references between types
            SchemaDefType::NamedType(type_name) => {
                // Handle cyclical type references
                if validated_types.contains(type_name) {
                    return Ok(());
                }
                validated_types.insert(type_name.clone());

                match named_types.get(type_name) {
                    Some(SchemaDefNamedType::Record(def)) => {
                        // Validate field types
                        for field_def in def.fields() {
                            Self::validate_schema(
                                &field_def.field_type,
                                named_types,
                                validated_types,
                            )?;
                        }
                        Ok(())
                    }
                    Some(SchemaDefNamedType::Enum(def)) => Ok(()),
                    None => Err(SchemaDefValidationError::ReferencedNamedTypeNotFound.into()),
                }
            }
        }
    }

    pub fn link_schemas(mut self) -> SchemaLinkerResult<LinkedSchemas> {
        // Apply aliases
        for (_, named_type) in &mut self.types {
            named_type.apply_type_aliases(&self.type_aliases);
        }

        let mut validated_types = Default::default();
        for (_, named_type) in &self.types {
            Self::validate_schema(
                &SchemaDefType::NamedType(named_type.type_name().to_string()),
                &self.types,
                &mut validated_types,
            )?;
        }

        let mut partial_hashes = HashMap::default();
        for (type_name, named_type) in &self.types {
            let mut hasher = siphasher::sip128::SipHasher::default();
            //println!("partial hash {}", named_type.type_name());
            named_type.partial_hash(&mut hasher);
            let partial_fingerprint = hasher.finish128().as_u128();
            partial_hashes.insert(type_name, partial_fingerprint);
        }

        let mut schemas_by_name: HashMap<String, SchemaFingerprint> = Default::default();
        let mut schemas: HashMap<SchemaFingerprint, SchemaNamedType> = Default::default();

        // Hash each thing
        for (type_name, named_type) in &self.types {
            let mut related_types = HashSet::default();
            related_types.insert(type_name.clone());

            loop {
                // We make a copy because otherwise we would be iterating the HashSet while we are appending to it
                let before_copy: Vec<_> = related_types.iter().cloned().collect();
                for related_type in &before_copy {
                    // If you hit this unwrap, a schema file is likely referencing a type that does not exist
                    // Keep in mind it's case-sensitive
                    let Some(related_type) = self.types.get(related_type) else {
                        panic!("Type named {} was referenced but undefined", related_type);
                    };
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

            // log::debug!(
            //     "type {} fingerprint is {}",
            //     type_name,
            //     fingerprint.as_uuid()
            // );
            schemas_by_name.insert(type_name.to_string(), fingerprint);
        }

        for (_type_name, named_type) in &self.types {
            let fingerprint = schemas_by_name.get(named_type.type_name()).unwrap();
            let schema = named_type.to_schema(&self.types, &schemas_by_name);
            schemas.insert(*fingerprint, schema);
        }

        Ok(LinkedSchemas {
            schemas_by_name,
            schemas,
        })
    }
}

pub struct LinkedSchemas {
    pub schemas_by_name: HashMap<String, SchemaFingerprint>,
    pub schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
}

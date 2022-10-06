
use std::hash::{Hash, Hasher};
use std::path::Path;
use uuid::Uuid;
use crate::{SchemaFingerprint, Value, HashMap, Schema};

// struct RegisteredSchema {
//     schema:
// }


// #[derive(Debug)]
// struct SchemaFromFileTypeName(String);

// enum SchemaFromFileType {
//     Nullable,
//
// }

#[derive(Debug)]
struct SchemaFromFileStaticArray {
    item_type: Box<SchemaFromFileTypeRef>,
    length: usize
}

#[derive(Debug)]
struct SchemaFromFileDynamicArray {
    item_type: Box<SchemaFromFileTypeRef>
}

#[derive(Debug)]
struct SchemaFromFileMap {
    key_type: Box<SchemaFromFileTypeRef>,
    value_type: Box<SchemaFromFileTypeRef>
}

#[derive(Debug)]
struct SchemaFromFileRecordField {
    field_name: String,
    aliases: Vec<String>,
    field_type: Box<SchemaFromFileTypeRef>
}

#[derive(Debug)]
struct SchemaFromFileRecord {
    type_name: String,
    aliases: Vec<String>,
    fields: Vec<SchemaFromFileRecordField>,
}

#[derive(Debug)]
struct SchemaFromFileEnumSymbol {
    symbol_name: String,
    aliases: Vec<String>,
    value: i32
}

#[derive(Debug)]
struct SchemaFromFileEnum {
    type_name: String,
    aliases: Vec<String>,
    symbols: Vec<SchemaFromFileEnumSymbol>,
}

#[derive(Debug)]
struct SchemaFromFileFixed {
    type_name: String,
    aliases: Vec<String>,
    length: usize
}

#[derive(Debug)]
enum SchemaFromFileTypeRef {
    Nullable(Box<SchemaFromFileTypeRef>),
    Boolean,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Bytes,
    Buffer,
    String,
    StaticArray(SchemaFromFileStaticArray),
    DynamicArray(SchemaFromFileDynamicArray),
    Map(SchemaFromFileMap),
    //RecordRef(SchemaFromFileTypeName),
    NamedType(SchemaFromFileTypeName)
    // Record(SchemaFromFileTypeName),
    // Enum(SchemaFromFileTypeName),
    // Fixed(SchemaFromFileTypeName),
    // union?
}

#[derive(Debug)]
pub enum SchemaLoaderError {
    Str(&'static str),
    String(String)
}

pub type SchemaLoaderResult<T> = Result<T, SchemaLoaderError>;



#[derive(Default)]
pub struct SchemaLoader {
    records: Vec<SchemaFromFileRecord>,
    // enums
    // fixed
    // union?
}

impl SchemaLoader {
    fn parse_json_schema_type_ref(value: &serde_json::Value, error_prefix: &str) -> SchemaLoaderResult<SchemaFromFileTypeRef> {
        let mut name;
        match value {
            serde_json::Value::String(type_name) => {
                name = type_name.as_str();
            },
            serde_json::Value::Object(json_object) => {
                name= json_object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::String(format!("{}Record field types must have a name", error_prefix)))?;
            },
            _ => return Err(SchemaLoaderError::String(format!("{}Type references must be a string or json object", error_prefix)))
        }

        Ok(match name {
            "nullable" => {
                let inner_type = value.get("inner_type").ok_or_else(|| SchemaLoaderError::String(format!("{}All nullable types must has an inner_type", error_prefix)))?;
                let inner_type = Self::parse_json_schema_type_ref(inner_type, error_prefix)?;

                SchemaFromFileTypeRef::Nullable(Box::new(inner_type))
            },
            "bool" => SchemaFromFileTypeRef::Boolean,
            "i32" => SchemaFromFileTypeRef::I32,
            "i64" => SchemaFromFileTypeRef::I64,
            "u32" => SchemaFromFileTypeRef::U32,
            "u64" => SchemaFromFileTypeRef::U64,
            "f32" => SchemaFromFileTypeRef::F32,
            "f64" => SchemaFromFileTypeRef::F64,
            "bytes" => SchemaFromFileTypeRef::Bytes,
            "buffer" => SchemaFromFileTypeRef::Buffer,
            "string" => SchemaFromFileTypeRef::String,
            "static_array" => {
                unimplemented!()
            },
            "dynamic_array" => {
                let inner_type = value.get("inner_type").ok_or_else(|| SchemaLoaderError::String(format!("{}All dynamic_array types must has an inner_type", error_prefix)))?;
                let inner_type = Self::parse_json_schema_type_ref(inner_type, error_prefix)?;

                SchemaFromFileTypeRef::DynamicArray(SchemaFromFileDynamicArray {
                    item_type: Box::new(inner_type)
                })
            },
            "map" => {
                unimplemented!()
            },
            "record_ref" => {
                unimplemented!()
            }
            // StaticArray(SchemaFromFileStaticArray),
            // DynamicArray(SchemaFromFileDynamicArray),
            // Map(SchemaFromFileMap),
            // RecordRef(String),
            // Record(SchemaFromFileRecord),
            // Enum(SchemaFromFileEnum),
            // Fixed(SchemaFromFileFixed),
            _ => SchemaFromFileTypeRef::NamedType(SchemaFromFileTypeName(name.to_string()))
        })
    }

    fn parse_json_schema_record_field(value: &serde_json::Value, error_prefix: &str) -> SchemaLoaderResult<SchemaFromFileRecordField> {
        let object = value.as_object().ok_or_else(|| SchemaLoaderError::String(format!("{}Record schema fields must be a json object", error_prefix)))?;

        let field_name = object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::String(format!("{}Record fields must be a name that is a string", error_prefix)))?.to_string();


        let error_prefix = format!("{}[Field {}]", error_prefix, field_name);
        //let field_schema = object.get("type").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();
        //let field_schema = object.get("type").ok_or_else(|| SchemaLoaderError::Str("Schema file record fields must have a type of string or json object"))?;
        let field_type_json_value = object.get("type").ok_or_else(|| SchemaLoaderError::String(format!("{}Record fields must have a type of string or json object", error_prefix)))?;
        let field_type = Self::parse_json_schema_type_ref(field_type_json_value, &error_prefix)?;

        Ok(SchemaFromFileRecordField {
            field_name,
            field_type: Box::new(field_type)
        })
    }

    fn parse_json_schema_record(&mut self, object: &serde_json::Map<String, serde_json::Value>, error_prefix: &str) -> SchemaLoaderResult<SchemaFromFileRecord> {
        let name = object.get("name").ok_or_else(|| SchemaLoaderError::String(format!("{}Records must have a name", error_prefix)))?;
        let name_str = name.as_str().ok_or_else(|| SchemaLoaderError::String(format!("{}Records must have a name", error_prefix)))?;

        let error_prefix = format!("{}[Record {}]", error_prefix, name_str);
        log::debug!("Parsing record named '{}'", name_str);

        let json_aliases = object.get("aliases").map(|x| x.as_array()).flatten();
        let mut aliases = vec![];
        if let Some(json_aliases) = json_aliases {
            for json_alias in json_aliases {
                aliases.push(json_alias.as_str().ok_or_else(|| SchemaLoaderError::String(format!("{}Record's aliases must be strings", error_prefix)))?)
            }
        }

        let json_fields = object.get("fields").map(|x| x.as_array()).flatten().ok_or_else(|| SchemaLoaderError::String(format!("{}Records must have an array of fields", error_prefix)))?;
        let mut fields = vec![];
        for json_field in json_fields {
            fields.push(Self::parse_json_schema_record_field(json_field, &error_prefix)?);
        }

        Ok(SchemaFromFileRecord {
            type_name: SchemaFromFileTypeName(name_str.to_string()),
            fields
        })
    }

    fn parse_json_schema(&mut self, value: &serde_json::Value, error_prefix: &str) -> SchemaLoaderResult<()> {
        let object = value.as_object().ok_or_else(|| SchemaLoaderError::String(format!("{}Schema file must be an array of json objects", error_prefix)))?;

        let object_type = object.get("type").ok_or_else(|| SchemaLoaderError::String(format!("{}Schema file objects must have a type field", error_prefix)))?;
        let object_type_str = object_type.as_str().ok_or_else(|| SchemaLoaderError::String(format!("{}Schema file objects must have a type field that is a string", error_prefix)))?;
        match object_type_str {
            "record" => {
                let record = self.parse_json_schema_record(object, error_prefix)?;
                //TODO: Add it to the list
                //println!("{:#?}", record);
                self.records.push(record);
            },
            _ => Err(SchemaLoaderError::String(format!("Schema file object has a type field that is unrecognized {:?}", object_type_str)))?
        }

        Ok(())
    }

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
            let value: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
            //println!("VALUE {:#?}", value);

            let json_objects = value.as_array().ok_or_else(|| SchemaLoaderError::Str("Schema file must be an array of json objects"))?;

            for json_object in json_objects {
                self.parse_json_schema(&json_object, &format!("[{}]", file.path().display()))?;
            }
        }

        Ok(())
    }

    pub fn finish(&mut self) {
        // Apply aliases

        // Hash each thing


    }
}

#[derive(Default)]
struct SchemaRegistry {
    // All schemas we know about, including old ones
    schemas: HashMap<SchemaFingerprint, Schema>,

    // Only current schemas can be looked up by name. Same schema can be aliased
    schema_by_name: HashMap<String, SchemaFingerprint>,
    schema_by_id: HashMap<Uuid, SchemaFingerprint>,
}

impl SchemaRegistry {
    pub fn load_schema_cache_from_dir() {
        // Ingest all types, verify no bad data or collisions with existing data
    }

    pub fn save_schema_cache_to_dir() {

    }

    pub fn load_current_schemas_from_dir() {
        // all schemas are loaded, reffing each other by name
        // to fingerprint a schema
        //  - hash name of schema
        //  - find all schemas referenced
        //  - has all referenced schemas deterministically (where they ref each other by name?)

        // 1. Ingest all schema data from disk

        // 2. Update all names to be latest values (i.e. correct aliases to proper names)

        // 3. Deterministically hash each schema (records only?)

        // 4. Produce new schema objects that reference each other via hash

        // 5. Merge data with existing data
    }

    pub fn read_schemas_from_file() {

    }

    pub fn read_schema_object() {

    }
}

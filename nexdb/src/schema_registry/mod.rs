
use std::hash::{Hash, Hasher};
use std::path::Path;
use uuid::Uuid;
use crate::{SchemaFingerprint, Value, HashMap, Schema};

// struct RegisteredSchema {
//     schema:
// }


#[derive(Debug)]
struct SchemaFromFileSchemaName(String);

#[derive(Debug)]
struct SchemaFromFileStaticArray {
    item_type: SchemaFromFileSchemaName,
    length: usize
}

#[derive(Debug)]
struct SchemaFromFileDynamicArray {
    item_type: SchemaFromFileSchemaName
}

#[derive(Debug)]
struct SchemaFromFileMap {
    key_type: SchemaFromFileSchemaName,
    value_type: SchemaFromFileSchemaName
}

#[derive(Debug)]
struct SchemaFromFileRecordField {
    field_name: String,
    field_schema: SchemaFromFileSchemaName
}

#[derive(Debug)]
struct SchemaFromFileRecord {
    type_name: SchemaFromFileSchemaName,
    fields: Vec<SchemaFromFileRecordField>,
}

#[derive(Debug)]
struct SchemaFromFileEnumSymbol {
    symbol_name: String,
    value: i32
}

#[derive(Debug)]
struct SchemaFromFileEnum {
    type_name: SchemaFromFileSchemaName,
    symbols: Vec<SchemaFromFileEnumSymbol>,
}

#[derive(Debug)]
struct SchemaFromFileFixed {
    type_name: SchemaFromFileSchemaName,
    length: usize
}

#[derive(Debug)]
enum SchemaFromFile {
    Nullable(SchemaFromFileSchemaName),
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
    RecordRef(String),
    Record(SchemaFromFileRecord),
    Enum(SchemaFromFileEnum),
    Fixed(SchemaFromFileFixed),
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

}

impl SchemaLoader {


    fn parse_json_schema_record_field(value: &serde_json::Value) -> SchemaLoaderResult<SchemaFromFileRecordField> {
        let object = value.as_object().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema fields must be a json object"))?;

        let field_name = object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();
        let field_schema = object.get("type").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();

        Ok(SchemaFromFileRecordField {
            field_name,
            field_schema: SchemaFromFileSchemaName(field_schema)
        })
    }

    fn parse_json_schema_record(&mut self, object: &serde_json::Map<String, serde_json::Value>) -> SchemaLoaderResult<SchemaFromFileRecord> {
        let name = object.get("name").ok_or_else(|| SchemaLoaderError::Str("Schema file record schemas must have a name"))?;
        let name_str = name.as_str().ok_or_else(|| SchemaLoaderError::Str("Schema file record schemas must have a name"))?;

        let json_aliases = object.get("aliases").map(|x| x.as_array()).flatten();
        let mut aliases = vec![];
        if let Some(json_aliases) = json_aliases {
            for json_alias in json_aliases {
                aliases.push(json_alias.as_str().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema aliases must be strings"))?)
            }
        }

        let json_fields = object.get("fields").map(|x| x.as_array()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record schemas must have an array of fields"))?;
        let mut fields = vec![];
        for json_field in json_fields {
            fields.push(Self::parse_json_schema_record_field(json_field)?);
        }

        Ok(SchemaFromFileRecord {
            type_name: SchemaFromFileSchemaName(name_str.to_string()),
            fields
        })
    }

    fn parse_json_schema(&mut self, value: &serde_json::Value) -> SchemaLoaderResult<()> {
        let object = value.as_object().ok_or_else(|| SchemaLoaderError::Str("Schema file must be an array of json objects"))?;

        let object_type = object.get("type").ok_or_else(|| SchemaLoaderError::Str("Schema file objects must have a type field"))?;
        let object_type_str = object_type.as_str().ok_or_else(|| SchemaLoaderError::Str("Schema file objects must have a type field that is a string"))?;
        match object_type_str {
            "record" => {
                let x = self.parse_json_schema_record(object)?;
                //TODO: Add it to the list
                println!("{:#?}", x);
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
            log::info!("Processing file {:?}", file.path());
            let schema_str = std::fs::read_to_string(file.path()).unwrap();
            let value: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
            //println!("VALUE {:#?}", value);

            let json_objects = value.as_array().ok_or_else(|| SchemaLoaderError::Str("Schema file must be an array of json objects"))?;

            for json_object in json_objects {
                self.parse_json_schema(&json_object)?;
            }
        }

        Ok(())
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

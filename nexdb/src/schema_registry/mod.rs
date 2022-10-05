
use std::hash::{Hash, Hasher};
use std::path::Path;
use uuid::Uuid;
use crate::{SchemaFingerprint, Value, HashMap, Schema};

// struct RegisteredSchema {
//     schema:
// }


struct SchemaFromFileSchemaName(String);

struct SchemaFromFileStaticArray {
    item_type: SchemaFromFileSchemaName,
    length: usize
}

struct SchemaFromFileDynamicArray {
    item_type: SchemaFromFileSchemaName
}

struct SchemaFromFileMap {
    key_type: SchemaFromFileSchemaName,
    value_type: SchemaFromFileSchemaName
}

struct SchemaFromFileRecordField {
    field_name: String,
    field_schema: SchemaFromFileSchemaName
}

struct SchemaFromFileRecord {
    type_name: SchemaFromFileSchemaName,
    fields: Vec<SchemaFromFileSchemaName>,
}

struct SchemaFromFileEnumSymbol {
    symbol_name: String,
    value: i32
}

struct SchemaFromFileEnum {
    type_name: SchemaFromFileSchemaName,
    symbols: Vec<SchemaFromFileEnumSymbol>,
}

struct SchemaFromFileFixed {
    type_name: SchemaFromFileSchemaName,
    length: usize
}

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

pub struct SchemaLoader {

}

impl SchemaLoader {
    pub fn add_source_dir<T: AsRef<Path>>(path: T) {
        let walker = globwalk::GlobWalkerBuilder::new(path.as_ref(), "*.schema")
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            let file = file.unwrap();
            log::info!("Processing file {:?}", file.path());
            let schema_str = std::fs::read_to_string(file.path()).unwrap();
            let value: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
            println!("VALUE {:?}", value);
        }
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

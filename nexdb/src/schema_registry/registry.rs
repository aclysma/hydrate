
use std::hash::{Hash, Hasher};
use std::path::Path;
use siphasher::sip128::Hasher128;
use uuid::Uuid;
use crate::{SchemaFingerprint, Value, HashMap, Schema, HashSet};


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

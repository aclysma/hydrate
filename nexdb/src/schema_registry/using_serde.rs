
use std::hash::{Hash, Hasher};
use std::path::Path;
use serde::de::MapAccess;
use uuid::Uuid;
use crate::{SchemaFingerprint, Value, HashMap, Schema};
use serde::Deserialize;

//TODO: TRY WITH https://github.com/serde-rs/json/issues/286 to get better error messages

#[derive(Deserialize, Debug)]
struct SchemaFromFileTypeRefComplex {
    name: String,
    #[serde(default)]
    inner_type: Option<Box<SchemaFromFileTypeRef>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SchemaFromFileTypeRef {
    Simple(String),
    Complex(SchemaFromFileTypeRefComplex),
    //Recursive(Box<SchemaFromFileTypeRef>)
}

#[derive(Deserialize, Debug)]
struct SchemaFromFileRecordField {
    name: String,
    #[serde(rename = "type")]
    //ty: serde_json::Value
    ty: SchemaFromFileTypeRef
}

// fn deser_type_ref<D: serde::Deserializer>(deserializer: D) -> Result<String, D::Error> {
//     struct Visitor;
//
//     impl serde::de::Visitor for Visitor {
//         type Value = SchemaFromFileTypeRef;
//
//         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//             formatter.write_str("type specified as string or object")
//         }
//
//         fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
//             Ok(SchemaFromFileTypeRef::Simple(s.to_string()))
//         }
//
//         fn visit_map<A>(self, map: A) -> Result<Self::Value, serde::de::Error> where A: MapAccess<'de> {
//             map.
//
//
//             Ok(SchemaFromFileTypeRef::Complex(SchemaFromFileTypeRefComplex {
//                 name:
//             }))
//         }
//     }
// }

#[derive(Deserialize, Debug)]
struct SchemaFromJsonNamedTypeDef {
    #[serde(rename = "type")]
    ty: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    fields: Vec<SchemaFromFileRecordField>
}

// // #[derive(Deserialize, Debug)]
// // struct SchemaFromJson {
// //     schemas: Vec<SchemaFromJsonNamedTypeDef>
// // }
//
//
//
// // struct RegisteredSchema {
// //     schema:
// // }
//
//
// #[derive(Debug)]
// struct SchemaFromFileTypeName(String);
//
// // enum SchemaFromFileType {
// //     Nullable,
// //
// // }
//
// #[derive(Debug)]
// struct SchemaFromFileStaticArray {
//     item_type: Box<SchemaFromFileTypeRef>,
//     length: usize
// }
//
// #[derive(Debug)]
// struct SchemaFromFileDynamicArray {
//     item_type: Box<SchemaFromFileTypeRef>
// }
//
// #[derive(Debug)]
// struct SchemaFromFileMap {
//     key_type: Box<SchemaFromFileTypeRef>,
//     value_type: Box<SchemaFromFileTypeRef>
// }
//
// #[derive(Debug)]
// struct SchemaFromFileRecordField {
//     field_name: String,
//     field_type: Box<SchemaFromFileTypeRef>
// }
//
// #[derive(Debug)]
// struct SchemaFromFileRecord {
//     type_name: SchemaFromFileTypeName,
//     fields: Vec<SchemaFromFileRecordField>,
// }
//
// #[derive(Debug)]
// struct SchemaFromFileEnumSymbol {
//     symbol_name: String,
//     value: i32
// }
//
// #[derive(Debug)]
// struct SchemaFromFileEnum {
//     type_name: SchemaFromFileTypeName,
//     symbols: Vec<SchemaFromFileEnumSymbol>,
// }
//
// #[derive(Debug)]
// struct SchemaFromFileFixed {
//     type_name: SchemaFromFileTypeName,
//     length: usize
// }
//
// #[derive(Debug)]
// enum SchemaFromFileTypeRef {
//     Nullable(Box<SchemaFromFileTypeRef>),
//     Boolean,
//     I32,
//     I64,
//     U32,
//     U64,
//     F32,
//     F64,
//     Bytes,
//     Buffer,
//     String,
//     StaticArray(SchemaFromFileStaticArray),
//     DynamicArray(SchemaFromFileDynamicArray),
//     Map(SchemaFromFileMap),
//     //RecordRef(SchemaFromFileTypeName),
//     NamedType(SchemaFromFileTypeName)
//     // Record(SchemaFromFileTypeName),
//     // Enum(SchemaFromFileTypeName),
//     // Fixed(SchemaFromFileTypeName),
//     // union?
// }

#[derive(Debug)]
pub enum SchemaLoaderError {
    Str(&'static str),
    String(String)
}

pub type SchemaLoaderResult<T> = Result<T, SchemaLoaderError>;



#[derive(Default)]
pub struct SchemaLoader {
    //records: Vec<SchemaFromFileRecord>,
    // enums
    // fixed
    // union?
}

impl SchemaLoader {

    /*
    fn parse_json_schema_type_ref(value: &serde_json::Value) -> SchemaLoaderResult<SchemaFromFileTypeRef> {
        let mut name;
        match value {
            serde_json::Value::String(type_name) => {
                name = type_name.as_str();
            },
            serde_json::Value::Object(json_object) => {
                name= json_object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record field types must have a name"))?;
            },
            _ => return Err(SchemaLoaderError::Str("Type references must be a string or json object"))
        }

        Ok(match name {
            "nullable" => {
                let inner_type = value.get("inner_type").ok_or_else(|| SchemaLoaderError::Str("nullable type must has an inner_type"))?;
                let inner_type = Self::parse_json_schema_type_ref(inner_type)?;

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
                let inner_type = value.get("inner_type").ok_or_else(|| SchemaLoaderError::Str("dynamic_array type must has an inner_type"))?;
                let inner_type = Self::parse_json_schema_type_ref(inner_type)?;

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

    fn parse_json_schema_record_field(value: &serde_json::Value) -> SchemaLoaderResult<SchemaFromFileRecordField> {
        let object = value.as_object().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema fields must be a json object"))?;

        let field_name = object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record fields must be a name that is a string"))?.to_string();
        //let field_schema = object.get("type").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();
        //let field_schema = object.get("type").ok_or_else(|| SchemaLoaderError::Str("Schema file record fields must have a type of string or json object"))?;
        let field_type_json_value = object.get("type").ok_or_else(|| SchemaLoaderError::Str("Schema file record fields must have a type of string or json object"))?;
        let field_type = Self::parse_json_schema_type_ref(field_type_json_value)?;

        Ok(SchemaFromFileRecordField {
            field_name,
            field_type: Box::new(field_type)
        })
    }

    fn parse_json_schema_record(&mut self, object: &serde_json::Map<String, serde_json::Value>) -> SchemaLoaderResult<SchemaFromFileRecord> {
        let name = object.get("name").ok_or_else(|| SchemaLoaderError::Str("Schema file records must have a name"))?;
        let name_str = name.as_str().ok_or_else(|| SchemaLoaderError::Str("Schema file records must have a name"))?;

        let json_aliases = object.get("aliases").map(|x| x.as_array()).flatten();
        let mut aliases = vec![];
        if let Some(json_aliases) = json_aliases {
            for json_alias in json_aliases {
                aliases.push(json_alias.as_str().ok_or_else(|| SchemaLoaderError::Str("Schema file record's aliases must be strings"))?)
            }
        }

        let json_fields = object.get("fields").map(|x| x.as_array()).flatten().ok_or_else(|| SchemaLoaderError::Str("Schema file records must have an array of fields"))?;
        let mut fields = vec![];
        for json_field in json_fields {
            fields.push(Self::parse_json_schema_record_field(json_field)?);
        }

        Ok(SchemaFromFileRecord {
            type_name: SchemaFromFileTypeName(name_str.to_string()),
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
*/
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
            let value: Vec<SchemaFromJsonNamedTypeDef> = serde_json::from_str(&schema_str).unwrap();
            println!("VALUE {:#?}", value);

            // let json_objects = value.as_array().ok_or_else(|| SchemaLoaderError::Str("Schema file must be an array of json objects"))?;
            //
            // for json_object in json_objects {
            //     self.parse_json_schema(&json_object)?;
            // }
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

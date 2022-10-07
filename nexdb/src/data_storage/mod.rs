use std::path::Path;
use uuid::Uuid;
use crate::{Database, HashMap, Schema, SchemaFingerprint, Value};
use serde::{Serialize, Deserialize};

fn ordered_map<S>(value: &HashMap<String, serde_json::Value>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

#[derive(Debug, Serialize, Deserialize)]
enum ObjectReference {
    Uuid(Uuid)
}


#[derive(Debug, Serialize, Deserialize)]
struct DataStorageJsonObject {
    object_id: Uuid,
    schema: Uuid,
    schema_name: String,
    prototype: Option<ObjectReference>,
    #[serde(serialize_with = "ordered_map")]
    properties: HashMap<String, serde_json::Value>,
    //property_null_overrides: HashMap<String, NullOverride>,
    //properties_in_replace_mode: HashSet<String>,
    //dynamic_array_entries: HashMap<String, Vec<Uuid>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DataStorageJsonSingleFile {
    objects: Vec<DataStorageJsonObject>
}

fn property_value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Nullable(_) => unimplemented!(),
        Value::Boolean(x) => serde_json::Value::from(*x),
        Value::I32(x) => serde_json::Value::from(*x),
        Value::I64(x) => serde_json::Value::from(*x),
        Value::U32(x) => serde_json::Value::from(*x),
        Value::U64(x) => serde_json::Value::from(*x),
        Value::F32(x) => serde_json::Value::from(*x),
        Value::F64(x) => serde_json::Value::from(*x),
        Value::Bytes(_) => unimplemented!(),
        Value::Buffer(_) => unimplemented!(),
        Value::String(x) => serde_json::Value::from(x.clone()),
        Value::StaticArray(_) => unimplemented!(),
        Value::DynamicArray(_) => unimplemented!(),
        Value::Map(_) => unimplemented!(),
        Value::RecordRef(_) => unimplemented!(),
        Value::Record(_) => unimplemented!(),
        Value::Enum(_) => unimplemented!(),
        Value::Fixed(_) => unimplemented!(),
    }
}

impl DataStorageJsonSingleFile {
    pub fn store<T: AsRef<Path>>(database: &Database, path: T) {

        let mut stored_objects = Vec::with_capacity(database.objects().len());

        for (id, obj) in database.objects() {
            let mut properties: HashMap<String, serde_json::Value> = Default::default();
            //let object_schema = Schema::NamedType(database.object_schema(*id).fingerprint());

            // Store simple properties
            for (key, value) in &obj.properties {
                let json_value = property_value_to_json(value);
                properties.insert(key.clone(), json_value);
            }

            // Store nullable status as a property

            // Store replace mode as a property

            // Store dynamic array entries as a property

            stored_objects.push(DataStorageJsonObject {
                object_id: Uuid::from_u128(id.0),
                schema: obj.schema.fingerprint().as_uuid(),
                schema_name: obj.schema.name().to_string(),
                prototype: obj.prototype.map(|x| ObjectReference::Uuid(Uuid::from_u128(x.0))),
                properties
            });
        }

        let storage = DataStorageJsonSingleFile {
            objects: stored_objects
        };

        let json = serde_json::to_string_pretty(&storage).unwrap();
        println!("JSON {}", json);

        let reloaded: DataStorageJsonSingleFile = serde_json::from_str(&json).unwrap();
        println!("RELOADED {:?}", reloaded);

        // let json2 = serde_json::to_string_pretty(&reloaded).unwrap();
        // println!("JSON {}", json);
        //
        // assert_eq!(json, json2);
    }

    pub fn load<T: AsRef<Path>>(database: &mut Database, path: T) {

        //let reloaded: DataStorageJsonSingleFile = serde_json::from_str(&json).unwrap();
    }
}
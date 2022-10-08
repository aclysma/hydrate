use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;
use crate::{Database, HashMap, NullOverride, ObjectId, OverrideBehavior, Schema, SchemaFingerprint, SchemaNamedType, Value};
use serde::{Serialize, Deserialize};

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

fn null_override_to_string_value(null_override: NullOverride) -> &'static str{
    match null_override {
        NullOverride::SetNull => "SetNull",
        NullOverride::SetNonNull => "SetNonNull",
    }
}

fn string_to_null_override_value(s: &str) -> Option<NullOverride> {
    match s {
        "SetNull" => Some(NullOverride::SetNull),
        "SetNonNull" => Some(NullOverride::SetNonNull),
        _ => None
    }
}

fn store_object_into_properties(
    database: &Database,
    object_id: ObjectId,
    stored_object: &mut DataStorageJsonObject,
    schema: &Schema,
    path: &str,
) {
    match schema {
        Schema::Nullable(inner_schema) => {
            //TODO: Scrape separately, so we save even if path ancestor is nulled?
            // Save null override
            // if let Some(null_override) = database.get_null_override(object_id, path) {
            //     let null_override_path = format!("{}.null_override", path);
            //     let null_override_value = null_override_to_string_value(null_override);
            //     stored_object.properties.insert(null_override_path, serde_json::Value::from(null_override_value.to_string()));
            // }

            // Save value
            if database.resolve_is_null(object_id, path) == Some(false) {
                let value_path = format!("{}.value", path);
                store_object_into_properties(database, object_id, stored_object, &*inner_schema, &value_path);
            }
        }
        Schema::Boolean | Schema::I32 | Schema::I64 | Schema::U32 | Schema::U64 | Schema::F32 | Schema::F64 | Schema::String => {
            let value = database.get_property_override(object_id, path);
            if let Some(value) = value {
                stored_object.properties.insert(path.to_string(), property_value_to_json(value));
            }
        }
        Schema::Bytes => {
            unimplemented!();
        }
        Schema::Buffer => {
            unimplemented!();
        }
        Schema::StaticArray(_) => {
            unimplemented!();
        }
        Schema::DynamicArray(dynamic_array) => {
            //TODO: Scrape separately, so we save even if path ancestor is nulled?
            // if database.get_override_behavior(object_id, path) == OverrideBehavior::Replace {
            //     let null_override_path = format!("{}.replace", path);
            //     stored_object.properties.insert(null_override_path, serde_json::Value::from(true));
            // }

            let elements = database.get_dynamic_array_overrides(object_id, path);
            for element_id in elements {
                let element_path = format!("{}.{}", path, element_id);
                store_object_into_properties(database, object_id, stored_object, dynamic_array.item_type(), &element_path);
            }
        }
        Schema::Map(_) => {
            unimplemented!();
        }
        Schema::NamedType(named_type) => {
            let named_type = database.find_named_type_by_fingerprint(*named_type).unwrap().clone();
            match named_type {
                SchemaNamedType::Record(record) => {
                    for field in record.fields() {
                        let field_path = if path.is_empty() {
                            field.name().to_string()
                        } else {
                            format!("{}.{}", path, field.name())
                        };
                        store_object_into_properties(database, object_id, stored_object, field.field_schema(), &field_path);
                    }
                }
                SchemaNamedType::Enum(_) => {
                    unimplemented!();
                }
                SchemaNamedType::Fixed(_) => {
                    unimplemented!();
                }
            }
        }
    }
}

fn restore_object_from_properties(
    database: &mut Database,
    object_id: ObjectId,
    stored_object: &DataStorageJsonObject,
    schema: &Schema,
    path: &str,
    max_path_length: usize
) {
    // Cyclical types can cause unbounded depth of properties, so limit ourselves to the
    // known max length of paths we will load.
    if path.len() > max_path_length {
        return;
    }

    match schema {
        Schema::Nullable(inner_schema) => {
            // restore null override
            // let null_override_path = format!("{}.null_override", path);
            // if let Some(null_override_str) = stored_object.properties.get(&null_override_path) {
            //     let null_override = string_to_null_override_value(null_override_str.as_str().unwrap()).unwrap();
            //     database.set_null_override(object_id, path, null_override);
            // }

            // Restore value
            let value_path = format!("{}.value", path);
            restore_object_from_properties(database, object_id, stored_object, &*inner_schema, &value_path, max_path_length);
        }
        Schema::Boolean => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore bool {} from {}", path, value);
                database.set_property_override(object_id, path, Value::Boolean(value.as_bool().unwrap()));
            }
        }
        Schema::I32 => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u32 {} from {}", path, value);
                database.set_property_override(object_id, path, Value::I32(value.as_i64().unwrap() as i32));
            }
        }
        Schema::I64 => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u64 {} from {}", path, value);
                database.set_property_override(object_id, path, Value::I64(value.as_i64().unwrap()));
            }
        }
        Schema::U32 => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u32 {} from {}", path, value);
                database.set_property_override(object_id, path, Value::U32(value.as_u64().unwrap() as u32));
            }
        }
        Schema::U64 => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u64 {} from {}", path, value);
                database.set_property_override(object_id, path, Value::U64(value.as_u64().unwrap()));
            }
        }
        Schema::F32 => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore f32 {} from {}", path, value);
                database.set_property_override(object_id, path, Value::F32(value.as_f64().unwrap() as f32));
            }
        }
        Schema::F64 => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore f64 {} from {}", path, value);
                database.set_property_override(object_id, path, Value::F32(value.as_f64().unwrap() as f32));
            }
        }
        Schema::Bytes => {
            unimplemented!();
        }
        Schema::Buffer => {
            unimplemented!();
        }
        Schema::String => {
            let value = stored_object.properties.get(path);
            if let Some(value) = value {
                log::debug!("restore string {} from {}", path, value);
                database.set_property_override(object_id, path, Value::String(value.as_str().unwrap().to_string()));
            }
        }
        Schema::StaticArray(_) => {
            unimplemented!();
        }
        Schema::DynamicArray(dynamic_array) => {
            let override_behavior_path = format!("{}.replace", path);
            if let Some(value) = stored_object.properties.get(path) {
                if value.as_bool() == Some(true) {
                    database.set_override_behavior(object_id, path, OverrideBehavior::Replace);
                }
            }

            let elements = stored_object.properties.get(path).unwrap().as_array().unwrap();
            for element in elements {
                let element_id = element.as_str().unwrap();
                database.add_dynamic_array_override(object_id, element_id);

                restore_object_from_properties(database, object_id, stored_object, dynamic_array.item_type(), &format!("{}.{}", path, element_id), max_path_length);
            }
        }
        Schema::Map(_) => {
            unimplemented!();
        }
        Schema::NamedType(named_type) => {
            let named_type = database.find_named_type_by_fingerprint(*named_type).unwrap().clone();
            match named_type {
                SchemaNamedType::Record(record) => {
                    for field in record.fields() {
                        let field_path = if path.is_empty() {
                            field.name().to_string()
                        } else {
                            format!("{}.{}", path, field.name())
                        };
                        restore_object_from_properties(database, object_id, stored_object, field.field_schema(), &field_path, max_path_length);
                    }
                }
                SchemaNamedType::Enum(_) => {
                    unimplemented!();
                }
                SchemaNamedType::Fixed(_) => {
                    unimplemented!();
                }
            }
        }
    }
}

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

impl ObjectReference {
    pub fn as_uuid(&self) -> Uuid {
        match self {
            ObjectReference::Uuid(uuid) => *uuid
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct DataStorageJsonObject {
    object_id: Uuid,
    schema: Uuid,
    schema_name: String,
    prototype: Option<ObjectReference>,
    #[serde(serialize_with = "ordered_map")]
    properties: HashMap<String, serde_json::Value>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DataStorageJsonSingleFile {
    objects: Vec<DataStorageJsonObject>
}

impl DataStorageJsonSingleFile {
    pub fn store_string(database: &Database) -> String {

        let mut stored_objects = Vec::with_capacity(database.objects().len());

        for (id, obj) in database.objects() {
            //let mut properties: HashMap<String, serde_json::Value> = Default::default();
            let mut stored_object = DataStorageJsonObject {
                object_id: Uuid::from_u128(id.0),
                schema: obj.schema.fingerprint().as_uuid(),
                schema_name: obj.schema.name().to_string(),
                prototype: obj.prototype.map(|x| ObjectReference::Uuid(Uuid::from_u128(x.0))),
                properties: Default::default()
            };

            // property_null_overrides
            for (path, null_override) in &obj.property_null_overrides {
                stored_object.properties.insert(format!("{}.null_override", path), serde_json::Value::from(null_override_to_string_value(*null_override)));
            }

            for path in &obj.properties_in_replace_mode {
                stored_object.properties.insert(format!("{}.replace", path), serde_json::Value::from(true));
            }

            for (path, elements) in &obj.dynamic_array_entries {
                let elements_json: Vec<_> = elements.iter().map(|x| serde_json::Value::from(x.to_string())).collect();
                let elements_json_array = serde_json::Value::from(elements_json);
                stored_object.properties.insert(path.to_string(), elements_json_array);
            }

            let schema = Schema::NamedType(obj.schema.fingerprint());
            store_object_into_properties(database, *id, &mut stored_object, &schema, "");
            stored_objects.push(stored_object);
        }

        let storage = DataStorageJsonSingleFile {
            objects: stored_objects
        };

        serde_json::to_string_pretty(&storage).unwrap()
    }


    pub fn load_string(database: &mut Database, json: &str) {
        let reloaded: DataStorageJsonSingleFile = serde_json::from_str(json).unwrap();

        for stored_object in &reloaded.objects {
            let schema_fingerprint = SchemaFingerprint(stored_object.schema.as_u128());
            let object_id = ObjectId(stored_object.object_id.as_u128());

            log::debug!("Restore object {}", stored_object.object_id);
            database.restore_object(
                object_id,
                schema_fingerprint,
                stored_object.schema_name.clone(),
                stored_object.prototype.as_ref().map(|x| ObjectId(x.as_uuid().as_u128()))
            );

            let schema = Schema::NamedType(schema_fingerprint);

            // We use the max path length to ensure we stop recursing through types when we know no properties will exist
            let mut max_path_length = 0;
            for (k, _) in &stored_object.properties {
                max_path_length = max_path_length.max(k.len());
            }

            restore_object_from_properties(database, object_id, stored_object, &schema, "", max_path_length);
        }


    }
}
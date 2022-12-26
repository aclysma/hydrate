use crate::edit_context::EditContext;
use crate::{DataObjectInfo, HashMap, HashSet, NullOverride, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, OverrideBehavior, Schema, SchemaFingerprint, SchemaNamedType, Value};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

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
        Value::ObjectRef(x) => serde_json::Value::from(x.as_uuid().to_string()),
        Value::Record(_) => unimplemented!(),
        Value::Enum(_) => unimplemented!(),
        Value::Fixed(_) => unimplemented!(),
    }
}

fn json_to_property_value_with_schema(
    schema: &Schema,
    value: &serde_json::Value,
) -> Value {
    match schema {
        Schema::Nullable(_) => unimplemented!(),
        Schema::Boolean => Value::Boolean(value.as_bool().unwrap()),
        Schema::I32 => Value::I32(value.as_i64().unwrap() as i32),
        Schema::I64 => Value::I64(value.as_i64().unwrap()),
        Schema::U32 => Value::U32(value.as_u64().unwrap() as u32),
        Schema::U64 => Value::U64(value.as_u64().unwrap()),
        Schema::F32 => Value::F32(value.as_f64().unwrap() as f32),
        Schema::F64 => Value::F64(value.as_f64().unwrap()),
        Schema::Bytes => unimplemented!(),
        Schema::Buffer => unimplemented!(),
        Schema::String => Value::String(value.as_str().unwrap().to_string()),
        Schema::StaticArray(_) => unimplemented!(),
        Schema::DynamicArray(_) => unimplemented!(),
        Schema::Map(_) => unimplemented!(),
        Schema::ObjectRef(_) => Value::ObjectRef(ObjectId(
            Uuid::parse_str(value.as_str().unwrap()).unwrap().as_u128(),
        )),
        Schema::NamedType(_) => unimplemented!(),
    }
}

fn json_to_property_value_without_schema(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => unimplemented!(),
        serde_json::Value::Bool(x) => Value::Boolean(*x),
        serde_json::Value::Number(number) => {
            if let Some(v) = number.as_u64() {
                Value::U64(v)
            } else if let Some(v) = number.as_i64() {
                Value::I64(v)
            } else {
                Value::F64(number.as_f64().unwrap())
            }
        }
        serde_json::Value::String(x) => Value::String(x.to_string()),
        serde_json::Value::Array(_) => unimplemented!(),
        serde_json::Value::Object(_) => unimplemented!(),
    }
}

fn json_to_property_value(
    schema: Option<&Schema>,
    value: &serde_json::Value,
) -> Value {
    if let Some(schema) = schema {
        json_to_property_value_with_schema(schema, value)
    } else {
        json_to_property_value_without_schema(value)
    }
}

fn null_override_to_string_value(null_override: NullOverride) -> &'static str {
    match null_override {
        NullOverride::SetNull => "SetNull",
        NullOverride::SetNonNull => "SetNonNull",
    }
}

fn string_to_null_override_value(s: &str) -> Option<NullOverride> {
    match s {
        "SetNull" => Some(NullOverride::SetNull),
        "SetNonNull" => Some(NullOverride::SetNonNull),
        _ => None,
    }
}

fn store_object_into_properties(
    edit_context: &EditContext,
    object_id: ObjectId,
    properties: &mut HashMap<String, serde_json::Value>,
    schema: &Schema,
    path: &str,
) {
    match schema {
        Schema::Nullable(inner_schema) => {
            // Save value
            if edit_context.resolve_is_null(object_id, path) == Some(false) {
                let value_path = format!("{}.value", path);
                store_object_into_properties(
                    edit_context,
                    object_id,
                    properties,
                    &*inner_schema,
                    &value_path,
                );
            }
        }
        Schema::Boolean
        | Schema::I32
        | Schema::I64
        | Schema::U32
        | Schema::U64
        | Schema::F32
        | Schema::F64
        | Schema::String => {
            let value = edit_context.get_property_override(object_id, path);
            if let Some(value) = value {
                properties
                    .insert(path.to_string(), property_value_to_json(value));
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
            let elements = edit_context.get_dynamic_array_overrides(object_id, path);
            if let Some(elements) = elements {
                for element_id in elements {
                    let element_path = format!("{}.{}", path, element_id);
                    store_object_into_properties(
                        edit_context,
                        object_id,
                        properties,
                        dynamic_array.item_type(),
                        &element_path,
                    );
                }
            }
        }
        Schema::Map(_) => {
            unimplemented!();
        }
        Schema::ObjectRef(_) => {
            let value = edit_context.get_property_override(object_id, path);
            if let Some(value) = value {
                properties
                    .insert(path.to_string(), property_value_to_json(value));
            }
        }
        Schema::NamedType(named_type) => {
            let named_type = edit_context
                .find_named_type_by_fingerprint(*named_type)
                .unwrap()
                .clone();
            match named_type {
                SchemaNamedType::Record(record) => {
                    for field in record.fields() {
                        let field_path = if path.is_empty() {
                            field.name().to_string()
                        } else {
                            format!("{}.{}", path, field.name())
                        };
                        store_object_into_properties(
                            edit_context,
                            object_id,
                            properties,
                            field.field_schema(),
                            &field_path,
                        );
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
    edit_context: &mut EditContext,
    object_id: ObjectId,
    properties: &HashMap<String, serde_json::Value>,
    schema: &Schema,
    path: &str,
    max_path_length: usize,
) {
    // Cyclical types can cause unbounded depth of properties, so limit ourselves to the
    // known max length of paths we will load.
    if path.len() > max_path_length {
        return;
    }

    match schema {
        Schema::Nullable(inner_schema) => {
            // Restore value
            let value_path = format!("{}.value", path);
            restore_object_from_properties(
                edit_context,
                object_id,
                properties,
                &*inner_schema,
                &value_path,
                max_path_length,
            );
        }
        Schema::Boolean => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore bool {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::Boolean(value.as_bool().unwrap()),
                );
            }
        }
        Schema::I32 => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u32 {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::I32(value.as_i64().unwrap() as i32),
                );
            }
        }
        Schema::I64 => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u64 {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::I64(value.as_i64().unwrap()),
                );
            }
        }
        Schema::U32 => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u32 {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::U32(value.as_u64().unwrap() as u32),
                );
            }
        }
        Schema::U64 => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore u64 {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::U64(value.as_u64().unwrap()),
                );
            }
        }
        Schema::F32 => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore f32 {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::F32(value.as_f64().unwrap() as f32),
                );
            }
        }
        Schema::F64 => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore f64 {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::F32(value.as_f64().unwrap() as f32),
                );
            }
        }
        Schema::Bytes => {
            unimplemented!();
        }
        Schema::Buffer => {
            unimplemented!();
        }
        Schema::String => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore string {} from {}", path, value);
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::String(value.as_str().unwrap().to_string()),
                );
            }
        }
        Schema::StaticArray(_) => {
            unimplemented!();
        }
        Schema::DynamicArray(dynamic_array) => {
            let override_behavior_path = format!("{}.replace", path);
            if let Some(value) = properties.get(&override_behavior_path) {
                if value.as_bool() == Some(true) {
                    edit_context.set_override_behavior(object_id, path, OverrideBehavior::Replace);
                }
            }

            let elements = properties
                .get(path)
                .unwrap()
                .as_array()
                .unwrap();
            for element in elements {
                let element_id = element.as_str().unwrap();
                edit_context.add_dynamic_array_override(object_id, path);

                restore_object_from_properties(
                    edit_context,
                    object_id,
                    properties,
                    dynamic_array.item_type(),
                    &format!("{}.{}", path, element_id),
                    max_path_length,
                );
            }
        }
        Schema::Map(_) => {
            unimplemented!();
        }
        Schema::ObjectRef(_) => {
            let value = properties.get(path);
            if let Some(value) = value {
                log::debug!("restore f64 {} from {}", path, value);
                let uuid = Uuid::parse_str(value.as_str().unwrap()).unwrap();
                edit_context.set_property_override(
                    object_id,
                    path,
                    Value::ObjectRef(ObjectId(uuid.as_u128())),
                );
            }
        }
        Schema::NamedType(named_type) => {
            let named_type = edit_context
                .find_named_type_by_fingerprint(*named_type)
                .unwrap()
                .clone();
            match named_type {
                SchemaNamedType::Record(record) => {
                    for field in record.fields() {
                        let field_path = if path.is_empty() {
                            field.name().to_string()
                        } else {
                            format!("{}.{}", path, field.name())
                        };
                        restore_object_from_properties(
                            edit_context,
                            object_id,
                            properties,
                            field.field_schema(),
                            &field_path,
                            max_path_length,
                        );
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

fn ordered_map<S>(
    value: &HashMap<String, serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

#[derive(Debug, Serialize, Deserialize)]
enum ObjectReference {
    Uuid(Uuid),
}

impl ObjectReference {
    pub fn as_uuid(&self) -> Uuid {
        match self {
            ObjectReference::Uuid(uuid) => *uuid,
        }
    }
}

fn restore_from_json_properties(
    edit_context: &mut EditContext,
    object_name: ObjectName,
    object_location: ObjectLocation,
    object_id: Uuid,
    schema: Uuid,
    schema_name: String,
    prototype: Option<Uuid>,
    json_properties: HashMap<String, serde_json::Value>
) -> ObjectId {
    let object_id = ObjectId(object_id.as_u128());
    let schema_fingerprint = SchemaFingerprint(schema.as_u128());
    let prototype = prototype.map(|x| ObjectId(x.as_u128()));

    let named_type = edit_context
        .find_named_type_by_fingerprint(schema_fingerprint)
        .unwrap()
        .clone();

    let mut properties: HashMap<String, Value> = Default::default();
    let mut property_null_overrides: HashMap<String, NullOverride> = Default::default();
    let mut properties_in_replace_mode: HashSet<String> = Default::default();
    let mut dynamic_array_entries: HashMap<String, HashSet<Uuid>> = Default::default();

    // We use the max path length to ensure we stop recursing through types when we know no properties will exist
    let mut max_path_length = 0;
    for (k, _) in &properties {
        max_path_length = max_path_length.max(k.len());
    }

    //let schema = Schema::NamedType(schema_fingerprint);

    //Alternative way via walking through schema
    //restore_object_from_properties(edit_context, object_id, stored_object, &schema, "", max_path_length);

    for (path, value) in json_properties {
        let split_path = path.rsplit_once('.');
        //let parent_path = split_path.map(|x| x.0);
        //let path_end = split_path.map(|x| x.1);

        let mut property_handled = false;

        if let Some((parent_path, path_end)) = split_path {
            let parent_schema = named_type
                .find_property_schema(parent_path, edit_context.schemas())
                .unwrap();
            if parent_schema.is_nullable() && path_end == "null_override" {
                let null_override =
                    string_to_null_override_value(value.as_str().unwrap()).unwrap();
                //edit_context.set_null_override(object_id, path, null_override);
                log::debug!("set null override {} to {:?}", parent_path, null_override);
                property_null_overrides.insert(parent_path.to_string(), null_override);
                property_handled = true;
            }

            if parent_schema.is_dynamic_array() && path_end == "replace" {
                if value.as_bool() == Some(true) {
                    log::debug!("set property {} to replace", parent_path);
                    properties_in_replace_mode.insert(parent_path.to_string());
                }

                property_handled = true;
            }
        }

        if !property_handled {
            let property_schema = named_type
                .find_property_schema(&path, edit_context.schemas())
                .unwrap();
            if property_schema.is_dynamic_array() {
                let json_array = value.as_array().unwrap();
                for json_array_element in json_array {
                    let element = json_array_element.as_str().unwrap();
                    let element = Uuid::from_str(element).unwrap();
                    let existing_entries =
                        dynamic_array_entries.entry(path.to_string()).or_default();
                    if !existing_entries.contains(&element) {
                        log::debug!("add dynamic array element {} to {:?}", element, path);
                        existing_entries.insert(element);
                    }
                }
            } else {
                let v = json_to_property_value_with_schema(&property_schema, &value);
                log::debug!("set {} to {:?}", path, v);
                properties.insert(path.to_string(), v);
            }
        }
    }

    let mut dynamic_array_entries_as_vec: HashMap<String, HashSet<Uuid>> =
        Default::default();
    for (k, v) in dynamic_array_entries {
        dynamic_array_entries_as_vec.insert(k, v.into_iter().collect());
    }

    edit_context.restore_object(
        object_id,
        object_name,
        object_location,
        prototype,
        schema_fingerprint,
        properties,
        property_null_overrides,
        properties_in_replace_mode,
        dynamic_array_entries_as_vec
    );

    object_id
}

fn store_object_to_json_properties(obj: &DataObjectInfo) -> HashMap<String, serde_json::Value> {
    let mut properties: HashMap<String, serde_json::Value> = Default::default();

    for (path, null_override) in &obj.property_null_overrides {
        properties.insert(
            format!("{}.null_override", path),
            serde_json::Value::from(null_override_to_string_value(*null_override)),
        );
    }

    for path in &obj.properties_in_replace_mode {
        properties
            .insert(format!("{}.replace", path), serde_json::Value::from(true));
    }

    for (path, elements) in &obj.dynamic_array_entries {
        let mut sorted_elements: Vec<_> = elements.iter().copied().collect();
        sorted_elements.sort();
        let elements_json: Vec<_> = sorted_elements
            .iter()
            .map(|x| serde_json::Value::from(x.to_string()))
            .collect();
        let elements_json_array = serde_json::Value::from(elements_json);
        properties
            .insert(path.to_string(), elements_json_array);
    }

    for (k, v) in &obj.properties {
        properties
            .insert(k.to_string(), property_value_to_json(v));
    }

    properties
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectSourceDataStorageJsonObject {
    name: String,
    parent_dir: Option<Uuid>,
    schema: Uuid,
    schema_name: String,
    prototype: Option<Uuid>,
    #[serde(serialize_with = "ordered_map")]
    properties: HashMap<String, serde_json::Value>,
}

impl ObjectSourceDataStorageJsonObject {
    pub fn load_object_from_string(
        edit_context: &mut EditContext,
        object_id: Uuid,
        object_source_id: ObjectSourceId,
        json: &str,
    ) {
        let stored_object: ObjectSourceDataStorageJsonObject = serde_json::from_str(json).unwrap();
        let path_node_id = ObjectId(stored_object.parent_dir.unwrap_or(Uuid::nil()).as_u128());
        let object_location = ObjectLocation::new(object_source_id, path_node_id);
        let object_name = if stored_object.name.is_empty() {
            ObjectName::empty()
        } else {
            ObjectName::new(stored_object.name)
        };

        //let location = ObjectLocation::new(object_source_id, path);
        restore_from_json_properties(edit_context, object_name, object_location, object_id, stored_object.schema, stored_object.schema_name, stored_object.prototype, stored_object.properties);
    }

    pub fn save_object_to_string(
        edit_context: &EditContext,
        object_id: ObjectId,
        parent_dir: Option<Uuid>,
    ) -> String {
        let obj = edit_context.objects().get(&object_id).unwrap();
        let properties = store_object_to_json_properties(obj);
        let mut stored_object = ObjectSourceDataStorageJsonObject {
            name: obj.object_name.as_string().cloned().unwrap_or_default(),
            parent_dir,
            schema: obj.schema.fingerprint().as_uuid(),
            schema_name: obj.schema.name().to_string(),
            prototype: obj
                .prototype
                .map(|x| Uuid::from_u128(x.0)),
            properties,
        };

        serde_json::to_string_pretty(&stored_object).unwrap()


        // name: String,
        // parent_dir: Option<Uuid>,
        // schema: Uuid,
        // schema_name: String,
        // prototype: Option<Uuid>,
        // #[serde(serialize_with = "ordered_map")]
        // properties: HashMap<String, serde_json::Value>,

    }
}
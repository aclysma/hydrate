use crate::edit_context::EditContext;
use crate::{
    BuildInfo, DataObjectInfo, HashMap, HashSet, HashSetIter, ImportInfo, ImporterId, NullOverride,
    ObjectId, ObjectLocation, ObjectName, ObjectSourceId, OverrideBehavior, Schema,
    SchemaFingerprint, SchemaNamedType, SchemaSet, SingleObject, Value,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
        Value::Bytes(x) => serde_json::Value::from(base64::encode(x)),
        Value::Buffer(_) => unimplemented!(),
        Value::String(x) => serde_json::Value::from(x.clone()),
        Value::StaticArray(_) => unimplemented!(),
        Value::DynamicArray(_) => unimplemented!(),
        Value::Map(_) => unimplemented!(),
        Value::ObjectRef(x) => serde_json::Value::from(x.as_uuid().to_string()),
        Value::Record(_) => unimplemented!(),
        Value::Enum(x) => serde_json::Value::from(x.symbol_name().to_string()),
        Value::Fixed(_) => unimplemented!(),
    }
}

fn json_to_property_value_with_schema(
    named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
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
        Schema::Bytes => Value::Bytes(base64::decode(value.as_str().unwrap()).unwrap()),
        Schema::Buffer => unimplemented!(),
        Schema::String => Value::String(value.as_str().unwrap().to_string()),
        Schema::StaticArray(_) => unimplemented!(),
        Schema::DynamicArray(_) => unimplemented!(),
        Schema::Map(_) => unimplemented!(),
        Schema::ObjectRef(_) => Value::ObjectRef(ObjectId(
            Uuid::parse_str(value.as_str().unwrap()).unwrap().as_u128(),
        )),
        Schema::NamedType(x) => {
            let named_type = named_types.get(x).unwrap();
            match named_type {
                SchemaNamedType::Record(_) => unimplemented!(),
                SchemaNamedType::Enum(e) => e.value_from_string(value.as_str().unwrap()).unwrap(),
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
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

fn ordered_map_json_value<S>(
    value: &HashMap<String, serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn ordered_map_uuid<S>(
    value: &HashMap<String, Uuid>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn load_json_properties(
    named_type: &SchemaNamedType,
    named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    json_properties: &HashMap<String, serde_json::Value>,
    properties: &mut HashMap<String, Value>,
    property_null_overrides: &mut HashMap<String, NullOverride>,
    mut properties_in_replace_mode: Option<&mut HashSet<String>>,
    dynamic_array_entries: &mut HashMap<String, HashSet<Uuid>>,
) {
    let mut max_path_length = 0;
    for (k, _) in json_properties {
        max_path_length = max_path_length.max(k.len());
    }

    for (path, value) in json_properties {
        let split_path = path.rsplit_once('.');
        //let parent_path = split_path.map(|x| x.0);
        //let path_end = split_path.map(|x| x.1);

        let mut property_handled = false;

        if let Some((parent_path, path_end)) = split_path {
            let parent_schema = named_type
                .find_property_schema(parent_path, named_types)
                .unwrap();
            if parent_schema.is_nullable() && path_end == "null_override" {
                let null_override = string_to_null_override_value(value.as_str().unwrap()).unwrap();
                //edit_context.set_null_override(object_id, path, null_override);
                log::trace!("set null override {} to {:?}", parent_path, null_override);
                property_null_overrides.insert(parent_path.to_string(), null_override);
                property_handled = true;
            }

            if parent_schema.is_dynamic_array() && path_end == "replace" {
                if let Some(properties_in_replace_mode) = &mut properties_in_replace_mode {
                    if value.as_bool() == Some(true) {
                        log::trace!("set property {} to replace", parent_path);
                        properties_in_replace_mode.insert(parent_path.to_string());
                    }
                }

                property_handled = true;
            }
        }

        if !property_handled {
            let property_schema = named_type.find_property_schema(&path, named_types).unwrap();
            if property_schema.is_dynamic_array() {
                let json_array = value.as_array().unwrap();
                for json_array_element in json_array {
                    let element = json_array_element.as_str().unwrap();
                    let element = Uuid::from_str(element).unwrap();
                    let existing_entries =
                        dynamic_array_entries.entry(path.to_string()).or_default();
                    if !existing_entries.contains(&element) {
                        log::trace!("add dynamic array element {} to {:?}", element, path);
                        existing_entries.insert(element);
                    }
                }
            } else {
                let v = json_to_property_value_with_schema(named_types, &property_schema, &value);
                log::trace!("set {} to {:?}", path, v);
                properties.insert(path.to_string(), v);
            }
        }
    }
}

fn store_json_properties(
    properties: &HashMap<String, Value>,
    property_null_overrides: &HashMap<String, NullOverride>,
    properties_in_replace_mode: Option<&HashSet<String>>,
    dynamic_array_entries: &HashMap<String, HashSet<Uuid>>,
) -> HashMap<String, serde_json::Value> {
    let mut saved_properties: HashMap<String, serde_json::Value> = Default::default();

    for (path, null_override) in property_null_overrides {
        saved_properties.insert(
            format!("{}.null_override", path),
            serde_json::Value::from(null_override_to_string_value(*null_override)),
        );
    }

    if let Some(properties_in_replace_mode) = properties_in_replace_mode {
        for path in properties_in_replace_mode {
            saved_properties.insert(format!("{}.replace", path), serde_json::Value::from(true));
        }
    }

    for (path, elements) in dynamic_array_entries {
        let mut sorted_elements: Vec<_> = elements.iter().copied().collect();
        sorted_elements.sort();
        let elements_json: Vec<_> = sorted_elements
            .iter()
            .map(|x| serde_json::Value::from(x.to_string()))
            .collect();
        let elements_json_array = serde_json::Value::from(elements_json);
        saved_properties.insert(path.to_string(), elements_json_array);
    }

    for (k, v) in properties {
        saved_properties.insert(k.to_string(), property_value_to_json(v));
    }

    saved_properties
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditContextObjectImportInfoJson {
    importer_id: Uuid,
    source_file_path: String,
    importable_name: String,
    file_references: Vec<PathBuf>,
}

impl EditContextObjectImportInfoJson {
    pub fn new(import_info: &ImportInfo) -> Self {
        let source_file_path = import_info.source_file_path.to_string_lossy().to_string();

        EditContextObjectImportInfoJson {
            importer_id: import_info.importer_id.0,
            source_file_path,
            importable_name: import_info.importable_name.clone(),
            file_references: import_info.file_references.clone(),
        }
    }

    pub fn to_import_info(
        &self,
        schema_set: &SchemaSet,
    ) -> ImportInfo {
        ImportInfo {
            importer_id: ImporterId(self.importer_id),
            source_file_path: PathBuf::from_str(&self.source_file_path).unwrap(),
            importable_name: self.importable_name.clone(),
            file_references: self.file_references.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditContextObjectBuildInfoJson {
    #[serde(serialize_with = "ordered_map_uuid")]
    file_reference_overrides: HashMap<String, Uuid>,
}

impl EditContextObjectBuildInfoJson {
    pub fn new(import_info: &BuildInfo) -> Self {
        let mut file_reference_overrides = HashMap::default();
        for (k, v) in &import_info.file_reference_overrides {
            file_reference_overrides.insert(k.to_string_lossy().to_string(), v.as_uuid());
        }

        EditContextObjectBuildInfoJson {
            file_reference_overrides,
        }
    }

    pub fn to_build_info(
        &self,
        schema_set: &SchemaSet,
    ) -> BuildInfo {
        let mut file_reference_overrides = HashMap::default();
        for (k, v) in &self.file_reference_overrides {
            file_reference_overrides.insert(PathBuf::from_str(k).unwrap(), ObjectId(v.as_u128()));
        }

        BuildInfo {
            file_reference_overrides,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditContextObjectJson {
    name: String,
    parent_dir: Option<Uuid>,
    schema: Uuid,
    schema_name: String,
    import_info: Option<EditContextObjectImportInfoJson>,
    build_info: EditContextObjectBuildInfoJson,
    prototype: Option<Uuid>,
    #[serde(serialize_with = "ordered_map_json_value")]
    properties: HashMap<String, serde_json::Value>,
}

impl EditContextObjectJson {
    pub fn load_edit_context_object_from_string(
        edit_context: &mut EditContext,
        object_id: Uuid,
        object_source_id: ObjectSourceId,
        json: &str,
    ) {
        let stored_object: EditContextObjectJson = serde_json::from_str(json).unwrap();
        let path_node_id = ObjectId(stored_object.parent_dir.unwrap_or(Uuid::nil()).as_u128());
        let object_location = ObjectLocation::new(object_source_id, path_node_id);
        let object_name = if stored_object.name.is_empty() {
            ObjectName::empty()
        } else {
            ObjectName::new(stored_object.name)
        };

        let object_id = ObjectId(object_id.as_u128());
        let schema_fingerprint = SchemaFingerprint(stored_object.schema.as_u128());
        let prototype = stored_object.prototype.map(|x| ObjectId(x.as_u128()));

        let named_type = edit_context.find_named_type_by_fingerprint(schema_fingerprint);

        let named_type = if let Some(named_type) = named_type {
            named_type.clone()
        } else {
            panic!("Can't load type {}", stored_object.schema_name);
        };

        let mut properties: HashMap<String, Value> = Default::default();
        let mut property_null_overrides: HashMap<String, NullOverride> = Default::default();
        let mut properties_in_replace_mode: HashSet<String> = Default::default();
        let mut dynamic_array_entries: HashMap<String, HashSet<Uuid>> = Default::default();

        load_json_properties(
            &named_type,
            edit_context.schemas(),
            &stored_object.properties,
            &mut properties,
            &mut property_null_overrides,
            Some(&mut properties_in_replace_mode),
            &mut dynamic_array_entries,
        );

        let import_info = stored_object
            .import_info
            .map(|x| x.to_import_info(edit_context.schema_set()));
        let build_info = stored_object
            .build_info
            .to_build_info(edit_context.schema_set());

        edit_context.restore_object(
            object_id,
            object_name,
            object_location,
            import_info,
            build_info,
            prototype,
            schema_fingerprint,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        );
    }

    pub fn save_edit_context_object_to_string(
        edit_context: &EditContext,
        object_id: ObjectId,
        parent_dir: Option<Uuid>,
    ) -> String {
        let obj = edit_context.objects().get(&object_id).unwrap();

        let json_properties = store_json_properties(
            &obj.properties,
            &obj.property_null_overrides,
            Some(&obj.properties_in_replace_mode),
            &obj.dynamic_array_entries,
        );

        let import_info = obj
            .import_info
            .as_ref()
            .map(|x| EditContextObjectImportInfoJson::new(&x));
        let build_info = EditContextObjectBuildInfoJson::new(&obj.build_info);

        let mut stored_object = EditContextObjectJson {
            name: obj.object_name.as_string().cloned().unwrap_or_default(),
            parent_dir,
            schema: obj.schema.fingerprint().as_uuid(),
            schema_name: obj.schema.name().to_string(),
            import_info,
            build_info,
            prototype: obj.prototype.map(|x| Uuid::from_u128(x.0)),
            properties: json_properties,
        };

        serde_json::to_string_pretty(&stored_object).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleObjectJson {
    schema: Uuid,
    schema_name: String,
    #[serde(serialize_with = "ordered_map_json_value")]
    properties: HashMap<String, serde_json::Value>,
}

impl SingleObjectJson {
    pub fn new(object: &SingleObject) -> SingleObjectJson {
        let json_properties = store_json_properties(
            &object.properties,
            &object.property_null_overrides,
            None,
            &object.dynamic_array_entries,
        );

        SingleObjectJson {
            schema: object.schema.fingerprint().as_uuid(),
            schema_name: object.schema.name().to_string(),
            properties: json_properties,
        }
    }

    pub fn to_single_object(
        &self,
        schema_set: &SchemaSet,
    ) -> SingleObject {
        let schema_fingerprint = SchemaFingerprint(self.schema.as_u128());

        let named_type = schema_set
            .find_named_type_by_fingerprint(schema_fingerprint)
            .unwrap()
            .clone();

        let mut properties: HashMap<String, Value> = Default::default();
        let mut property_null_overrides: HashMap<String, NullOverride> = Default::default();
        let mut dynamic_array_entries: HashMap<String, HashSet<Uuid>> = Default::default();

        load_json_properties(
            &named_type,
            schema_set.schemas(),
            &self.properties,
            &mut properties,
            &mut property_null_overrides,
            None,
            &mut dynamic_array_entries,
        );

        SingleObject::restore(
            schema_set,
            schema_fingerprint,
            properties,
            property_null_overrides,
            dynamic_array_entries,
        )
    }

    pub fn load_single_object_from_string(
        schema_set: &SchemaSet,
        json: &str,
    ) -> SingleObject {
        let stored_object: SingleObjectJson = serde_json::from_str(json).unwrap();
        stored_object.to_single_object(schema_set)
    }

    pub fn save_single_object_to_string(object: &SingleObject) -> String {
        let stored_object = SingleObjectJson::new(object);
        serde_json::to_string_pretty(&stored_object).unwrap()
    }
}

use crate::{AssetId, BuildInfo, DataSetAssetInfo, HashMap, HashSet, ImportInfo, ImporterId, NullOverride, PathReference, Schema, SchemaFingerprint, SchemaNamedType, SchemaSet, SingleObject, Value, PathReferenceNamespaceResolver};
use crate::{AssetLocation, AssetName, DataSetResult, ImportableName, OrderedSet};
use hydrate_schema::DataSetError;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

fn property_value_to_json(
    value: &Value,
    buffers: &mut Option<Vec<Arc<Vec<u8>>>>,
) -> serde_json::Value {
    match value {
        Value::Nullable(_) => unimplemented!(),
        Value::Boolean(x) => serde_json::Value::from(*x),
        Value::I32(x) => serde_json::Value::from(*x),
        Value::I64(x) => serde_json::Value::from(*x),
        Value::U32(x) => serde_json::Value::from(*x),
        Value::U64(x) => serde_json::Value::from(*x),
        Value::F32(x) => serde_json::Value::from(*x),
        Value::F64(x) => serde_json::Value::from(*x),
        Value::Bytes(x) => {
            if let Some(buffers) = buffers {
                // Copy the data into a new buffer and create a json value that indexes into it
                let buffer_index = buffers.len();
                buffers.push(x.clone());
                serde_json::Value::from(buffer_index)
            } else {
                // Encode the data inline as a base64 string
                serde_json::Value::from(base64::encode(&**x))
            }
        }
        Value::String(x) => serde_json::Value::from(x.to_string()),
        Value::StaticArray(_) => unimplemented!(),
        Value::DynamicArray(_) => unimplemented!(),
        Value::Map(_) => unimplemented!(),
        Value::AssetRef(x) => serde_json::Value::from(x.as_uuid().to_string()),
        Value::Record(_) => unimplemented!(),
        Value::Enum(x) => serde_json::Value::from(x.symbol_name().to_string()),
    }
}

fn json_to_property_value_with_schema(
    named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    schema: &Schema,
    value: &serde_json::Value,
    buffers: &Option<Vec<Arc<Vec<u8>>>>,
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
        Schema::Bytes => {
            if let Some(buffers) = buffers {
                // The data is an index into a buffer, take the data from the buffer
                let buffer_index = value.as_u64().unwrap() as usize;
                Value::Bytes(buffers[buffer_index].clone())
            } else {
                // The data is encoded inline as a base64 string, decode and return the value
                let data = base64::decode(value.as_str().unwrap()).unwrap();
                Value::Bytes(Arc::new(data))
            }
        }
        Schema::String => Value::String(Arc::new(value.as_str().unwrap().to_string())),
        Schema::StaticArray(_) => unimplemented!(),
        Schema::DynamicArray(_) => unimplemented!(),
        Schema::Map(_) => unimplemented!(),
        Schema::AssetRef(_) => Value::AssetRef(AssetId::from_uuid(
            Uuid::parse_str(value.as_str().unwrap()).unwrap(),
        )),
        Schema::Record(_) => unimplemented!(),
        Schema::Enum(x) => {
            let named_type = named_types.get(x).unwrap();
            match named_type {
                SchemaNamedType::Record(_) => {
                    panic!("A Schema::Enum is matching a named type that is not an enum")
                }
                SchemaNamedType::Enum(e) => {
                    Value::enum_value_from_string(e, value.as_str().unwrap()).unwrap()
                }
            }
        }
    }
}

fn null_override_to_string_value(null_override: NullOverride) -> &'static str {
    match null_override {
        NullOverride::SetNull => "SetNull",
        NullOverride::SetNonNull => "SetNonNull",
        NullOverride::Unset => unreachable!(), // Should not be in the map
    }
}

fn string_to_null_override_value(s: &str) -> Option<NullOverride> {
    match s {
        "SetNull" => Some(NullOverride::SetNull),
        "SetNonNull" => Some(NullOverride::SetNonNull),
        _ => None,
    }
}

fn ordered_map_file_references<S>(
    value: &HashMap<Uuid, String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
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
    dynamic_collection_entries: &mut HashMap<String, OrderedSet<Uuid>>,
    buffers: &mut Option<Vec<Arc<Vec<u8>>>>,
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
            if property_schema.is_dynamic_array() || property_schema.is_map() {
                let json_array = value.as_array().unwrap();
                for json_array_element in json_array {
                    let element = json_array_element.as_str().unwrap();
                    let element = Uuid::from_str(element).unwrap();
                    let existing_entries = dynamic_collection_entries
                        .entry(path.to_string())
                        .or_default();
                    if !existing_entries.contains(&element) {
                        log::trace!("add dynamic array element {} to {:?}", element, path);
                        let newly_inserted = existing_entries.try_insert_at_end(element);
                        assert!(newly_inserted);
                    }
                }
            } else {
                let v = json_to_property_value_with_schema(
                    named_types,
                    &property_schema,
                    &value,
                    buffers,
                );
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
    dynamic_collection_entries: &HashMap<String, OrderedSet<Uuid>>,
    buffers: &mut Option<Vec<Arc<Vec<u8>>>>,
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

    for (path, elements) in dynamic_collection_entries {
        let elements_json: Vec<_> = elements
            .iter()
            .map(|x| serde_json::Value::from(x.to_string()))
            .collect();
        let elements_json_array = serde_json::Value::from(elements_json);
        saved_properties.insert(path.to_string(), elements_json_array);
    }

    for (k, v) in properties {
        saved_properties.insert(k.to_string(), property_value_to_json(v, buffers));
    }

    saved_properties
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReferenceJson {
    reference_hash: Uuid,
    canonical_path_reference: String,
}

// Import Info, part of AssetJson
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetImportInfoJson {
    importer_id: Uuid,

    //source_file_root: String,
    source_file_path: String,
    importable_name: String,

    #[serde(serialize_with = "ordered_map_file_references")]
    file_references: HashMap<Uuid, String>,

    // These are all encoded as hex to avoid json/u64 weirdness
    source_file_modified_timestamp: String,
    source_file_size: String,
    import_data_contents_hash: String,
}

impl AssetImportInfoJson {
    pub fn new(
        import_info: &ImportInfo
    ) -> Self {
        let source_file_path = format!("{}", PathReference::new(import_info.source_file().namespace().to_string(), import_info.source_file().path().to_string(), ImportableName::default()));

        AssetImportInfoJson {
            importer_id: import_info.importer_id().0,
            source_file_path,
            importable_name: import_info
                .importable_name()
                .name()
                .map(|x| x.to_string())
                .unwrap_or_default(),
            file_references: import_info
                .file_references()
                .iter()
                .map(|(k, v)| {
                    (*k, v.to_string())
                })
                .collect(),
            source_file_modified_timestamp: format!(
                "{:0>16x}",
                import_info.source_file_modified_timestamp()
            ),
            source_file_size: format!("{:0>16x}", import_info.source_file_size()),
            import_data_contents_hash: format!("{:0>16x}", import_info.import_data_contents_hash()),
        }
    }

    pub fn to_import_info(
        &self,
        _schema_set: &SchemaSet,
        namespace_resolver: &dyn PathReferenceNamespaceResolver,

    ) -> DataSetResult<ImportInfo> {
        let mut path_references = HashMap::default();
        for (key, value) in &self.file_references {
            let path_reference: PathReference = value.into();
            path_references.insert(*key, path_reference.simplify(namespace_resolver));
        }

        let mut path_reference: PathReference = self.source_file_path.clone().into();
        let source_file = PathReference::new(path_reference.namespace().to_string(), path_reference.path().to_string(), ImportableName::new(self.importable_name.clone())).simplify(namespace_resolver);

        let source_file_modified_timestamp =
            u64::from_str_radix(&self.source_file_modified_timestamp, 16)
                .map_err(|_| (DataSetError::StorageFormatError))?;
        let source_file_size = u64::from_str_radix(&self.source_file_size, 16)
            .map_err(|_| (DataSetError::StorageFormatError))?;
        let import_data_contents_hash = u64::from_str_radix(&self.import_data_contents_hash, 16)
            .map_err(|_| (DataSetError::StorageFormatError))?;

        Ok(ImportInfo::new(
            ImporterId(self.importer_id),
            source_file,
            path_references,
            source_file_modified_timestamp,
            source_file_size,
            import_data_contents_hash,
        ))
    }
}

// Build Info, part of AssetJson
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetBuildInfoJson {
    #[serde(serialize_with = "ordered_map_uuid")]
    file_reference_overrides: HashMap<String, Uuid>,
}

impl AssetBuildInfoJson {
    pub fn new(import_info: &BuildInfo) -> Self {
        let mut file_reference_overrides = HashMap::default();
        for (k, v) in &import_info.file_reference_overrides {
            file_reference_overrides.insert(k.to_string(), v.as_uuid());
        }

        AssetBuildInfoJson {
            file_reference_overrides,
        }
    }

    pub fn to_build_info(
        &self,
        _schema_set: &SchemaSet,
        namespace_resolver: &dyn PathReferenceNamespaceResolver,
    ) -> BuildInfo {
        let mut file_reference_overrides = HashMap::default();
        for (k, v) in &self.file_reference_overrides {
            let path_reference: PathReference = k.into();
            file_reference_overrides.insert(path_reference.simplify(namespace_resolver), AssetId::from_uuid(*v));
        }

        BuildInfo {
            file_reference_overrides,
        }
    }
}

pub trait RestoreAssetFromStorageImpl {
    fn restore_asset(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
        asset_location: AssetLocation,
        import_info: Option<ImportInfo>,
        build_info: BuildInfo,
        prototype: Option<AssetId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_collection_entries: HashMap<String, OrderedSet<Uuid>>,
    ) -> DataSetResult<()>;

    fn namespace_resolver(&self) -> &dyn PathReferenceNamespaceResolver;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetJson {
    id: Option<Uuid>,
    name: String,
    parent_dir: Option<Uuid>,
    schema: Uuid,
    schema_name: String,
    import_info: Option<AssetImportInfoJson>,
    build_info: AssetBuildInfoJson,
    prototype: Option<Uuid>,
    #[serde(serialize_with = "ordered_map_json_value")]
    properties: HashMap<String, serde_json::Value>,
}

impl AssetJson {
    #[profiling::function]
    pub fn load_asset_from_string(
        restore_asset_impl: &mut dyn RestoreAssetFromStorageImpl,
        schema_set: &SchemaSet,
        override_asset_id: Option<Uuid>,
        // If the file doesn't claim a location and we don't override it, we will default to this
        default_asset_location: AssetLocation,
        // If set, we use this instead of what the file says to use
        override_asset_location: Option<AssetLocation>,
        json: &str,
    ) -> DataSetResult<AssetId> {
        let stored_asset: AssetJson = {
            profiling::scope!("serde_json::from_str");
            serde_json::from_str(json).unwrap()
        };

        // Use the provided override, or what's in the file, or worst case default to asset_source_id
        let asset_location = if let Some(override_asset_location) = override_asset_location {
            override_asset_location
        } else {
            // If no parent is specified, default it to the root node for this data source
            stored_asset
                .parent_dir
                .map(|x| AssetLocation::new(AssetId::from_uuid(x)))
                .unwrap_or(default_asset_location)
        };

        let asset_name = if stored_asset.name.is_empty() {
            AssetName::empty()
        } else {
            AssetName::new(stored_asset.name)
        };

        let asset_id = if let Some(override_asset_id) = override_asset_id {
            // If an ID was provided, use it
            AssetId::from_uuid(override_asset_id)
        } else {
            // Otherwise read it from the file. If there was no ID specified, generate a new one
            AssetId::from_uuid(stored_asset.id.unwrap_or_else(Uuid::new_v4))
        };

        let schema_fingerprint = SchemaFingerprint::from_uuid(stored_asset.schema);
        let prototype = stored_asset.prototype.map(|x| AssetId::from_uuid(x));

        let named_type = schema_set.find_named_type_by_fingerprint(schema_fingerprint);

        let named_type = if let Some(named_type) = named_type {
            named_type.clone()
        } else {
            log::error!("Can't load type {} by fingerprint, trying by name. Schema migration not yet implemented", stored_asset.schema_name);
            schema_set
                .find_named_type(stored_asset.schema_name)?
                .clone()

            //Fingerprint doesn't match, this may need to be a data migration in the future
            //panic!("Can't load type {}", stored_asset.schema_name);
        };

        let mut properties: HashMap<String, Value> = Default::default();
        let mut property_null_overrides: HashMap<String, NullOverride> = Default::default();
        let mut properties_in_replace_mode: HashSet<String> = Default::default();
        let mut dynamic_collection_entries: HashMap<String, OrderedSet<Uuid>> = Default::default();
        let mut buffers = None;

        load_json_properties(
            &named_type,
            schema_set.schemas(),
            &stored_asset.properties,
            &mut properties,
            &mut property_null_overrides,
            Some(&mut properties_in_replace_mode),
            &mut dynamic_collection_entries,
            &mut buffers,
        );

        let import_info = if let Some(import_info) = stored_asset.import_info {
            Some(import_info.to_import_info(schema_set, restore_asset_impl.namespace_resolver())?)
        } else {
            None
        };

        let build_info = stored_asset.build_info.to_build_info(schema_set, restore_asset_impl.namespace_resolver());

        restore_asset_impl.restore_asset(
            asset_id,
            asset_name,
            asset_location,
            import_info,
            build_info,
            prototype,
            named_type.fingerprint(),
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_collection_entries,
        )?;

        Ok(asset_id)
    }

    #[profiling::function]
    pub fn save_asset_to_string(
        assets: &HashMap<AssetId, DataSetAssetInfo>,
        asset_id: AssetId,
        // We only save the ID in the file if using path-based file system storage. Otherwise the
        // id is the file path/name
        include_asset_id_in_file: bool,
        asset_location: Option<AssetLocation>,
    ) -> String {
        let obj = assets.get(&asset_id).unwrap();
        let mut buffers = None;

        let json_properties = store_json_properties(
            obj.properties(),
            obj.property_null_overrides(),
            Some(obj.properties_in_replace_mode()),
            obj.dynamic_collection_entries(),
            &mut buffers,
        );

        let import_info = obj
            .import_info()
            .as_ref()
            .map(|x| AssetImportInfoJson::new(&x));
        let build_info = AssetBuildInfoJson::new(obj.build_info());

        let written_asset_id = if include_asset_id_in_file {
            Some(asset_id.as_uuid())
        } else {
            None
        };
        let stored_asset = AssetJson {
            id: written_asset_id,
            name: obj.asset_name().as_string().cloned().unwrap_or_default(),
            parent_dir: asset_location.map(|x| x.path_node_id().as_uuid()),
            schema: obj.schema().fingerprint().as_uuid(),
            schema_name: obj.schema().name().to_string(),
            import_info,
            build_info,
            prototype: obj.prototype().map(|x| x.as_uuid()),
            properties: json_properties,
        };

        profiling::scope!("serde_json::to_string_pretty");
        serde_json::to_string_pretty(&stored_asset).unwrap()
    }
}

// You can create this with SingleObjectJson::new and serialize it to disk to save
// You can deserialize this and read using SingleObjectJson::to_single_object
#[derive(Debug, Serialize, Deserialize)]
pub struct SingleObjectJson {
    //contents_hash: u64,
    schema: Uuid,
    schema_name: String,
    #[serde(serialize_with = "ordered_map_json_value")]
    properties: HashMap<String, serde_json::Value>,
}

impl SingleObjectJson {
    pub fn new(
        object: &SingleObject,
        // If buffers are provided, the bulk data is stored here instead of inline with the rest of the properties
        buffers: &mut Option<Vec<Arc<Vec<u8>>>>,
    ) -> SingleObjectJson {
        let json_properties = store_json_properties(
            &object.properties(),
            &object.property_null_overrides(),
            None,
            &object.dynamic_collection_entries(),
            buffers,
        );

        let mut hasher = siphasher::sip::SipHasher::default();
        // This includes schema, all other contents of the asset
        object.hash(&mut hasher);

        SingleObjectJson {
            //contents_hash: hasher.finish().into(),
            schema: object.schema().fingerprint().as_uuid(),
            schema_name: object.schema().name().to_string(),
            properties: json_properties,
        }
    }

    pub fn to_single_object(
        &self,
        schema_set: &SchemaSet,
        // If buffers are provided, then we read bulk data from here instead from inline
        buffers: &mut Option<Vec<Arc<Vec<u8>>>>,
    ) -> SingleObject {
        let schema_fingerprint = SchemaFingerprint::from_uuid(self.schema);

        let named_type = schema_set
            .find_named_type_by_fingerprint(schema_fingerprint)
            .unwrap()
            .clone();

        let mut properties: HashMap<String, Value> = Default::default();
        let mut property_null_overrides: HashMap<String, NullOverride> = Default::default();
        let mut dynamic_collection_entries: HashMap<String, OrderedSet<Uuid>> = Default::default();

        load_json_properties(
            &named_type,
            schema_set.schemas(),
            &self.properties,
            &mut properties,
            &mut property_null_overrides,
            None,
            &mut dynamic_collection_entries,
            buffers,
        );

        SingleObject::restore(
            schema_set,
            schema_fingerprint,
            properties,
            property_null_overrides,
            dynamic_collection_entries,
        )
    }
}

#[derive(Default, Clone)]
pub struct MetaFile {
    pub past_id_assignments: HashMap<ImportableName, AssetId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaFileJson {
    #[serde(serialize_with = "ordered_map_uuid")]
    pub past_id_assignments: HashMap<String, Uuid>,
}

impl MetaFileJson {
    #[profiling::function]
    pub fn load_from_string(json: &str) -> MetaFile {
        let meta_file: MetaFileJson = {
            profiling::scope!("serde_json::from_str");
            serde_json::from_str(json).unwrap()
        };
        let mut past_id_assignments = HashMap::default();
        for past_id_assignment in meta_file.past_id_assignments {
            past_id_assignments.insert(
                ImportableName::new(past_id_assignment.0),
                AssetId::from_uuid(past_id_assignment.1),
            );
        }

        MetaFile {
            past_id_assignments,
        }
    }

    #[profiling::function]
    pub fn store_to_string(meta_file: &MetaFile) -> String {
        let mut past_id_assignments = HashMap::default();
        for past_id_assignment in &meta_file.past_id_assignments {
            past_id_assignments.insert(
                past_id_assignment
                    .0
                    .name()
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                past_id_assignment.1.as_uuid(),
            );
        }

        let json_object = MetaFileJson {
            past_id_assignments,
        };
        profiling::scope!("serde_json::to_string_pretty");
        serde_json::to_string_pretty(&json_object).unwrap()
    }
}


use std::hash::{Hash, Hasher};
use std::path::Path;
use siphasher::sip128::Hasher128;
use uuid::Uuid;
use crate::{SchemaFingerprint, Value, HashMap, Schema, HashSet};

#[derive(Debug)]
pub enum SchemaFromFileParserError {
    Str(&'static str),
    String(String)
}

pub type SchemaFromFileParserResult<T> = Result<T, SchemaFromFileParserError>;

#[derive(Debug)]
pub(super) struct SchemaFromFileStaticArray {
    pub(super) item_type: Box<SchemaFromFileTypeRef>,
    pub(super) length: usize
}

impl SchemaFromFileStaticArray {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        self.item_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        self.item_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.item_type.partial_hash(hasher);
        self.length.hash(hasher);
    }
}

#[derive(Debug)]
pub(super) struct SchemaFromFileDynamicArray {
    pub(super) item_type: Box<SchemaFromFileTypeRef>
}

impl SchemaFromFileDynamicArray {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        self.item_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        self.item_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.item_type.partial_hash(hasher);
    }
}

#[derive(Debug)]
pub(super) struct SchemaFromFileMap {
    pub(super) key_type: Box<SchemaFromFileTypeRef>,
    pub(super) value_type: Box<SchemaFromFileTypeRef>
}

impl SchemaFromFileMap {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        self.key_type.apply_type_aliases(aliases);
        self.value_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        self.key_type.collect_all_related_types(types);
        self.value_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.key_type.partial_hash(hasher);
        self.value_type.partial_hash(hasher);
    }
}

#[derive(Debug)]
pub(super) struct SchemaFromFileRecordField {
    pub(super) field_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) field_type: Box<SchemaFromFileTypeRef>
}

impl SchemaFromFileRecordField {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        self.field_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        self.field_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.field_name.hash(hasher);
        self.field_type.partial_hash(hasher);
    }
}

//TODO: Verify we don't have dupe field names
#[derive(Debug)]
pub(super) struct SchemaFromFileRecord {
    pub(super) type_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) fields: Vec<SchemaFromFileRecordField>,
}

impl SchemaFromFileRecord {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        for field in &mut self.fields {
            field.apply_type_aliases(aliases);
        }
    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        types.insert(self.type_name.clone());
        for field in &self.fields {
            field.collect_all_related_types(types);
        }
    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.type_name.hash(hasher);

        let mut sorted_fields: Vec<_> = self.fields.iter().collect();
        sorted_fields.sort_by_key(|x| &x.field_name);

        for field in sorted_fields {
            field.partial_hash(hasher);
        }
    }
}

#[derive(Debug)]
pub(super) struct SchemaFromFileEnumSymbol {
    pub(super) symbol_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) value: i32
}

impl SchemaFromFileEnumSymbol {
    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.symbol_name.hash(hasher);
        self.value.hash(hasher);
    }
}

//TODO: Verify that we don't have dupe symbol names or values
#[derive(Debug)]
pub(super) struct SchemaFromFileEnum {
    pub(super) type_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) symbols: Vec<SchemaFromFileEnumSymbol>,
}

impl SchemaFromFileEnum {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {

    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        types.insert(self.type_name.clone());

    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.type_name.hash(hasher);

        let mut sorted_symbols: Vec<_> = self.symbols.iter().collect();
        sorted_symbols.sort_by_key(|x| x.value);

        for symbol in sorted_symbols {
            symbol.partial_hash(hasher);
        }
    }
}

#[derive(Debug)]
pub(super) struct SchemaFromFileFixed {
    pub(super) type_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) length: usize
}

impl SchemaFromFileFixed {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {

    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        types.insert(self.type_name.clone());

    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        self.type_name.hash(hasher);
        self.length.hash(hasher);
    }
}

#[derive(Debug)]
pub(super) enum SchemaFromFileTypeRef {
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
    NamedType(String)
    // Record(SchemaFromFileTypeName),
    // Enum(SchemaFromFileTypeName),
    // Fixed(SchemaFromFileTypeName),
    // union?
}

impl SchemaFromFileTypeRef {
    fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        match self {
            SchemaFromFileTypeRef::Nullable(x) => x.apply_type_aliases(aliases),
            SchemaFromFileTypeRef::Boolean => {}
            SchemaFromFileTypeRef::I32 => {}
            SchemaFromFileTypeRef::I64 => {}
            SchemaFromFileTypeRef::U32 => {}
            SchemaFromFileTypeRef::U64 => {}
            SchemaFromFileTypeRef::F32 => {}
            SchemaFromFileTypeRef::F64 => {}
            SchemaFromFileTypeRef::Bytes => {}
            SchemaFromFileTypeRef::Buffer => {}
            SchemaFromFileTypeRef::String => {}
            SchemaFromFileTypeRef::StaticArray(x) => x.apply_type_aliases(aliases),
            SchemaFromFileTypeRef::DynamicArray(x) => x.apply_type_aliases(aliases),
            SchemaFromFileTypeRef::Map(x) => x.apply_type_aliases(aliases),
            SchemaFromFileTypeRef::NamedType(x) => {
                let alias = aliases.get(x);
                if let Some(alias) = alias {
                    *x = alias.clone();
                }
            },
        }
    }

    fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        match self {
            SchemaFromFileTypeRef::Nullable(x) => x.collect_all_related_types(types),
            SchemaFromFileTypeRef::Boolean => {}
            SchemaFromFileTypeRef::I32 => {}
            SchemaFromFileTypeRef::I64 => {}
            SchemaFromFileTypeRef::U32 => {}
            SchemaFromFileTypeRef::U64 => {}
            SchemaFromFileTypeRef::F32 => {}
            SchemaFromFileTypeRef::F64 => {}
            SchemaFromFileTypeRef::Bytes => {}
            SchemaFromFileTypeRef::Buffer => {}
            SchemaFromFileTypeRef::String => {}
            SchemaFromFileTypeRef::StaticArray(x) => x.collect_all_related_types(types),
            SchemaFromFileTypeRef::DynamicArray(x) => x.collect_all_related_types(types),
            SchemaFromFileTypeRef::Map(x) => x.collect_all_related_types(types),
            SchemaFromFileTypeRef::NamedType(x) => {
                types.insert(x.clone());
            },
        }
    }

    fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        match self {
            SchemaFromFileTypeRef::Nullable(x) => {
                "Nullable".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaFromFileTypeRef::Boolean => "Boolean".hash(hasher),
            SchemaFromFileTypeRef::I32 => "I32".hash(hasher),
            SchemaFromFileTypeRef::I64 => "I64".hash(hasher),
            SchemaFromFileTypeRef::U32 => "U32".hash(hasher),
            SchemaFromFileTypeRef::U64 => "U64".hash(hasher),
            SchemaFromFileTypeRef::F32 => "F32".hash(hasher),
            SchemaFromFileTypeRef::F64 => "F64".hash(hasher),
            SchemaFromFileTypeRef::Bytes => "Bytes".hash(hasher),
            SchemaFromFileTypeRef::Buffer => "Buffer".hash(hasher),
            SchemaFromFileTypeRef::String => "String".hash(hasher),
            SchemaFromFileTypeRef::StaticArray(x) => {
                "StaticArray".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaFromFileTypeRef::DynamicArray(x) => {
                "DynamicArray".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaFromFileTypeRef::Map(x) => {
                "Map".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaFromFileTypeRef::NamedType(x) => {
                "NamedType".hash(hasher);
                x.hash(hasher);
            }
        }
    }
}

pub(super) enum SchemaFromFileNamedType {
    Record(SchemaFromFileRecord),
    Enum(SchemaFromFileEnum),
    Fixed(SchemaFromFileFixed),
}

impl SchemaFromFileNamedType {
    pub(super) fn type_name(&self) -> &str {
        match self {
            SchemaFromFileNamedType::Record(x) => &x.type_name,
            SchemaFromFileNamedType::Enum(x) => &x.type_name,
            SchemaFromFileNamedType::Fixed(x) => &x.type_name
        }
    }

    pub(super) fn aliases(&self) -> &[String] {
        match self {
            SchemaFromFileNamedType::Record(x) => &x.aliases,
            SchemaFromFileNamedType::Enum(x) => &x.aliases,
            SchemaFromFileNamedType::Fixed(x) => &x.aliases
        }
    }

    pub(super) fn apply_type_aliases(&mut self, aliases: &HashMap<String, String>) {
        match self {
            SchemaFromFileNamedType::Record(x) => x.apply_type_aliases(aliases),
            SchemaFromFileNamedType::Enum(x) => x.apply_type_aliases(aliases),
            SchemaFromFileNamedType::Fixed(x) => x.apply_type_aliases(aliases),
        }
    }

    pub(super) fn collect_all_related_types(&self, types: &mut HashSet<String>) {
        match self {
            SchemaFromFileNamedType::Record(x) => x.collect_all_related_types(types),
            SchemaFromFileNamedType::Enum(x) => x.collect_all_related_types(types),
            SchemaFromFileNamedType::Fixed(x) => x.collect_all_related_types(types),
        }
    }

    pub(super) fn partial_hash<T: Hasher>(&self, hasher: &mut T) {
        match self {
            SchemaFromFileNamedType::Record(x) => {
                "record".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaFromFileNamedType::Enum(x) => {
                "enum".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaFromFileNamedType::Fixed(x) => {
                "fixed".hash(hasher);
                x.partial_hash(hasher);
            }
        }
    }
}

fn parse_json_schema_type_ref(json_value: &serde_json::Value, error_prefix: &str) -> SchemaFromFileParserResult<SchemaFromFileTypeRef> {
    let mut name;
    match json_value {
        serde_json::Value::String(type_name) => {
            name = type_name.as_str();
        },
        serde_json::Value::Object(json_object) => {
            name= json_object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Record field types must have a name", error_prefix)))?;
        },
        _ => return Err(SchemaFromFileParserError::String(format!("{}Type references must be a string or json object", error_prefix)))
    }

    Ok(match name {
        "nullable" => {
            let inner_type = json_value.get("inner_type").ok_or_else(|| SchemaFromFileParserError::String(format!("{}All nullable types must has an inner_type", error_prefix)))?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;

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
            let inner_type = json_value.get("inner_type").ok_or_else(|| SchemaFromFileParserError::String(format!("{}All dynamic_array types must has an inner_type", error_prefix)))?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;

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
        _ => SchemaFromFileTypeRef::NamedType(name.to_string())
    })
}

// fn parse_alias_list(json_value: &serde_json::Value, error_prefix: &str) -> SchemaFromFileParserResult<Vec<String>> {
//     let values = json_value.as_array().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Aliases must be an array of strings", error_prefix)))?;
//
//     let mut strings = Vec::with_capacity(values.len());
//     for value in values {
//         strings.push(value.as_str().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Aliases must be an array of strings", error_prefix)))?.to_string());
//     }
//
//     Ok(strings)
// }

fn parse_json_schema_record_field(json_object: &serde_json::Value, error_prefix: &str) -> SchemaFromFileParserResult<SchemaFromFileRecordField> {
    let object = json_object.as_object().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Record schema fields must be a json object", error_prefix)))?;

    let field_name = object.get("name").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Record fields must be a name that is a string", error_prefix)))?.to_string();
    let json_aliases = object.get("aliases").map(|x| x.as_array()).flatten();
    let mut aliases = vec![];
    if let Some(json_aliases) = json_aliases {
        for json_alias in json_aliases {
            aliases.push(json_alias.as_str().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Fields's aliases must be strings", error_prefix)))?.to_string())
        }
    }
    // let aliases = if let Some(json_aliases) = object.get("aliases") {
    //     Self::parse_alias_list(json_aliases, error_prefix)?
    // } else {
    //     vec![]
    // };
    let error_prefix = format!("{}[Field {}]", error_prefix, field_name);
    //let field_schema = object.get("type").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaFromFileParserError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();
    //let field_schema = object.get("type").ok_or_else(|| SchemaFromFileParserError::Str("Schema file record fields must have a type of string or json object"))?;
    let field_type_json_value = object.get("type").ok_or_else(|| SchemaFromFileParserError::String(format!("{}Record fields must have a type of string or json object", error_prefix)))?;
    let field_type = parse_json_schema_type_ref(field_type_json_value, &error_prefix)?;

    Ok(SchemaFromFileRecordField {
        field_name,
        aliases,
        field_type: Box::new(field_type)
    })
}

fn parse_json_schema_record(json_object: &serde_json::Map<String, serde_json::Value>, error_prefix: &str) -> SchemaFromFileParserResult<SchemaFromFileRecord> {
    let name = json_object.get("name").ok_or_else(|| SchemaFromFileParserError::String(format!("{}Records must have a name", error_prefix)))?;
    let name_str = name.as_str().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Records must have a name", error_prefix)))?;

    let error_prefix = format!("{}[Record {}]", error_prefix, name_str);
    log::debug!("Parsing record named '{}'", name_str);

    let json_aliases = json_object.get("aliases").map(|x| x.as_array()).flatten();
    let mut aliases = vec![];
    if let Some(json_aliases) = json_aliases {
        for json_alias in json_aliases {
            aliases.push(json_alias.as_str().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Record's aliases must be strings", error_prefix)))?.to_string())
        }
    }

    let json_fields = json_object.get("fields").map(|x| x.as_array()).flatten().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Records must have an array of fields", error_prefix)))?;
    let mut fields = vec![];
    for json_field in json_fields {
        fields.push(parse_json_schema_record_field(json_field, &error_prefix)?);
    }

    Ok(SchemaFromFileRecord {
        type_name: name_str.to_string(),
        aliases,
        fields
    })
}

pub(super) fn parse_json_schema(json_value: &serde_json::Value, error_prefix: &str) -> SchemaFromFileParserResult<SchemaFromFileNamedType> {
    let object = json_value.as_object().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Schema file must be an array of json objects", error_prefix)))?;

    let object_type = object.get("type").ok_or_else(|| SchemaFromFileParserError::String(format!("{}Schema file objects must have a type field", error_prefix)))?;
    let object_type_str = object_type.as_str().ok_or_else(|| SchemaFromFileParserError::String(format!("{}Schema file objects must have a type field that is a string", error_prefix)))?;
    match object_type_str {
        "record" => {
            let record = parse_json_schema_record(object, error_prefix)?;
            return Ok(SchemaFromFileNamedType::Record(record))

            //self.types.insert(record.type_name, NamedType::Record())



            //self.records.push(record);
        },
        _ => Err(SchemaFromFileParserError::String(format!("Schema file object has a type field that is unrecognized {:?}", object_type_str)))?
    }

    //Ok(())
}
use crate::AssetId;
use crate::{HashMap, Schema, SchemaFingerprint, SchemaNamedType, SchemaSet};
use std::hash::{Hash, Hasher};

use hydrate_schema::SchemaEnum;

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Boolean(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Vec<u8>),
    String(String),
    AssetRef(AssetId),
    Enum(ValueEnum),
    Fixed(Box<[u8]>),
}

impl PropertyValue {
    pub fn as_value(&self) -> Value {
        match self {
            PropertyValue::Boolean(x) => Value::Boolean(*x),
            PropertyValue::I32(x) => Value::I32(*x),
            PropertyValue::I64(x) => Value::I64(*x),
            PropertyValue::U32(x) => Value::U32(*x),
            PropertyValue::U64(x) => Value::U64(*x),
            PropertyValue::F32(x) => Value::F32(*x),
            PropertyValue::F64(x) => Value::F64(*x),
            PropertyValue::Bytes(x) => Value::Bytes(x.clone()),
            PropertyValue::String(x) => Value::String(x.clone()),
            PropertyValue::AssetRef(x) => Value::AssetRef(*x),
            PropertyValue::Enum(x) => Value::Enum(x.clone()),
            PropertyValue::Fixed(x) => Value::Fixed(x.clone()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValueMap {
    properties: HashMap<Value, Value>,
}

impl Hash for ValueMap {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        let mut hash = 0;
        for (k, v) in &self.properties {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            k.hash(&mut inner_hasher);
            v.hash(&mut inner_hasher);
            hash = hash ^ inner_hasher.finish();
        }

        hash.hash(state);
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValueRecord {
    properties: HashMap<String, Value>,
}

impl Hash for ValueRecord {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        let mut hash = 0;
        for (k, v) in &self.properties {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            k.hash(&mut inner_hasher);
            v.hash(&mut inner_hasher);
            hash = hash ^ inner_hasher.finish();
        }

        hash.hash(state);
    }
}

/*
impl ValueRecord {
    // fn get_property(&self, property_name: impl AsRef<str>) -> Option<&Value> {
    //     self.properties.get(property_name.as_ref())
    // }

    pub fn find_property_value(&self, property_name: impl AsRef<str>) -> Option<&Value> {
        self.properties.get(property_name.as_ref())
    }
}
*/
#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct ValueEnum {
    //symbol_index: u32,
    symbol_name: String,
}

impl ValueEnum {
    pub fn new(symbol_name: String) -> Self {
        ValueEnum { symbol_name }
    }

    pub fn symbol_name(&self) -> &str {
        &self.symbol_name
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Nullable(Option<Box<Value>>),
    Boolean(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Vec<u8>),
    // buffer value hash
    String(String),
    StaticArray(Vec<Value>),
    DynamicArray(Vec<Value>),
    Map(ValueMap),
    AssetRef(AssetId),
    Record(ValueRecord),
    Enum(ValueEnum),
    Fixed(Box<[u8]>),
}

impl Hash for Value {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        match self {
            Value::Nullable(x) => x.hash(state),
            Value::Boolean(x) => x.hash(state),
            Value::I32(x) => x.hash(state),
            Value::I64(x) => x.hash(state),
            Value::U32(x) => x.hash(state),
            Value::U64(x) => x.hash(state),
            Value::F32(x) => x.to_bits().hash(state),
            Value::F64(x) => x.to_bits().hash(state),
            Value::Bytes(x) => x.hash(state),
            Value::String(x) => x.hash(state),
            Value::StaticArray(x) => x.hash(state),
            Value::DynamicArray(x) => x.hash(state),
            Value::Map(x) => x.hash(state),
            Value::AssetRef(x) => x.hash(state),
            Value::Record(x) => x.hash(state),
            Value::Enum(x) => x.hash(state),
            Value::Fixed(x) => x.hash(state),
        }
    }
}

const DEFAULT_VALUE_NULLABLE: Value = Value::Nullable(None);
const DEFAULT_VALUE_BOOLEAN: Value = Value::Boolean(false);
const DEFAULT_VALUE_I32: Value = Value::I32(0);
const DEFAULT_VALUE_I64: Value = Value::I64(0);
const DEFAULT_VALUE_U32: Value = Value::U32(0);
const DEFAULT_VALUE_U64: Value = Value::U64(0);
const DEFAULT_VALUE_F32: Value = Value::F32(0.0);
const DEFAULT_VALUE_F64: Value = Value::F64(0.0);
const DEFAULT_VALUE_ASSET_REF: Value = Value::AssetRef(AssetId::null());

lazy_static::lazy_static! {
    static ref DEFAULT_VALUE_BYTES: Value = Value::Bytes(Default::default());
    static ref DEFAULT_VALUE_STRING: Value = Value::String(Default::default());
    static ref DEFAULT_VALUE_STATIC_ARRAY: Value = Value::StaticArray(Default::default());
    static ref DEFAULT_VALUE_DYNAMIC_ARRAY: Value = Value::DynamicArray(Default::default());
    static ref DEFAULT_VALUE_MAP: Value = Value::Map(ValueMap::default());
    static ref DEFAULT_VALUE_RECORD: Value = Value::Record(ValueRecord::default());
    static ref DEFAULT_VALUE_ENUM: Value = Value::Enum(ValueEnum::default());
    static ref DEFAULT_VALUE_FIXED: Value = Value::Fixed(Box::new([]));
}

impl Value {
    pub fn default_for_schema<'a>(
        schema: &Schema,
        schema_set: &'a SchemaSet,
    ) -> &'a Self {
        match schema {
            Schema::Nullable(_) => &DEFAULT_VALUE_NULLABLE,
            Schema::Boolean => &DEFAULT_VALUE_BOOLEAN,
            Schema::I32 => &DEFAULT_VALUE_I32,
            Schema::I64 => &DEFAULT_VALUE_I64,
            Schema::U32 => &DEFAULT_VALUE_U32,
            Schema::U64 => &DEFAULT_VALUE_U64,
            Schema::F32 => &DEFAULT_VALUE_F32,
            Schema::F64 => &DEFAULT_VALUE_F64,
            Schema::Bytes => &DEFAULT_VALUE_BYTES,
            Schema::String => &DEFAULT_VALUE_STRING,
            Schema::StaticArray(_) => &DEFAULT_VALUE_STATIC_ARRAY,
            Schema::DynamicArray(_) => &DEFAULT_VALUE_DYNAMIC_ARRAY,
            Schema::Map(_) => &DEFAULT_VALUE_MAP,
            Schema::AssetRef(_) => &DEFAULT_VALUE_ASSET_REF,
            Schema::NamedType(named_type_id) => {
                let named_type = schema_set.schemas().get(named_type_id).unwrap();
                match named_type {
                    SchemaNamedType::Record(_) => &DEFAULT_VALUE_RECORD,
                    SchemaNamedType::Enum(enum_schema) => schema_set
                        .default_value_for_enum(enum_schema.fingerprint())
                        .unwrap(),
                    SchemaNamedType::Fixed(_) => &DEFAULT_VALUE_FIXED,
                }
            }
        }
    }

    pub fn matches_schema(
        &self,
        schema: &Schema,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> bool {
        match self {
            Value::Nullable(inner_value) => {
                match schema {
                    Schema::Nullable(inner_schema) => {
                        if let Some(inner_value) = inner_value {
                            // check inner value is the intended schema
                            inner_value.matches_schema(inner_schema, named_types)
                        } else {
                            // value is null, that's allowed
                            true
                        }
                    }
                    _ => false,
                }
            }
            Value::Boolean(_) => schema.is_boolean(),
            Value::I32(_) => schema.is_i32(),
            Value::I64(_) => schema.is_i64(),
            Value::U32(_) => schema.is_u32(),
            Value::U64(_) => schema.is_u64(),
            Value::F32(_) => schema.is_f32(),
            Value::F64(_) => schema.is_f64(),
            Value::Bytes(_) => schema.is_bytes(),
            Value::String(_) => schema.is_string(),
            Value::StaticArray(inner_values) => match schema {
                Schema::StaticArray(inner_schema) => {
                    if inner_schema.length() != inner_values.len() {
                        return false;
                    }

                    for value in inner_values {
                        if !value.matches_schema(&*inner_schema.item_type(), named_types) {
                            return false;
                        }
                    }

                    true
                }
                _ => false,
            },
            Value::DynamicArray(inner_values) => match schema {
                Schema::DynamicArray(inner_schema) => {
                    for inner_value in inner_values {
                        if !inner_value.matches_schema(inner_schema.item_type(), named_types) {
                            return false;
                        }
                    }

                    true
                }
                _ => false,
            },
            Value::Map(inner_value) => match schema {
                Schema::Map(inner_schema) => {
                    for (k, v) in &inner_value.properties {
                        if !k.matches_schema(inner_schema.key_type(), named_types) {
                            return false;
                        }

                        if !v.matches_schema(inner_schema.value_type(), named_types) {
                            return false;
                        }
                    }

                    true
                }
                _ => false,
            },
            Value::AssetRef(_) => {
                //TODO: Validate type
                schema.is_asset_ref()
            }
            Value::Record(inner_value) => {
                // All value properties must exist and match in the schema. However we allow the
                // value to be missing properties in the schema
                match schema {
                    Schema::NamedType(named_type_id) => {
                        let named_type = named_types.get(named_type_id).unwrap();
                        match named_type {
                            SchemaNamedType::Record(inner_schema) => {
                                // Walk through all properties and make sure the field exists and type matches
                                for (k, v) in &inner_value.properties {
                                    let mut property_match_found = false;
                                    for field in inner_schema.fields() {
                                        if field.name() == k {
                                            if v.matches_schema(field.field_schema(), named_types) {
                                                property_match_found = true;
                                                break;
                                            } else {
                                                return false;
                                            }
                                        }
                                    }

                                    if !property_match_found {
                                        return false;
                                    }
                                }

                                true
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
            }
            Value::Enum(inner_value) => match schema {
                Schema::NamedType(named_type_id) => {
                    let named_type = named_types.get(named_type_id).unwrap();
                    match named_type {
                        SchemaNamedType::Enum(inner_schema) => {
                            for option in inner_schema.symbols() {
                                if option.name() == inner_value.symbol_name {
                                    return true;
                                }
                            }

                            false
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
            Value::Fixed(value) => match schema {
                Schema::NamedType(named_type_id) => {
                    let named_type = named_types.get(named_type_id).unwrap();
                    match named_type {
                        SchemaNamedType::Fixed(inner_schema) => {
                            value.len() == inner_schema.length()
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
        }
    }

    //
    // Nullable
    //
    pub fn is_nullable(&self) -> bool {
        match self {
            Value::Nullable(_) => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Nullable(None) => true,
            _ => false,
        }
    }

    pub fn as_nullable(&self) -> Option<Option<&Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_ref().map(|x| x.as_ref())),
            _ => None,
        }
    }

    pub fn as_nullable_mut(&mut self) -> Option<Option<&mut Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_mut().map(|x| x.as_mut())),
            _ => None,
        }
    }

    //
    // Boolean
    //
    pub fn is_boolean(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            _ => false,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(x) => Some(*x),
            _ => None,
        }
    }

    pub fn set_boolean(
        &mut self,
        value: bool,
    ) {
        *self = Value::Boolean(value);
    }

    //
    // i32
    //
    pub fn is_i32(&self) -> bool {
        match self {
            Value::I32(_) => true,
            _ => false,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::I32(x) => Some(*x as i32),
            Value::U32(x) => Some(*x as i32),
            Value::I64(x) => Some(*x as i32),
            Value::U64(x) => Some(*x as i32),
            Value::F32(x) => Some(*x as i32),
            Value::F64(x) => Some(*x as i32),
            _ => None,
        }
    }

    pub fn set_i32(
        &mut self,
        value: i32,
    ) {
        *self = Value::I32(value);
    }

    // fn get_i32(&self) -> Option<i32> {
    //     match self {
    //         Value::I32(x) => Some(*x),
    //         _ => None
    //     }
    // }
    //
    // fn get_i32_mut(&mut self) -> Option<&mut i32> {
    //     match self {
    //         Value::I32(x) => Some(&mut *x),
    //         _ => None
    //     }
    // }

    //
    // u32
    //
    pub fn is_u32(&self) -> bool {
        match self {
            Value::U32(_) => true,
            _ => false,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::I32(x) => Some(*x as u32),
            Value::U32(x) => Some(*x as u32),
            Value::I64(x) => Some(*x as u32),
            Value::U64(x) => Some(*x as u32),
            Value::F32(x) => Some(*x as u32),
            Value::F64(x) => Some(*x as u32),
            _ => None,
        }
    }

    pub fn set_u32(
        &mut self,
        value: u32,
    ) {
        *self = Value::U32(value);
    }

    //
    // i64
    //
    pub fn is_i64(&self) -> bool {
        match self {
            Value::I64(_) => true,
            _ => false,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I32(x) => Some(*x as i64),
            Value::U32(x) => Some(*x as i64),
            Value::I64(x) => Some(*x as i64),
            Value::U64(x) => Some(*x as i64),
            Value::F32(x) => Some(*x as i64),
            Value::F64(x) => Some(*x as i64),
            _ => None,
        }
    }

    pub fn set_i64(
        &mut self,
        value: i64,
    ) {
        *self = Value::I64(value);
    }

    //
    // u64
    //
    pub fn is_u64(&self) -> bool {
        match self {
            Value::U64(_) => true,
            _ => false,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::I32(x) => Some(*x as u64),
            Value::U32(x) => Some(*x as u64),
            Value::I64(x) => Some(*x as u64),
            Value::U64(x) => Some(*x as u64),
            Value::F32(x) => Some(*x as u64),
            Value::F64(x) => Some(*x as u64),
            _ => None,
        }
    }

    pub fn set_u64(
        &mut self,
        value: u64,
    ) {
        *self = Value::U64(value);
    }

    //
    // f32
    //
    pub fn is_f32(&self) -> bool {
        match self {
            Value::F32(_) => true,
            _ => false,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::I32(x) => Some(*x as f32),
            Value::U32(x) => Some(*x as f32),
            Value::I64(x) => Some(*x as f32),
            Value::U64(x) => Some(*x as f32),
            Value::F32(x) => Some(*x),
            Value::F64(x) => Some(*x as f32),
            _ => None,
        }
    }

    pub fn set_f32(
        &mut self,
        value: f32,
    ) {
        *self = Value::F32(value);
    }

    //
    // f64
    //
    pub fn is_f64(&self) -> bool {
        match self {
            Value::F64(_) => true,
            _ => false,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::I32(x) => Some(*x as f64),
            Value::U32(x) => Some(*x as f64),
            Value::I64(x) => Some(*x as f64),
            Value::U64(x) => Some(*x as f64),
            Value::F32(x) => Some(*x as f64),
            Value::F64(x) => Some(*x),
            _ => None,
        }
    }

    pub fn set_f64(
        &mut self,
        value: f64,
    ) {
        *self = Value::F64(value);
    }

    //
    // Bytes
    //
    pub fn is_bytes(&self) -> bool {
        match self {
            Value::Bytes(_) => true,
            _ => false,
        }
    }

    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            Value::Bytes(x) => Some(x),
            _ => None,
        }
    }
    pub fn set_bytes(
        &mut self,
        value: Vec<u8>,
    ) {
        *self = Value::Bytes(value);
    }

    //
    // Buffer
    //

    //
    // String
    //
    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(x) => Some(&*x),
            _ => None,
        }
    }

    pub fn set_string(
        &mut self,
        value: String,
    ) {
        *self = Value::String(value);
    }

    //
    // StaticArray
    //

    //
    // DynamicArray
    //

    //
    // Map
    //

    //
    // AssetRef
    //
    pub fn is_asset_ref(&self) -> bool {
        match self {
            Value::AssetRef(_) => true,
            _ => false,
        }
    }

    pub fn as_asset_ref(&self) -> Option<AssetId> {
        match self {
            Value::AssetRef(x) => Some(*x),
            _ => None,
        }
    }

    pub fn set_asset_ref(
        &mut self,
        value: AssetId,
    ) {
        *self = Value::AssetRef(value);
    }

    //
    // Record
    //
    pub fn is_record(&self) -> bool {
        match self {
            Value::Record(_) => true,
            _ => false,
        }
    }

    pub fn as_record(&self) -> Option<&ValueRecord> {
        match self {
            Value::Record(x) => Some(&*x),
            _ => None,
        }
    }

    pub fn set_record(
        &mut self,
        value: ValueRecord,
    ) {
        *self = Value::Record(value);
    }

    //
    // Enum
    //
    pub fn is_enum(&self) -> bool {
        match self {
            Value::Enum(_) => true,
            _ => false,
        }
    }

    pub fn as_enum(&self) -> Option<&ValueEnum> {
        match self {
            Value::Enum(x) => Some(&*x),
            _ => None,
        }
    }

    pub fn set_enum(
        &mut self,
        value: ValueEnum,
    ) {
        *self = Value::Enum(value);
    }

    pub fn enum_value_from_string(
        schema_enum: &SchemaEnum,
        name: &str,
    ) -> Option<Value> {
        for symbol in &*schema_enum.symbols() {
            if symbol.name() == name {
                return Some(Value::Enum(ValueEnum::new(name.to_string())));
            }

            for alias in symbol.aliases() {
                if alias == name {
                    return Some(Value::Enum(ValueEnum::new(name.to_string())));
                }
            }
        }

        None
    }

    //
    // Fixed
    //

    pub fn as_property_value(&self) -> Option<PropertyValue> {
        match self {
            Value::Boolean(x) => Some(PropertyValue::Boolean(*x)),
            Value::I32(x) => Some(PropertyValue::I32(*x)),
            Value::I64(x) => Some(PropertyValue::I64(*x)),
            Value::U32(x) => Some(PropertyValue::U32(*x)),
            Value::U64(x) => Some(PropertyValue::U64(*x)),
            Value::F32(x) => Some(PropertyValue::F32(*x)),
            Value::F64(x) => Some(PropertyValue::F64(*x)),
            Value::Bytes(x) => Some(PropertyValue::Bytes(x.clone())),
            Value::String(x) => Some(PropertyValue::String(x.clone())),
            Value::AssetRef(x) => Some(PropertyValue::AssetRef(*x)),
            Value::Enum(x) => Some(PropertyValue::Enum(x.clone())),
            Value::Fixed(x) => Some(PropertyValue::Fixed(x.clone())),
            _ => None,
        }
    }

    pub fn are_matching_property_values(
        lhs: &Value,
        rhs: &Value,
    ) -> bool {
        match (lhs, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => *lhs == *rhs,
            (Value::I32(lhs), Value::I32(rhs)) => *lhs == *rhs,
            (Value::I64(lhs), Value::I64(rhs)) => *lhs == *rhs,
            (Value::U32(lhs), Value::U32(rhs)) => *lhs == *rhs,
            (Value::U64(lhs), Value::U64(rhs)) => *lhs == *rhs,
            (Value::F32(lhs), Value::F32(rhs)) => *lhs == *rhs,
            (Value::F64(lhs), Value::F64(rhs)) => *lhs == *rhs,
            (Value::Bytes(lhs), Value::Bytes(rhs)) => *lhs == *rhs,
            (Value::String(lhs), Value::String(rhs)) => *lhs == *rhs,
            (Value::AssetRef(lhs), Value::AssetRef(rhs)) => *lhs == *rhs,
            (Value::Enum(lhs), Value::Enum(rhs)) => *lhs == *rhs,
            (Value::Fixed(lhs), Value::Fixed(rhs)) => *lhs == *rhs,
            _ => false,
        }
    }
}

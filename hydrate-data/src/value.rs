use crate::AssetId;
use crate::{HashMap, Schema, SchemaFingerprint, SchemaNamedType, SchemaSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use hydrate_schema::{DataSetError, DataSetResult, SchemaEnum};

/// All the possible value types that can exist that do not potentially contain values within them.
/// So excludes containers, nullable, records, etc.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Boolean(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Arc<Vec<u8>>),
    String(Arc<String>),
    AssetRef(AssetId),
    Enum(ValueEnum),
}

impl PropertyValue {
    /// Convert to a value
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
        }
    }

    /// Validates if the values could be property values, and if they can if they would be matching
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
            _ => false,
        }
    }
}

/// Hashmap-like container
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

/// A struct-like container of properties
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

/// A value for an enum. Strings are used instead of numbers so that we can handle loading
/// "broken" data.
#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct ValueEnum {
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

/// All the possible types that can be stored in a Value
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
    Bytes(Arc<Vec<u8>>),
    // buffer value hash
    String(Arc<String>),
    StaticArray(Vec<Value>),
    DynamicArray(Vec<Value>),
    Map(ValueMap),
    AssetRef(AssetId),
    Record(ValueRecord),
    Enum(ValueEnum),
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
    static ref DEFAULT_VALUE_BYTES: Value = Value::Bytes(Arc::new(Vec::default()));
    static ref DEFAULT_VALUE_STRING: Value = Value::String(Arc::from(String::default()));
    static ref DEFAULT_VALUE_STATIC_ARRAY: Value = Value::StaticArray(Default::default());
    static ref DEFAULT_VALUE_DYNAMIC_ARRAY: Value = Value::DynamicArray(Default::default());
    static ref DEFAULT_VALUE_MAP: Value = Value::Map(ValueMap::default());
    static ref DEFAULT_VALUE_RECORD: Value = Value::Record(ValueRecord::default());
    static ref DEFAULT_VALUE_ENUM: Value = Value::Enum(ValueEnum::default());
}

impl Value {
    /// Produces a default value for the given schema. Because schemas may reference other schemas,
    /// and a default value may have containers in it, we need to have access to all schemas that
    /// may exist.
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
            Schema::Record(_) => &DEFAULT_VALUE_RECORD,
            Schema::Enum(named_type_id) => {
                schema_set.default_value_for_enum(*named_type_id).unwrap()
            }
        }
    }

    /// Validates that the value matches the provided schema exactly. Even if this returns false,
    /// it may still be possible to migrate the data into the given schema. This will recursively
    /// descend through containers, records, etc.
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
                    // We can be lazy about having the correct number of values in the Vec, which allows for an empty
                    // static array to be represented by an empty vec
                    // if inner_schema.length() != inner_values.len() {
                    //     return false;
                    // }

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
                    Schema::Record(named_type_id) => {
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
                            _ => panic!("A Schema::Record fingerprint is matching a named type that isn't a record"),
                        }
                    }
                    _ => false,
                }
            }
            Value::Enum(inner_value) => {
                match schema {
                    Schema::Enum(named_type_id) => {
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
                        _ => panic!("A Schema::Enum fingerprint is matching a named type that isn't a enum"),
                    }
                    }
                    _ => false,
                }
            }
        }
    }

    /// Returns the value as a property value, if possible. Some types cannot be stored as
    /// PropertyValue
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
            _ => None,
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

    pub fn as_nullable(&self) -> DataSetResult<Option<&Value>> {
        self.try_as_nullable().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_nullable(&self) -> Option<Option<&Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_ref().map(|x| x.as_ref())),
            _ => None,
        }
    }

    pub fn set_nullable(
        &mut self,
        value: Option<Value>,
    ) {
        *self = Value::Nullable(value.map(|x| Box::new(x)))
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

    pub fn as_boolean(&self) -> DataSetResult<bool> {
        self.try_as_boolean().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_boolean(&self) -> Option<bool> {
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

    pub fn as_i32(&self) -> DataSetResult<i32> {
        self.try_as_i32().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_i32(&self) -> Option<i32> {
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

    //
    // u32
    //
    pub fn is_u32(&self) -> bool {
        match self {
            Value::U32(_) => true,
            _ => false,
        }
    }

    pub fn as_u32(&self) -> DataSetResult<u32> {
        self.try_as_u32().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_u32(&self) -> Option<u32> {
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

    pub fn as_i64(&self) -> DataSetResult<i64> {
        self.try_as_i64().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_i64(&self) -> Option<i64> {
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

    pub fn as_u64(&self) -> DataSetResult<u64> {
        self.try_as_u64().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_u64(&self) -> Option<u64> {
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

    pub fn as_f32(&self) -> DataSetResult<f32> {
        self.try_as_f32().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_f32(&self) -> Option<f32> {
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

    pub fn as_f64(&self) -> DataSetResult<f64> {
        self.try_as_f64().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_f64(&self) -> Option<f64> {
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

    pub fn as_bytes(&self) -> DataSetResult<&Arc<Vec<u8>>> {
        self.try_as_bytes().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_bytes(&self) -> Option<&Arc<Vec<u8>>> {
        match self {
            Value::Bytes(x) => Some(x),
            _ => None,
        }
    }
    pub fn set_bytes(
        &mut self,
        value: Arc<Vec<u8>>,
    ) {
        *self = Value::Bytes(value);
    }

    //
    // String
    //
    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn as_string(&self) -> DataSetResult<&Arc<String>> {
        self.try_as_string().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_string(&self) -> Option<&Arc<String>> {
        match self {
            Value::String(x) => Some(x),
            _ => None,
        }
    }

    pub fn set_string(
        &mut self,
        value: Arc<String>,
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

    pub fn as_asset_ref(&self) -> DataSetResult<AssetId> {
        self.try_as_asset_ref().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_asset_ref(&self) -> Option<AssetId> {
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

    pub fn as_record(&self) -> DataSetResult<&ValueRecord> {
        self.try_as_record().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_record(&self) -> Option<&ValueRecord> {
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

    pub fn as_enum(&self) -> DataSetResult<&ValueEnum> {
        self.try_as_enum().ok_or(DataSetError::InvalidSchema)
    }

    pub fn try_as_enum(&self) -> Option<&ValueEnum> {
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

    /// Utility function to convert a string to an enum value. This handles potentially matching
    /// a symbol alias and using the new symbol name instead. We generally expect an enum values
    /// in memory to use the current symbol name, not an alias
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
}

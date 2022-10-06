use crate::{BufferId, HashMap, Schema, SchemaFingerprint, SchemaId, SchemaNamedType};
use crate::ObjectId;
use crate::Value::RecordRef;

#[derive(Clone, Debug)]
pub struct ValueMap {
    properties: HashMap<Value, Value>
}

#[derive(Clone, Debug, Default)]
pub struct ValueRecord {
    properties: HashMap<String, Value>
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
#[derive(Clone, Debug)]
pub struct ValueEnum {
    //symbol_index: u32,
    symbol_name: String,
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
    Buffer(BufferId),
    // buffer value hash
    String(String),
    StaticArray(Vec<Value>),
    DynamicArray(Vec<Value>),
    Map(ValueMap),
    RecordRef(ObjectId),
    Record(ValueRecord),
    Enum(ValueEnum),
    Fixed(Box<[u8]>),
}

impl Value {
    pub fn default_for_schema(schema: &Schema, named_types: &HashMap<SchemaFingerprint, SchemaNamedType>) -> Self {
        match schema {
            Schema::Nullable(inner) => Value::Nullable(Default::default()),
            Schema::Boolean => Value::Boolean(Default::default()),
            Schema::I32 => Value::I32(Default::default()),
            Schema::I64 => Value::I64(Default::default()),
            Schema::U32 => Value::U32(Default::default()),
            Schema::U64 => Value::U64(Default::default()),
            Schema::F32 => Value::F32(Default::default()),
            Schema::F64 => Value::F64(Default::default()),
            Schema::Bytes => Value::Bytes(Default::default()),
            Schema::Buffer => Value::Buffer(BufferId::null()),
            Schema::String => Value::String(Default::default()),
            Schema::StaticArray(inner) => Value::StaticArray(vec![Value::default_for_schema(&inner.item_type, named_types); inner.length]),
            Schema::DynamicArray(inner) => Value::DynamicArray(vec![]),
            Schema::Map(inner) => Value::Map(ValueMap {
                properties: Default::default()
            }),
            Schema::RecordRef(inner) => Value::RecordRef(ObjectId::null()),
            Schema::NamedType(named_type_id) => {
                let named_type = named_types.get(named_type_id).unwrap();
                match named_type {
                    SchemaNamedType::Record(inner) => Value::Record(ValueRecord {
                        properties: Default::default()
                    }),
                    SchemaNamedType::Enum(inner) => Value::Enum(ValueEnum {
                        symbol_name: inner.symbols()[0].name().to_string()
                    }),
                    SchemaNamedType::Fixed(inner) => Value::Fixed(vec![0u8; inner.length()].into_boxed_slice()),
                }
            }
            // Schema::Record(inner) => Value::Record(ValueRecord {
            //     properties: Default::default()
            // }),
            // Schema::Enum(inner) => Value::Enum(ValueEnum {
            //     symbol_name: inner.symbols()[0].name().to_string()
            // }),
            // Schema::Fixed(inner) => Value::Fixed(vec![0u8; inner.length()].into_boxed_slice()),
        }
    }

    pub fn matches_schema(&self, schema: &Schema, named_types: &HashMap<SchemaFingerprint, SchemaNamedType>) -> bool {
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
                    },
                    _ => false
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
            Value::Buffer(_) => schema.is_buffer(),
            Value::String(_) => schema.is_string(),
            Value::StaticArray(inner_values) => {
                match schema {
                    Schema::StaticArray(inner_schema) => {
                        if inner_schema.length != inner_values.len() {
                            return false
                        }

                        for value in inner_values {
                            if !value.matches_schema(&*inner_schema.item_type, named_types) {
                                return false;
                            }
                        }

                        true
                    },
                    _ => false
                }
            }
            Value::DynamicArray(inner_values) => {
                match schema {
                    Schema::DynamicArray(inner_schema) => {
                        for inner_value in inner_values {
                            if !inner_value.matches_schema(inner_schema.item_type(), named_types) {
                                return false
                            }
                        }

                        true
                    },
                    _ => false
                }
            }
            Value::Map(inner_value) => {
                match schema {
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
                    },
                    _ => false
                }
            }
            Value::RecordRef(_) => unimplemented!(),
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
                            _ => false
                        }
                    },
                    _ => false
                }
            },
            Value::Enum(inner_value) => {
                match schema {
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
                            },
                            _ => false
                        }
                    },
                    _ => false
                }
            }
            Value::Fixed(value) => {
                match schema {
                    Schema::NamedType(named_type_id) => {

                        let named_type = named_types.get(named_type_id).unwrap();
                        match named_type {
                            SchemaNamedType::Fixed(inner_schema) => {
                                value.len() == inner_schema.length()
                            },
                            _ => false
                        }
                    },
                    _ => false
                }
            }
        }
    }

    //
    // Nullable
    //
    pub fn is_nullable(&self) -> bool {
        match self {
            Value::Nullable(_) => true,
            _ => false
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Nullable(None) => true,
            _ => false
        }
    }

    pub fn as_nullable(&self) -> Option<Option<&Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_ref().map(|x| x.as_ref())),
            _ => None
        }
    }

    pub fn as_nullable_mut(&mut self) -> Option<Option<&mut Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_mut().map(|x| x.as_mut())),
            _ => None
        }
    }

    //
    // Boolean
    //
    pub fn is_boolean(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            _ => false
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(x) => Some(*x),
            _ => None
        }
    }

    pub fn set_boolean(&mut self, value: bool) {
        *self = Value::Boolean(value);
    }

    //
    // i32
    //
    pub fn is_i32(&self) -> bool {
        match self {
            Value::I32(_) => true,
            _ => false
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
            _ => None
        }
    }

    pub fn set_i32(&mut self, value: i32) {
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
            _ => false
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
            _ => None
        }
    }

    pub fn set_u32(&mut self, value: u32) {
        *self = Value::U32(value);
    }

    //
    // i64
    //
    pub fn is_i64(&self) -> bool {
        match self {
            Value::I64(_) => true,
            _ => false
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
            _ => None
        }
    }

    pub fn set_i64(&mut self, value: i64) {
        *self = Value::I64(value);
    }

    //
    // u64
    //
    pub fn is_u64(&self) -> bool {
        match self {
            Value::U64(_) => true,
            _ => false
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
            _ => None
        }
    }

    pub fn set_u64(&mut self, value: u64) {
        *self = Value::U64(value);
    }

    //
    // f32
    //
    pub fn is_f32(&self) -> bool {
        match self {
            Value::F32(_) => true,
            _ => false
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
            _ => None
        }
    }

    pub fn set_f32(&mut self, value: f32) {
        *self = Value::F32(value);
    }

    //
    // f64
    //
    pub fn is_f64(&self) -> bool {
        match self {
            Value::F64(_) => true,
            _ => false
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
            _ => None
        }
    }

    pub fn set_f64(&mut self, value: f64) {
        *self = Value::F64(value);
    }

    //
    // Bytes
    //

    //
    // Buffer
    //

    //
    // String
    //
    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(x) => Some(&*x),
            _ => None
        }
    }

    pub fn set_string(&mut self, value: String) {
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
    // RecordRef
    //

    //
    // Record
    //
    pub fn is_record(&self) -> bool {
        match self {
            Value::Record(_) => true,
            _ => false
        }
    }

    pub fn as_record(&self) -> Option<&ValueRecord> {
        match self {
            Value::Record(x) => Some(&*x),
            _ => None
        }
    }

    pub fn set_record(&mut self, value: ValueRecord) {
        *self = Value::Record(value);
    }

    //
    // Enum
    //

    //
    // Fixed
    //
}

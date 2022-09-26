use crate::HashMap;
use crate::ObjectId;

pub struct ValueMap {
    properties: HashMap<Value, Value>
}

pub struct ValueRecord {
    properties: HashMap<String, Value>
}

pub struct ValueEnum {
    symbol_index: u32,
    symbol_name: String,
}

pub enum Value {
    Nullable(Option<Box<Value>>),
    Boolean(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Bytes(Vec<u8>),
    Buffer(u128),
    // buffer value hash
    String(String),
    StaticArray(Vec<Value>),
    DynamicArray(Vec<Value>),
    Map(ValueMap),
    ObjectRef(ObjectId),
    Struct(ValueRecord),
    Enum(ValueEnum),
    Fixed(Box<[u8]>),
}

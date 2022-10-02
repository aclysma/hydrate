use crate::{BufferId, HashMap, Schema};
use crate::ObjectId;

#[derive(Clone, Debug)]
pub struct ValueMap {
    properties: HashMap<Value, Value>
}

#[derive(Clone, Debug, Default)]
pub struct ValueRecord {
    properties: HashMap<String, Value>
}

impl ValueRecord {
    // fn get_property(&self, property_name: impl AsRef<str>) -> Option<&Value> {
    //     self.properties.get(property_name.as_ref())
    // }

    pub fn find_property_value(&self, property_name: impl AsRef<str>) -> Option<&Value> {
        self.properties.get(property_name.as_ref())
    }
}

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
    pub(crate) fn matches_schema(&self, schema: &Schema) -> bool {
        match self {
            Value::Nullable(inner_value) => {
                match schema {
                    Schema::Nullable(inner_schema) => {
                        if let Some(inner_value) = inner_value {
                            // check inner value is the intended schema
                            inner_value.matches_schema(inner_schema)
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
                            if !value.matches_schema(&*inner_schema.item_type) {
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
                            if !inner_value.matches_schema(inner_schema.item_type()) {
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
                            if !k.matches_schema(inner_schema.key_type()) {
                                return false;
                            }

                            if !v.matches_schema(inner_schema.value_type()) {
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
                    Schema::Record(inner_schema) => {
                        // Walk through all properties and make sure the field exists and type matches
                        for (k, v) in &inner_value.properties {
                            let mut property_match_found = false;
                            for field in inner_schema.fields() {
                                if field.name() == k {
                                    if v.matches_schema(field.field_schema()) {
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
                    },
                    _ => false
                }
            },
            Value::Enum(inner_value) => {
                match schema {
                    Schema::Enum(inner_schema) => {
                        for option in inner_schema.symbols() {
                            if option.name() == inner_value.symbol_name {
                                return true;
                            }
                        }

                        false
                    },
                    _ => false
                }
            }
            Value::Fixed(value) => {
                match schema {
                    Schema::Fixed(inner_schema) => {
                        value.len() == inner_schema.length()
                    },
                    _ => false
                }
            }
        }
    }

    //
    // Nullable
    //
    fn is_nullable(&self) -> bool {
        match self {
            Value::Nullable(_) => true,
            _ => false
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Value::Nullable(None) => true,
            _ => false
        }
    }

    fn as_nullable(&self) -> Option<Option<&Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_ref().map(|x| x.as_ref())),
            _ => None
        }
    }

    fn as_nullable_mut(&mut self) -> Option<Option<&mut Value>> {
        match self {
            Value::Nullable(x) => Some(x.as_mut().map(|x| x.as_mut())),
            _ => None
        }
    }

    //
    // Boolean
    //
    fn is_boolean(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            _ => false
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(x) => Some(*x),
            _ => None
        }
    }

    fn set_boolean(&mut self, value: bool) {
        *self = Value::Boolean(value);
    }

    //
    // i32
    //
    fn is_i32(&self) -> bool {
        match self {
            Value::I32(_) => true,
            _ => false
        }
    }

    fn as_i32(&self) -> Option<i32> {
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

    fn set_i32(&mut self, value: i32) {
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
    fn is_u32(&self) -> bool {
        match self {
            Value::U32(_) => true,
            _ => false
        }
    }

    fn as_u32(&self) -> Option<u32> {
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

    fn set_u32(&mut self, value: u32) {
        *self = Value::U32(value);
    }

    //
    // i64
    //
    fn is_i64(&self) -> bool {
        match self {
            Value::I64(_) => true,
            _ => false
        }
    }

    fn as_i64(&self) -> Option<i64> {
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

    fn set_i64(&mut self, value: i64) {
        *self = Value::I64(value);
    }

    //
    // u64
    //
    fn is_u64(&self) -> bool {
        match self {
            Value::U64(_) => true,
            _ => false
        }
    }

    fn as_u64(&self) -> Option<u64> {
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

    fn set_u64(&mut self, value: u64) {
        *self = Value::U64(value);
    }

    //
    // f32
    //
    fn is_f32(&self) -> bool {
        match self {
            Value::F32(_) => true,
            _ => false
        }
    }

    fn as_f32(&self) -> Option<f32> {
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

    fn set_f32(&mut self, value: f32) {
        *self = Value::F32(value);
    }

    //
    // f64
    //
    fn is_f64(&self) -> bool {
        match self {
            Value::F64(_) => true,
            _ => false
        }
    }

    fn as_f64(&self) -> Option<f64> {
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

    fn set_f64(&mut self, value: f64) {
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

    pub fn find_property_value(&self, name: impl AsRef<str>) -> Option<&Value> {
        let mut record = None;
        match self {
            Value::Record(x) => {
                record = Some(x);
            },
            _ => {}
        }

        if let Some(record) = record {
            record.properties.get(name.as_ref())
        } else {
            None
        }
    }

    pub fn find_property_value_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Value> {
        //let mut record = None;
        match self {
            // Value::Nullable(x) => {
            //     //record
            //     if name.as_ref() == "is_null" {
            //         Value::Boolean(x.is_none())
            //     } else if name.as_ref() == "value" {
            //         x.map(|x| &mut *x)
            //     } else {
            //         None
            //     }
            // }
            Value::Record(x) => {
                //record = Some(x);
                x.properties.get_mut(name.as_ref())
            },
            _ => None
        }

        // if let Some(record) = record {
        //     record.properties.get_mut(name.as_ref())
        // } else {
        //     None
        // }
    }
/*
    pub fn find_property_path_value<T: AsRef<str>>(&self, path: &[T]) -> Option<&Value> {
        let mut value = self;

        for path_element in path {
            let v = value.find_property_value(path_element);

            if let Some(v) = v {
                value = v;
            } else {
                return None;
            }
        }

        Some(value)
    }

    pub fn find_property_path_value_mut<T: AsRef<str>>(&mut self, path: &[T]) -> Option<&mut Value> {
        let mut value = self;

        for path_element in path {
            let v = value.find_property_value_mut(path_element);

            if let Some(v) = v {
                value = v;
            } else {
                return None;
            }
        }

        Some(value)
    }
*/
    pub fn clear_property_value(&mut self, name: impl AsRef<str>) -> Option<Value> {
        let mut record = None;
        match self {
            Value::Record(x) => {
                record = Some(x);
            },
            _ => {}
        }

        if let Some(record) = record {
            record.properties.remove(name.as_ref())
        } else {
            None
        }
    }
/*
    pub fn clear_property_path_value<T: AsRef<str>>(&mut self, path: &[T]) -> Option<Value> {
        if let Some(last) = path.last() {
            let v = self.find_property_path_value_mut(&path[0..path.len() - 1]);
            if let Some(v) = v {
                v.clear_property_value(last.as_ref())
            } else {
                None
            }
        } else {
            None
        }
    }
*/
    pub fn set_property_value(&mut self, name: impl Into<String>, value: Value) -> bool {
        let mut record = None;
        match self {
            Value::Record(x) => {
                record = Some(x);
            },
            _ => {}
        }

        if let Some(record) = record {
            record.properties.insert(name.into(), value);
            true
        } else {
            false
        }
    }
/*
    pub fn set_property_path_value<T: AsRef<str>>(&mut self, path: &[T], value: Value) -> bool {
        if path.is_empty() {
            *self = value;
            return true;
        }

        if let Some(p) = self.find_property_value_mut(path.first().unwrap()) {
            p.set_property_path_value(&path[1..], value)
        } else {
            let mut p = Value::Record(Default::default());
            p.set_property_path_value(&path[1..], value);
            self.set_property_value(path.first().unwrap().as_ref(), p);
            true
        }
    }
*/
    //
    // Enum
    //

    //
    // Fixed
    //
}


#[cfg(test)]
mod test {
    use crate::{HashMap, Value, ValueRecord};

    #[test]
    fn test_find_property_path() {
        let mut properties_min = HashMap::default();
        properties_min.insert("x".to_string(), Value::I32(1));
        properties_min.insert("y".to_string(), Value::I32(2));
        properties_min.insert("z".to_string(), Value::I32(3));

        let mut properties_max = HashMap::default();
        properties_max.insert("x".to_string(), Value::I32(4));
        properties_max.insert("y".to_string(), Value::I32(5));
        properties_max.insert("z".to_string(), Value::I32(6));

        let mut record_properties = HashMap::default();
        record_properties.insert("min".to_string(), Value::Record(ValueRecord {properties:properties_min }));
        record_properties.insert("max".to_string(), Value::Record(ValueRecord {properties:properties_max }));

        let record = Value::Record(ValueRecord {
            properties: record_properties
        });

        // Access properties
        /*
        assert_eq!(record.find_property_path_value(&["min"]).unwrap().find_property_path_value(&["x"]).unwrap().as_i32().unwrap(), 1);
        assert_eq!(record.find_property_path_value(&["min", "x"]).unwrap().as_i32().unwrap(), 1);
        assert_eq!(record.find_property_path_value(&["min", "y"]).unwrap().as_i32().unwrap(), 2);
        assert_eq!(record.find_property_path_value(&["min", "z"]).unwrap().as_i32().unwrap(), 3);
        assert_eq!(record.find_property_path_value(&["max", "x"]).unwrap().as_i32().unwrap(), 4);
        assert_eq!(record.find_property_path_value(&["max", "y"]).unwrap().as_i32().unwrap(), 5);
        assert_eq!(record.find_property_path_value(&["max", "z"]).unwrap().as_i32().unwrap(), 6);

        // Fail at accessing non-existent properties
        assert!(record.find_property_path_value(&["asdfsadf"]).is_none());
        assert!(record.find_property_path_value(&["max", "asds"]).is_none());
        assert!(record.find_property_path_value(&["max", "x", "aaaaa"]).is_none());

         */

    }

    #[test]
    fn test_get_property_path() {
        let mut properties_min = HashMap::default();
        properties_min.insert("x".to_string(), Value::I32(1));
        properties_min.insert("y".to_string(), Value::I32(2));
        properties_min.insert("z".to_string(), Value::I32(3));

        let mut properties_max = HashMap::default();
        properties_max.insert("x".to_string(), Value::I32(4));
        properties_max.insert("y".to_string(), Value::I32(5));
        properties_max.insert("z".to_string(), Value::I32(6));

        let mut record_properties = HashMap::default();
        record_properties.insert("min".to_string(), Value::Record(ValueRecord {properties:properties_min }));
        record_properties.insert("max".to_string(), Value::Record(ValueRecord {properties:properties_max }));

        let mut record = Value::Record(ValueRecord {
            properties: record_properties
        });/*

        // Set and clear a property
        assert_eq!(record.find_property_path_value(&["min", "x"]).unwrap().as_i32().unwrap(), 1);
        record.set_property_path_value(&["min", "x"], Value::I32(10));
        assert_eq!(record.find_property_path_value(&["min", "x"]).unwrap().as_i32().unwrap(), 10);
        record.clear_property_path_value(&["min", "x"]);
        assert!(record.find_property_path_value(&["min", "x"]).is_none());

        // Set and clear another property
        assert_eq!(record.find_property_path_value(&["max", "y"]).unwrap().as_i32().unwrap(), 5);
        record.set_property_path_value(&["max", "y"], Value::I32(20));
        assert_eq!(record.find_property_path_value(&["max", "y"]).unwrap().as_i32().unwrap(), 20);
        record.clear_property_path_value(&["max", "y"]);
        assert!(record.find_property_path_value(&["max", "y"]).is_none());

        // Setting a property where parent property is not a record should fail, returning false
        assert!(!record.set_property_path_value(&["min", "x", "asdfsaf"], Value::I32(10)));
        */
    }
}
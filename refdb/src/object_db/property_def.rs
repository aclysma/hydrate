pub use super::*;

#[derive(Clone)]
pub struct PropertyDef {
    // name, type, editor, legal subobject types, tooltip, transient, UI name
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: Value
}

impl PropertyDef {
    pub fn new_u64<T: Into<String>>(name: T) -> Self {
        PropertyDef {
            name: name.into(),
            property_type: PropertyType::U64,
            default_value: Value::U64(0)
        }
    }

    pub fn new_f32<T: Into<String>>(name: T) -> Self {
        PropertyDef {
            name: name.into(),
            property_type: PropertyType::F32,
            default_value: Value::F32(0.0)
        }
    }

    // pub fn new_subobject<T: Into<String>>(name: T, object_type: ObjectTypeId) -> Self {
    //     PropertyDef {
    //         name: name.into(),
    //         property_type: PropertyType::Subobject(object_type),
    //         default_value: Value::Subobject(ObjectKey::null())
    //     }
    // }
}

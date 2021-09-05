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
        Self::new_u64_with_default(name, 0)
    }

    pub fn new_u64_with_default<T: Into<String>>(name: T, default_value: u64) -> Self {
        PropertyDef {
            name: name.into(),
            property_type: PropertyType::U64,
            default_value: Value::U64(default_value)
        }
    }

    pub fn new_f32<T: Into<String>>(name: T) -> Self {
        Self::new_f32_with_default(name, 0.0)
    }

    pub fn new_f32_with_default<T: Into<String>>(name: T, default_value: f32) -> Self {
        PropertyDef {
            name: name.into(),
            property_type: PropertyType::F32,
            default_value: Value::F32(default_value)
        }
    }

    // If ty is an object type, default value of null will create a default object of the given type
    // If ty is an interface type, default value of null is not allowed
    pub fn new_subobject<T: Into<String>, S: Into<TypeSelector>>(name: T, ty: S) -> Self {
        Self::new_subobject_with_default(name, ty, ObjectId(ObjectKey::null()))
    }

    pub fn new_subobject_with_default<T: Into<String>, S: Into<TypeSelector>>(name: T, ty: S, default_value: ObjectId) -> Self {
        PropertyDef {
            name: name.into(),
            property_type: PropertyType::Subobject(ty.into()),
            default_value: Value::Subobject(ObjectKey::null())
        }
    }
}

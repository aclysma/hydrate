use ahash::{AHashMap, AHashSet};
use uuid::Uuid;
use std::collections::VecDeque;
use slotmap::SlotMap;
use slotmap::Key;

#[cfg(test)]
mod tests;

mod bits;
pub use bits::*;

mod error;
pub use error::*;

mod value;
pub use value::*;

mod db;
pub use db::*;

mod property_def;
pub use property_def::*;
use std::any::Any;

//
// up to 64 properties in a single object type
//
const MAX_PROPERTY_COUNT : usize = 64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PropertyIndex(u8);

impl PropertyIndex {
    pub fn from_index(index: usize) -> PropertyIndex {
        PropertyIndex(index as u8)
    }
}

//
// Up to 65k object types
//
const MAX_OBJECT_TYPE_COUNT : usize = u16::MAX as usize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectTypeId(u16);

//
// Up to 64 interface types
//
const MAX_INTERFACE_TYPE_COUNT : usize = 64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceTypeId(u8);

//
// Up to 4B objects, uses a u32 ID and u32 version
//
slotmap::new_key_type! { pub struct ObjectKey; }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId(ObjectKey);

impl ObjectId {
    pub fn null() -> ObjectId {
        ObjectId(ObjectKey::null())
    }
}

// // Magic ID to represent any type
// const ANY_INTERFACE_TYPE_ID : InterfaceTypeId = InterfaceTypeId(u8::MAX);

type PropertyBits = BitsU64;
type InterfaceBits = BitsU64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeSelector {
    Any,
    Interface(InterfaceTypeId),
    Object(ObjectTypeId),
}

impl TypeSelector {
    pub fn is_concrete(self) -> bool {
        match self {
            TypeSelector::Any => false,
            TypeSelector::Interface(_) => false,
            TypeSelector::Object(_) => true,
        }
    }
}

impl From<TypeId> for TypeSelector {
    fn from(ty: TypeId) -> Self {
        match ty {
            TypeId::Interface(v) => TypeSelector::Interface(v),
            TypeId::Object(v) => TypeSelector::Object(v)
        }
    }
}

impl From<InterfaceTypeId> for TypeSelector {
    fn from(ty: InterfaceTypeId) -> Self {
        TypeSelector::Interface(ty)
    }
}

impl From<ObjectTypeId> for TypeSelector {
    fn from(ty: ObjectTypeId) -> Self {
        TypeSelector::Object(ty)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeId {
    Interface(InterfaceTypeId),
    Object(ObjectTypeId)
}

impl From<InterfaceTypeId> for TypeId {
    fn from(ty: InterfaceTypeId) -> Self {
        TypeId::Interface(ty)
    }
}

impl From<ObjectTypeId> for TypeId {
    fn from(ty: ObjectTypeId) -> Self {
        TypeId::Object(ty)
    }
}

impl TypeId {
    pub fn object_type(self) -> Option<ObjectTypeId> {
        match self {
            TypeId::Interface(_) => None,
            TypeId::Object(v) => Some(v)
        }
    }

    pub fn interface_type(self) -> Option<InterfaceTypeId> {
        match self {
            TypeId::Interface(v) => Some(v),
            TypeId::Object(_) => None
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyType {
    U64,
    F32,
    // Ref(ObjectTypeId),
    Subobject(TypeSelector),
    // RefSet(ObjectTypeId),
    // SubobjectSet(ObjectTypeId),
}

impl PropertyType {
    pub fn is_primitive_value(self) -> bool {
        match self {
            PropertyType::U64 => true,
            PropertyType::F32 => true,
            PropertyType::Subobject(_) => false,
        }
    }

    pub fn type_selector(self) -> Option<TypeSelector> {
        match self {
            PropertyType::U64 => None,
            PropertyType::F32 => None,
            PropertyType::Subobject(s) => Some(s)
        }
    }
}

pub struct InterfaceType {
    pub name: String,
    pub implementors: AHashSet<ObjectTypeId>,
}

pub struct ObjectType {
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub interfaces: InterfaceBits,

    //default_property_values: Vec<Value>,
    //default_object: ObjectId,
}

pub struct ObjectInfo {
    //valid: bool,
    //generation: u32,
    prototype: ObjectKey,
    object_type_id: ObjectTypeId,
    property_values: Vec<Value>,
    inherited_properties: PropertyBits,

    // base_object
    // overridden field mask
    // owner
    // id
}

// struct ObjectDetail {
//     property_values: Vec<Value>,
//     object: ObjectKey,
// }


// Registering enums?
// Registering implementing types (i.e interfaces)
// Set (adds/removew)
// Buffer types

// Guid/Reference

#[derive(Copy, Clone, Debug)]
struct ObjectTypeHash(u64);

// Fast lookup
// struct ObjectProperty {
//
// }


// struct ObjectMeta {
//     name: String,
//     hash: ObjectTypeHash,
//     properties: Vec<ObjectProperty>,
//     property_defs: Vec<ObjectPropertyDef>,
//
//     // name, name_hash, type, offset from root
//     // default object
//     // aspects
// }


// struct Object {
// }

#[test]
pub fn test_object_db() {

}

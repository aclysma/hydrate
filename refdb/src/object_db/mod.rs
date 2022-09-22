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

/// Intended to represent a single particular type
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

/// Intended to represent a range of allowed types
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
pub enum PropertyType {
    U64,
    F32,
    // Ref(ObjectTypeId),
    Subobject(TypeSelector),
    // RefSet(ObjectTypeId),
    SubobjectSet(TypeSelector),
}

impl PropertyType {
    pub fn is_primitive_value(self) -> bool {
        match self {
            PropertyType::U64 => true,
            PropertyType::F32 => true,
            PropertyType::Subobject(_) => false,
            PropertyType::SubobjectSet(_) => false,
        }
    }

    pub fn type_selector(self) -> Option<TypeSelector> {
        match self {
            PropertyType::U64 => None,
            PropertyType::F32 => None,
            PropertyType::Subobject(s) => Some(s),
            PropertyType::SubobjectSet(s) => Some(s)
        }
    }
}

/*
// example: typo'd name migrated to correct new name
{
// as long as this property remains listed, we will try to maintain backwards/forwards
// compatibility (continue writing field for old clients, )
    postion: {
        migrate_to: "position",
        type: vec3,
    },
    // shorthand, just use the type name
    position: vec3
}
*/
pub struct InterfaceType {
    pub name: String,
    pub implementors: AHashSet<ObjectTypeId>,
}

pub struct ObjectType {
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub interfaces: InterfaceBits,
}

pub struct ObjectInfo {
    prototype: ObjectKey,
    object_type_id: ObjectTypeId,
    property_values: Vec<Value>,
    inherited_properties: PropertyBits,
    owner: ObjectKey,

    //TODO (if needed): owner, id
}

// Registering enums?
// Registering implementing types (i.e interfaces)
// Set (adds/removes)
// Buffer types
// Guid/Reference

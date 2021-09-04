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

mod property_def;
pub use property_def::*;
use std::any::Any;

//
// up to 64 properties in a single object type
//
const MAX_PROPERTY_COUNT : usize = 64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PropertyIndex(u8);

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
    pub fn type_selector(self) -> Option<TypeSelector> {
        match self {
            PropertyType::U64 => None,
            PropertyType::F32 => None,
            PropertyType::Subobject(s) => Some(s)
        }
    }
}

struct InterfaceType {
    name: String,
    implementors: AHashSet<ObjectTypeId>,
}

struct ObjectType {
    name: String,
    properties: Vec<PropertyDef>,
    interfaces: InterfaceBits,

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

#[derive(Default)]
pub struct ObjectDb {
    interface_types: Vec<InterfaceType>,
    object_types: Vec<ObjectType>,
    objects: SlotMap<ObjectKey, ObjectInfo>,
    type_by_name: AHashMap<String, TypeId>,
    type_by_uuid: AHashMap<Uuid, TypeId>,
}

impl ObjectDb {
    fn inteface_type(&self, id: InterfaceTypeId) -> &InterfaceType {
        &self.interface_types[id.0 as usize]
    }

    fn interface_type_mut(&mut self, id: InterfaceTypeId) -> &mut InterfaceType {
        &mut self.interface_types[id.0 as usize]
    }

    fn object_type(&self, id: ObjectTypeId) -> &ObjectType {
        &self.object_types[id.0 as usize]
    }

    fn object_type_mut(&mut self, id: ObjectTypeId) -> &mut ObjectType {
        &mut self.object_types[id.0 as usize]
    }

    pub fn register_interface_type<S: Into<String>>(
        &mut self,
        uuid: uuid::Uuid,
        name: S,
    ) -> ObjectDbResult<InterfaceTypeId> {
        if self.object_types.len() >= MAX_INTERFACE_TYPE_COUNT {
            Err(format!("More than {} interface types not supported", MAX_INTERFACE_TYPE_COUNT))?;
        }

        // Create the type
        let name = name.into();
        let interface_type = InterfaceType {
            name: name.clone(),
            implementors: Default::default()
        };

        // Add the type to the list of types and appropriate lookups
        let type_id = InterfaceTypeId(self.interface_types.len() as u8);
        self.interface_types.push(interface_type);
        let old = self.type_by_name.insert(name, TypeId::Interface(type_id));
        assert!(old.is_none());
        let old = self.type_by_uuid.insert(uuid, TypeId::Interface(type_id));
        assert!(old.is_none());

        Ok(type_id)
    }

    pub fn register_object_type<S: Into<String>>(
        &mut self,
        uuid: uuid::Uuid,
        name: S,
        properties: &[PropertyDef]
    ) -> ObjectDbResult<ObjectTypeId> {
        if properties.len() > MAX_PROPERTY_COUNT {
            Err(format!("More than {} properties not supported", MAX_PROPERTY_COUNT))?;
        }

        if self.object_types.len() >= MAX_OBJECT_TYPE_COUNT {
            Err(format!("More than {} object types not supported", MAX_OBJECT_TYPE_COUNT))?;
        }

        for p in properties {
            if !p.default_value.can_convert_to(self, p.property_type) {
                Err(format!("The given value {:?} cannot be assigned to property type {:?}", p.default_value, p.property_type))?;
            }
        }

        // Create the type
        let name = name.into();
        let properties : Vec<PropertyDef> = properties.iter().cloned().collect();
        //let default_property_values = properties.iter().map(|x| x.default_value).cloned().collect();
        let object_type = ObjectType {
            name: name.clone(),
            properties,
            interfaces: Default::default()
            //default_property_values
            //default_object: ObjectKey::null()
        };

        // Add the type to the list of types and appropriate lookups
        let type_id = ObjectTypeId(self.object_types.len() as u16);
        self.object_types.push(object_type);
        let old = self.type_by_name.insert(name, TypeId::Object(type_id));
        assert!(old.is_none());
        let old = self.type_by_uuid.insert(uuid, TypeId::Object(type_id));
        assert!(old.is_none());

        // // Create the default object
        //let type_id = ObjectTypeId(type_index);
        // let default_object_id = self.create_object(type_id);
        // self.types[type_index as usize].default_object = default_object_id;
        // let mut default_object = &mut self.objects[default_object_id.0];
        //
        // // Initialize all the properties
        // let object_type = &self.types[type_index as usize];
        // for (p, v) in object_type.properties.iter().zip(&mut default_object.property_values) {
        //     *v = p.default_value.clone().convert_to(p.property_type).unwrap(); // can_convert_to() is checked above
        // }

        Ok(type_id)
    }

    pub fn add_implementor(&mut self, interface: InterfaceTypeId, implementor: ObjectTypeId) {
        self.interface_type_mut(interface).implementors.insert(implementor);
        self.object_type_mut(implementor).interfaces.set(interface.0 as usize, true);
    }

    //TODO: Get/Set default object? May not need it, we have default property values on the type

    pub fn find_type_by_name(&self, name: &str) -> Option<TypeId> {
        self.type_by_name.get(name).copied()
    }

    pub fn find_type_by_uuid(&self, uuid: &Uuid) -> Option<TypeId> {
        self.type_by_uuid.get(uuid).copied()
    }

    pub fn type_id_of_object(&self, object_id: ObjectId) -> ObjectTypeId {
        let object = &self.objects[object_id.0];
        object.object_type_id
    }

    pub fn is_object_type_allowed(&self, object_type_id: ObjectTypeId, selector: TypeSelector) -> bool {
        match selector {
            TypeSelector::Any => true,
            TypeSelector::Interface(type_id) => {
                self.object_type(object_type_id).interfaces.is_set(type_id.0 as usize)
            },
            TypeSelector::Object(type_id) => {
                object_type_id == type_id
            }
        }
    }

    // fn create_empty_object(&mut self, object_type_id: ObjectTypeId) -> ObjectKey {
    //
    // }

    // type_id: Which type to create an instance of
    // prototype: Defines which object we should use for our default values. If not set, uses default object
    // fn do_create_object(&mut self, type_id: ObjectTypeId, prototype: Option<ObjectId>, inherit_properties: bool) -> ObjectId {
    //     let object_type = &mut self.types[type_id.0 as usize];
    //
    //     let property_count = object_type.properties.len();
    //     let mut property_values = Vec::<Value>::with_capacity(property_count);
    //     for p in &object_type.properties {
    //         property_values.push(p.default_value.clone());
    //     }
    //
    //     let object_id = self.objects.insert(ObjectInfo {
    //         prototype: prototype.0.unwrap_or(ObjectKey::null()),
    //         object_type_id: type_id,
    //         property_values,
    //         inherited_properties: PropertyBits::default(),
    //     });
    //
    //     ObjectId(object_id)
    // }

    pub fn create_object(&mut self, type_id: ObjectTypeId) -> ObjectId {
        let object_type = &self.object_type(type_id);

        let property_count = object_type.properties.len();
        let mut property_values = Vec::<Value>::with_capacity(property_count);
        for p in &object_type.properties {
            property_values.push(p.default_value.clone());
        }

        let object_id = self.objects.insert(ObjectInfo {
            prototype: ObjectKey::null(),
            object_type_id: type_id,
            property_values,
            inherited_properties: PropertyBits::default(),
        });

        ObjectId(object_id)
    }

    pub fn create_prototype_instance(&mut self, prototype_object_id: ObjectId) -> ObjectId {
        debug_assert!(!prototype_object_id.0.is_null());
        let prototype = &self.objects[prototype_object_id.0];
        let object_type_id = prototype.object_type_id;
        let object_type = self.object_type(object_type_id);

        let property_count = object_type.properties.len();
        let mut property_values = Vec::<Value>::with_capacity(property_count);
        for p in &prototype.property_values {
            property_values.push(p.clone());
        }

        let mut inherited_properties = PropertyBits::default();
        inherited_properties.set_first_n(property_count, true);
        let object_id = self.objects.insert(ObjectInfo {
            prototype: prototype_object_id.0,
            object_type_id: object_type_id,
            property_values,
            inherited_properties,
        });

        ObjectId(object_id)
    }

    pub fn detach_from_prototype(&mut self, object_id: ObjectId) {
        let object = &mut self.objects[object_id.0];

        // Clear the prototype
        let prototype = object.prototype;
        object.prototype = ObjectKey::null();

        // Clear inherited properties
        let inherited_properties = object.inherited_properties;
        object.inherited_properties = PropertyBits::default();

        // Copy any inherited value from the prototype into this object.
        let property_count = object.property_values.len();
        if !prototype.is_null() {
            for i in 0..property_count {
                if inherited_properties.is_set(i) {
                    self.objects[object_id.0].property_values[i] = self.objects[prototype].property_values[i].clone();
                }
            }
        } else {
            let object_type = &self.object_types[object.object_type_id.0 as usize];
            for i in 0..property_count {
                if inherited_properties.is_set(i) {
                    object.property_values[i] = object_type.properties[i].default_value.clone();
                }
            }
        }
    }

    pub fn clone_object(&mut self, object_to_clone: ObjectId) -> ObjectId {
        let mut object = self.create_prototype_instance(object_to_clone);
        self.detach_from_prototype(object);
        object
    }

    //pub fn copy_object()

    pub fn find_property(&self, type_id: ObjectTypeId, name: &str) -> Option<PropertyIndex> {
        let p = self.object_type(type_id).properties.iter().position(|x| x.name == name);
        p.map(|x| PropertyIndex(x as u8))
    }

    // pub fn value(&self, object_id: ObjectId, property: PropertyIndex) -> Value {
    //     self.objects[object_id.0].property_values[property.0 as usize]
    // }
    //
    // pub fn value_mut(&mut self, object_id: ObjectId, property: PropertyIndex) -> &mut Value {
    //     &mut self.objects[object_id.0].property_values[property.0 as usize]
    // }

    fn property_value(&self, mut object_id: ObjectId, property: PropertyIndex) -> &Value {
        let mut object_key = object_id.0;
        loop {
            if object_key.is_null() {
                // Use the type's default value
                break;
            }

            let object = &self.objects[object_key];
            if !object.inherited_properties.is_set(property.0 as usize) {
                // Use this object's value
                break;
            } else {
                // Continue looping, checking the next prototype up the tree
                object_key = object.prototype;
            }
        }

        if object_key.is_null() {
            let object_type_id = self.objects[object_id.0].object_type_id;
            &self.object_type(object_type_id).properties[property.0 as usize].default_value
        } else {
            &self.objects[object_key].property_values[property.0 as usize]
        }
    }

    pub fn get_u64(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<u64> {
        self.property_value(object_id, property).get_u64()
    }

    pub fn set_u64(&mut self, object_id: ObjectId, property: PropertyIndex, value: u64) -> ObjectDbResult<()> {
        let object = &mut self.objects[object_id.0];
        object.property_values[property.0 as usize].set_u64(value)?;
        object.inherited_properties.set(property.0 as usize, false);
        Ok(())
    }

    pub fn get_f32(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<f32> {
        self.property_value(object_id, property).get_f32()
    }

    pub fn set_f32(&mut self, object_id: ObjectId, property: PropertyIndex, value: f32) -> ObjectDbResult<()> {
        let object = &mut self.objects[object_id.0];
        object.property_values[property.0 as usize].set_f32(value)?;
        object.inherited_properties.set(property.0 as usize, false);
        Ok(())
    }

    pub fn clear_property_override(&mut self, object_id: ObjectId, property: PropertyIndex) {
        let object = &mut self.objects[object_id.0];
        object.inherited_properties.set(property.0 as usize, true);
    }

    pub fn set_property_override(&mut self, object_id: ObjectId, property: PropertyIndex) {
        let current_value = self.property_value(object_id, property).clone();
        let object = &mut self.objects[object_id.0];
        object.property_values[property.0 as usize] = current_value;
        object.inherited_properties.set(property.0 as usize, false);
    }

    //TODO: Improve API to handle a chain of prototypes?
    pub fn apply_property_override_to_prototype(&mut self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<()> {
        let current_value = self.property_value(object_id, property).clone();
        let prototype = self.objects[object_id.0].prototype;
        assert!(!prototype.is_null());
        if prototype.is_null() {

        }

        let prototype = &mut self.objects[prototype];
        prototype.property_values[property.0 as usize] = current_value;
        prototype.inherited_properties.set(property.0 as usize, false);

        let object = &mut self.objects[object_id.0];
        object.inherited_properties.set(property.0 as usize, true);
        Ok(())
    }

    pub fn is_property_overriden(&mut self, object_id: ObjectId, property: PropertyIndex) -> bool {
        let object = &mut self.objects[object_id.0];
        object.inherited_properties.is_set(property.0 as usize)
    }

    // pub fn get_subobject(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<ObjectId> {
    //     self.objects[object_id.0].property_values[property.0 as usize].get_subobject().map(|x| ObjectId(x))
    // }

    pub fn set_subobject(&mut self, object_id: ObjectId, property: PropertyIndex, subobject_id: ObjectId) -> ObjectDbResult<()> {
        let object_type_id = self.type_id_of_object(object_id);
        let object_type = self.object_type(object_type_id);
        let type_selector = object_type.properties[property.0 as usize].property_type.type_selector().ok_or(ObjectDbError::TypeError)?;
        let subobject_type_id = self.type_id_of_object(subobject_id);
        if !self.is_object_type_allowed(subobject_type_id, type_selector) {
            Err(ObjectDbError::TypeError)?;
        }

        self.objects[object_id.0].property_values[property.0 as usize].set_subobject(subobject_id.0)
    }




    // Allocator settings

    // Buffer Management

    // String Management

    // register_type
    // set_default_object/set_default_values?
    // get_default_object
    // is_default
    //
    // register/applying interfaces
    //
    // find/enumerate types
    // find/enumerate properties
    // find/enumerate interfaces
    //
    // find objects (type, filtering, etc.)
    //
    // undo/redo
    //
    // creating/cloning objects
    // adding/removing subobjects
    //
    // garbage collect deleted?
    //
    // get/set properties, subobjects
    //
    // save
    //
    // apply_to_base_prefab
    // detach from prototype
    //
    // get_base_prefab
    // check_if_overridden
    //
    // change detection
    //


    //
}

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

use ahash::{AHashMap, AHashSet};
use uuid::Uuid;
use std::collections::VecDeque;
use slotmap::SlotMap;
use slotmap::Key;

mod error;
pub use error::*;

mod value;
pub use value::*;

mod property_def;
pub use property_def::*;

// up to 64 properties
const MAX_PROPERTY_COUNT : usize = 64;
//pub type PropertyBits = bitvec::array::BitArray<bitvec::order::Lsb0, [u64; 1]>;

#[derive(Copy, Clone)]
pub struct PropertyIndex(u8);

const MAX_TYPE_COUNT : usize = u16::MAX as usize;

#[derive(Copy, Clone)]
pub struct ObjectTypeId(u16);

#[derive(Copy, Clone, Default)]
struct PropertyBits {
    bits: u64
}

impl PropertyBits {
    fn is_set(self, index: usize) -> bool {
        (self.bits | (1<<(index as u64))) != 0
    }

    fn set(mut self, index: usize, value: bool) -> Self {
        if value {
            self.bits |= (1<<(index as u64));
        } else {
            self.bits &= !(1<<(index as u64));
        }

        self
    }

    fn set_first_n(mut self, count: usize, value: bool) -> Self {
        if value {
            if count == 64 {
                self.bits = !0;
            } else {
                self.bits |= (1<<((count as u64) + 1)) - 1;
            }
        } else {
            if count == 64 {
                self.bits = 0;
            } else {
                self.bits &= !((1<<((count as u64) + 1)) - 1);
            }
        }

        self
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyType {
    U64,
    F32,
    // Ref(ObjectTypeId),
    // Subobject(ObjectTypeId),
    // RefSet(ObjectTypeId),
    // SubobjectSet(ObjectTypeId),
}

slotmap::new_key_type! { pub struct ObjectKey; }

#[derive(Copy, Clone)]
pub struct ObjectId(ObjectKey);

struct ObjectType {
    name: String,
    properties: Vec<PropertyDef>,
}

pub struct ObjectInfo {
    //valid: bool,
    //generation: u32,
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
    types: Vec<ObjectType>,
    objects: SlotMap<ObjectKey, ObjectInfo>,
    type_by_name: AHashMap<String, u16>,
    type_by_uuid: AHashMap<Uuid, u16>,
}

impl ObjectDb {
    pub fn register_type<S: Into<String>>(&mut self, uuid: uuid::Uuid, name: S, properties: &[PropertyDef]) -> ObjectDbResult<ObjectTypeId> {
        if properties.len() > MAX_PROPERTY_COUNT {
            Err(format!("More than {} properties not supported", MAX_PROPERTY_COUNT))?;
        }

        if self.types.len() >= MAX_TYPE_COUNT {
            Err(format!("More than {} types not supported", MAX_TYPE_COUNT))?;
        }

        for p in properties {
            if !p.default_value.can_convert_to(p.property_type) {
                Err(format!("The given value {:?} cannot be assigned to property type {:?}", p.default_value, p.property_type))?;
            }
        }

        // Create the type
        let name = name.into();
        let properties : Vec<PropertyDef> = properties.iter().cloned().collect();
        let object_type = ObjectType {
            name: name.clone(),
            properties,
        };

        // Add the type to the list of types and appropriate lookups
        let type_index = self.types.len() as u16;
        self.types.push(object_type);
        let old = self.type_by_name.insert(name, type_index);
        assert!(old.is_none());
        let old = self.type_by_uuid.insert(uuid, type_index);
        assert!(old.is_none());

        // Create the default object
        let type_id = ObjectTypeId(type_index);
        let default_object_id = self.create_object(type_id);
        let mut default_object = &mut self.objects[default_object_id.0];

        // Initialize all the properties
        let object_type = &self.types[type_index as usize];
        for (p, v) in object_type.properties.iter().zip(&mut default_object.property_values) {
            *v = p.default_value.clone().convert_to(p.property_type).unwrap(); // can_convert_to() is checked above
        }

        Ok(type_id)
    }

    //TODO: Get/Set default object? May not need it, we have default property values

    pub fn get_type_by_name(&self, name: &str) -> Option<ObjectTypeId> {
        self.type_by_name.get(name).map(|x| ObjectTypeId(*x as u16))
    }

    pub fn get_type_by_uuid(&self, uuid: &Uuid) -> Option<ObjectTypeId> {
        self.type_by_uuid.get(uuid).map(|x| ObjectTypeId(*x as u16))
    }

    pub fn create_object(&mut self, type_id: ObjectTypeId) -> ObjectId {
        let object_type = &mut self.types[type_id.0 as usize];

        let property_count = object_type.properties.len();
        let mut property_values = Vec::<Value>::with_capacity(property_count);
        for p in &object_type.properties {
            property_values.push(p.default_value.clone());
        }

        let object_id = self.objects.insert(ObjectInfo {
            object_type_id: type_id,
            property_values,
            inherited_properties: PropertyBits::default(),
        });

        ObjectId(object_id)
    }

    pub fn find_property(&self, type_id: ObjectTypeId, name: &str) -> Option<PropertyIndex> {
        let p = self.types[type_id.0 as usize].properties.iter().position(|x| x.name == name);
        p.map(|x| PropertyIndex(x as u8))
    }

    // pub fn value(&self, object_id: ObjectId, property: PropertyIndex) -> Value {
    //     self.objects[object_id.0].property_values[property.0 as usize]
    // }
    //
    // pub fn value_mut(&mut self, object_id: ObjectId, property: PropertyIndex) -> &mut Value {
    //     &mut self.objects[object_id.0].property_values[property.0 as usize]
    // }

    pub fn get_u64(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<u64> {
        self.objects[object_id.0].property_values[property.0 as usize].get_u64()
    }

    pub fn set_u64(&mut self, object_id: ObjectId, property: PropertyIndex, value: u64) -> ObjectDbResult<()> {
        self.objects[object_id.0].property_values[property.0 as usize].set_u64(value)
    }

    pub fn get_f32(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<f32> {
        self.objects[object_id.0].property_values[property.0 as usize].get_f32()
    }

    pub fn set_f32(&mut self, object_id: ObjectId, property: PropertyIndex, value: f32) -> ObjectDbResult<()> {
        self.objects[object_id.0].property_values[property.0 as usize].set_f32(value)
    }

    // pub fn get_subobject(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<ObjectId> {
    //     self.objects[object_id.0].property_values[property.0 as usize].get_subobject().map(|x| ObjectId(x))
    // }
    //
    // pub fn set_subobject(&mut self, object_id: ObjectId, property: PropertyIndex, value: ObjectId) -> ObjectDbResult<()> {
    //     //TODO: Type verification
    //     self.objects[object_id.0].property_values[property.0 as usize].set_subobject(value.0)
    // }




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

use ahash::AHashMap;

enum PropertyType {
    U64,
    F32,
}

// enum PropertyEditor {
//     Default
// }

// Public API
struct PropertyDef {
    // name, type, editor, legal subobject types, tooltip, transient, UI name
    name: String,
    property_type: PropertyType,
}

// struct TypeDef<'a> {
//     properties: &'a [PropertyDef]
// }

struct ObjectProperty {

}

struct ObjectType {
    name: String,
    properties: Vec<PropertyDef>
}


#[derive(Default)]
struct ObjectDb {
    types: Vec<ObjectType>,
    type_by_name: AHashMap<String, usize>
}

impl ObjectDb {
    fn register_type<S: Into<String>>(&mut self, name: S, properties: &[PropertyDef]) {
        let name = name.into();
        let mut properties : Vec<PropertyDef> = properties.iter().collect();
        let object_type = ObjectType {
            name: name.clone(),
            properties
        };

        let type_index = self.types.len();
        self.types.push(object_type);
        self.type_by_name.insert(name, type_index);
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


struct ObjectMeta {
    name: String,
    hash: ObjectTypeHash,
    properties: Vec<ObjectProperty>,
    property_defs: Vec<ObjectPropertyDef>,

    // name, name_hash, type, offset from root
    // default object
    // aspects
}

enum Value {
    Int32(i32),
    Uint64(u64),
}

struct Object {
    // base_object
    // overridden field mask
    // owner
    // id
    //
}

#[test]
pub fn test_object_db() {
    let mut db = ObjectDb::default();
    db.register_type("Vec3", &[
        PropertyDef { name: "x".to_string(), property_type: PropertyType::F32 },
        PropertyDef { name: "y".to_string(), property_type: PropertyType::F32 },
        PropertyDef { name: "z".to_string(), property_type: PropertyType::F32 },
    ])

    db.register_type("Vec4", &[
        PropertyDef { name: "x".to_string(), property_type: PropertyType::F32 },
        PropertyDef { name: "y".to_string(), property_type: PropertyType::F32 },
        PropertyDef { name: "z".to_string(), property_type: PropertyType::F32 },
    ])
}
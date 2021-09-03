use uuid::Uuid;

mod hashing;
mod object_db;
use object_db::*;

pub fn run() {
    let mut db = ObjectDb::default();
    let vec3_type = db.register_type(Uuid::parse_str("6f9cc150-886e-4800-9ba1-cd0534847c9d").unwrap(), "Vec3", &[
        PropertyDef::new_f32("x"),
        PropertyDef::new_f32("y"),
        PropertyDef::new_f32("z"),
    ]).unwrap();

    let vec3_prototype = db.create_object(vec3_type);
    let x_property = db.find_property(vec3_type, "x").unwrap();

    // Set prototype to some non-default value
    db.set_f32(vec3_prototype, x_property, 1.0).unwrap();
    let value = db.get_f32(vec3_prototype, x_property).unwrap();
    assert_eq!(1.0, value);

    // Create a prototype instance, by default inheriting the same values
    let vec3_instance = db.create_prototype_instance(vec3_prototype);
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(1.0, value);

    // Modify the prototype and verify the instance has the new property
    db.set_f32(vec3_prototype, x_property, 2.0).unwrap();
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(2.0, value);

    // Set the instance, overriding the value
    db.set_f32(vec3_instance, x_property, 3.0).unwrap();
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(3.0, value);

    // Clear the override, it now uses the prototype's value instead
    db.clear_property_override(vec3_instance, x_property);
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(2.0, value);

    // Detach the instance to be its own object, no longer modified by the prototype. This will copy
    // data from the prototype into the instance to ensure this does not modify the apparent property
    // values of the instance
    db.detach_from_prototype(vec3_instance);
    db.set_f32(vec3_prototype, x_property, 4.0).unwrap();
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(2.0, value);

    // Reseting the property now will return it to the default
    db.clear_property_override(vec3_instance, x_property);
    let value4 = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(0.0, value4);

    // Create a new instance from the prototype
    let vec3_instance = db.create_prototype_instance(vec3_prototype);
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(4.0, value);

    // Override the property
    db.set_f32(vec3_instance, x_property, 5.0);
    let value = db.get_f32(vec3_instance, x_property).unwrap();
    assert_eq!(5.0, value);

    // Apply it to the prototype
    db.apply_property_override_to_prototype(vec3_instance, x_property);
    let value = db.get_f32(vec3_prototype, x_property).unwrap();
    assert_eq!(5.0, value);


    // let transform_type = db.register_type(Uuid::parse_str("e0fe7ed4-8e02-4d08-b2c9-7ac7d21b7c9f").unwrap(), "Transform", &[
    //     PropertyDef::new_subobject("position", vec3_type),
    //     PropertyDef::new_subobject("rotation", vec4_type),
    //     PropertyDef::new_subobject("scale", vec3_type),
    // ]).unwrap();

    // let transform_object = db.create_object(transform_type);
    // let position_property = db.find_property(transform_type, "position").unwrap();
    // db.set_subobject(transform_object, position_property, vec3_object);





    //let vec4_type = db.get_type_by_name("Vec4").unwrap();
}

use super::*;

fn register_vec3(db: &mut ObjectDb) -> ObjectTypeId {
    db.register_object_type(Uuid::parse_str("6f9cc150-886e-4800-9ba1-cd0534847c9d").unwrap(), "Vec3", &[
        PropertyDef::new_f32("x"),
        PropertyDef::new_f32("y"),
        PropertyDef::new_f32("z"),
    ]).unwrap()
}

{
    uuid: "5dfbf0cc-b76f-4210-84c6-73343b4b0d9e",
    name: "vec3",
    rs_out: "src/whatever/vec3.rs",
    properties: {
        x: f32,
        y: f32,
        z: f32,
    }
}

fn register_vec4(db: &mut ObjectDb) -> ObjectTypeId {
    db.register_object_type(
        Uuid::parse_str("37e94237-a173-47c1-a8fa-b49b2448bf3e").unwrap(),
        "Vec3",
        &[
            PropertyDef::new_f32("x"),
            PropertyDef::new_f32("y"),
            PropertyDef::new_f32("z"),
        ]
    ).unwrap()
}

fn register_transform(db: &mut ObjectDb) -> ObjectTypeId {
    let vec3_type = db.find_type_by_name("Vec3").unwrap();
    let vec4_type = db.find_type_by_name("Vec4").unwrap();

    db.register_object_type(Uuid::parse_str("962ff571-ef95-4761-baaa-37efbbccef43").unwrap(), "Transform", &[
        PropertyDef::new_subobject("position", vec3_type),
        PropertyDef::new_subobject("rotation", vec4_type),
        PropertyDef::new_subobject("scale", vec3_type),
    ]).unwrap()
}

#[test]
pub fn test_override_f32_property_on_root_object() {
    let mut db = ObjectDb::default();
    let vec3_type = register_vec3(&mut db);

    let x_property = db.find_property(vec3_type, "x").unwrap();

    // Create a prototype object
    let vec3_prototype = db.create_object(vec3_type);
    let value = db.get_f32(vec3_prototype, x_property).unwrap();
    assert_eq!(0.0, value);

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
}

//TODO: Verify properties are used with the correct object types?

//TODO: Default object for interface props?
#[test]
pub fn test_override_f32_property_on_subobject() {
    let mut db = ObjectDb::default();
    let vec3_type = register_vec3(&mut db);
    let _vec4_type = register_vec4(&mut db);
    let transform_type = register_transform(&mut db);

    let x_property = db.find_property(vec3_type, "x").unwrap();
    let position_property = db.find_property(transform_type, "position").unwrap();

    // Create a prototype object
    let prototype_transform = db.create_object(transform_type);
    let prototype_position = db.get_subobject(prototype_transform, position_property).unwrap();
    let value = db.get_f32(prototype_position, x_property).unwrap();
    assert_eq!(0.0, value);

    // Set prototype to some non-default value
    db.set_f32(prototype_position, x_property, 1.0).unwrap();
    let value = db.get_f32(prototype_position, x_property).unwrap();
    assert_eq!(1.0, value);

    // Create a prototype instance, by default inheriting the same values
    let instance_transform = db.create_prototype_instance(prototype_transform);
    let instance_position = db.get_subobject(instance_transform, position_property).unwrap();
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(1.0, value);

    // Modify the prototype and verify the instance has the new property
    db.set_f32(prototype_position, x_property, 2.0).unwrap();
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(2.0, value);

    // Set the instance, overriding the value
    db.set_f32(instance_position, x_property, 3.0).unwrap();
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(3.0, value);

    // Clear the override, it now uses the prototype's value instead
    db.clear_property_override(instance_position, x_property);
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(2.0, value);

    // Detach the instance to be its own object, no longer modified by the prototype. This will copy
    // data from the prototype into the instance to ensure this does not modify the apparent property
    // values of the instance
    db.detach_from_prototype(instance_position);
    db.set_f32(prototype_position, x_property, 4.0).unwrap();
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(2.0, value);

    // Reseting the property now will return it to the default
    db.clear_property_override(instance_position, x_property);
    let value4 = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(0.0, value4);

    // Create a new instance from the prototype
    let instance_position = db.create_prototype_instance(prototype_position);
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(4.0, value);

    // Override the property
    db.set_f32(instance_position, x_property, 5.0);
    let value = db.get_f32(instance_position, x_property).unwrap();
    assert_eq!(5.0, value);

    // Apply it to the prototype
    db.apply_property_override_to_prototype(instance_position, x_property);
    let value = db.get_f32(prototype_position, x_property).unwrap();
    assert_eq!(5.0, value);
}

fn register_vec3_set(db: &mut ObjectDb) -> ObjectTypeId {
    let vec3_type = db.find_type_by_name("Vec3").unwrap();

    db.register_object_type(Uuid::parse_str("c95a1326-4261-493d-9f20-11709b4ceda0").unwrap(), "Vec3Set", &[
        PropertyDef::new_subobject_set("vec3_set", vec3_type),
    ]).unwrap()
}

#[test]
pub fn test_subobject_set() {
    let mut db = ObjectDb::default();
    let vec3_type = register_vec3(&mut db);
    let vec3_set_type = register_vec3_set(&mut db);
    let vec3_set_property = db.find_property(vec3_set_type, "vec3_set");

    let v3_value1 = db.create_object(vec3_type);
    let v3_value2 = db.create_object(vec3_type);
    let v3_value3 = db.create_object(vec3_type);
    let v3_set = db.create_object(vec3_set_type);

    db.add_subobject_to_set(v3_set, vec3_set_property, v3_value1);
    db.add_subobject_to_set(v3_set, vec3_set_property, v3_value2);
    db.add_subobject_to_set(v3_set, vec3_set_property, v3_value3);
}
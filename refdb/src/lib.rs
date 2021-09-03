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

    let vec4_type = db.register_type(Uuid::parse_str("402ff791-2f65-44a8-af70-960c5ceb861f").unwrap(), "Vec4", &[
        PropertyDef::new_f32("x"),
        PropertyDef::new_f32("y"),
        PropertyDef::new_f32("z"),
        PropertyDef::new_f32("w"),
    ]).unwrap();

    let vec3_object = db.create_object(vec3_type);
    let x_property = db.find_property(vec3_type, "x").unwrap();

    db.set_f32(vec3_object, x_property, 1.0).unwrap();
    let value = db.get_f32(vec3_object, x_property).unwrap();
    dbg!(value);


    // let transform_type = db.register_type(Uuid::parse_str("e0fe7ed4-8e02-4d08-b2c9-7ac7d21b7c9f").unwrap(), "Transform", &[
    //     PropertyDef::new_subobject("position", vec3_type),
    //     PropertyDef::new_subobject("rotation", vec4_type),
    //     PropertyDef::new_subobject("scale", vec3_type),
    // ]).unwrap();

    // let transform_object = db.create_object(transform_type);
    // let position_property = db.find_property(transform_type, "position").unwrap();
    // db.set_subobject(transform_object, position_property, vec3_object);





    let vec4_type = db.get_type_by_name("Vec4").unwrap();
}

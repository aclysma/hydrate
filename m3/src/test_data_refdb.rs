use refdb::{ObjectTypeId, ObjectId};
use uuid::Uuid;
use refdb::ObjectDb;
use refdb::PropertyDef;
use refdb::TypeId::Object;

fn register_vec3(db: &mut ObjectDb) -> ObjectTypeId {
    db.register_object_type(Uuid::parse_str("6f9cc150-886e-4800-9ba1-cd0534847c9d").unwrap(), "Vec3", &[
        PropertyDef::new_f32("x"),
        PropertyDef::new_f32("y"),
        PropertyDef::new_f32("z"),
    ]).unwrap()
}

fn register_vec4(db: &mut ObjectDb) -> ObjectTypeId {
    db.register_object_type(Uuid::parse_str("37e94237-a173-47c1-a8fa-b49b2448bf3e").unwrap(), "Vec4", &[
        PropertyDef::new_f32("x"),
        PropertyDef::new_f32("y"),
        PropertyDef::new_f32("z"),
        PropertyDef::new_f32("w"),
    ]).unwrap()
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


pub struct TestData {
    pub db: ObjectDb,
    pub prototype_obj: ObjectId,
    pub instance_obj: ObjectId,
}

pub fn setup_test_data() -> TestData {
    let mut db = ObjectDb::default();
    let vec3_type = register_vec3(&mut db);
    let vec4_type = register_vec4(&mut db);
    let transform_type = register_transform(&mut db);

    let x_property = db.find_property(vec3_type, "x").unwrap();
    let position_property = db.find_property(transform_type, "position").unwrap();

    // Create a prototype object
    let prototype_transform = db.create_object(transform_type);
    let prototype_position = db.get_subobject(prototype_transform, position_property).unwrap();

    // Set prototype to some non-default value
    db.set_f32(prototype_position, x_property, 1.0).unwrap();

    // Create a prototype instance, by default inheriting the same values. Override a value
    let instance_transform = db.create_prototype_instance(prototype_transform);
    let instance_position = db.get_subobject(instance_transform, position_property).unwrap();
    db.set_f32(instance_position, x_property, 2.0).unwrap();


    TestData {
        db,
        prototype_obj: prototype_transform,
        instance_obj: instance_transform
    }
}
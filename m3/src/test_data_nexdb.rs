
pub struct TestData {
    pub db: nexdb::Database,
    pub prototype_obj: nexdb::ObjectId,
    pub instance_obj: nexdb::ObjectId,
}

pub fn setup_test_data() -> TestData {
    let mut db = nexdb::Database::default();

    let vec3_schema_object = db.register_record_type("Vec3", |builder| {
        builder.add_f64("x");
        builder.add_f64("y");
        builder.add_f64("z");
    });

    let vec4_schema_object = db.register_record_type("Vec4", |builder| {
        builder.add_f32("x");
        builder.add_f32("y");
        builder.add_f32("z");
        builder.add_f32("w");
    });

    let transform_schema_object = db.register_record_type("Transform", |builder| {
        builder.add_struct("position", &vec3_schema_object);
        builder.add_struct("rotation", &vec4_schema_object);
        builder.add_struct("scale", &vec3_schema_object);
    });

    let prototype_obj = db.new_object(&transform_schema_object);
    let instance_obj = db.new_object_from_prototype(prototype_obj);

    db.set_property_override(prototype_obj, &"position.x", nexdb::Value::F64(10.0));
    db.set_property_override(instance_obj, &"position.x", nexdb::Value::F64(20.0));

    TestData {
        db,
        prototype_obj,
        instance_obj
    }
}
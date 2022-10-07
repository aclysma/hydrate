use nexdb::{Schema, SchemaDefType};

pub struct TestData {
    pub db: nexdb::Database,
    pub prototype_obj: nexdb::ObjectId,
    pub instance_obj: nexdb::ObjectId,
}

pub fn setup_test_data() -> TestData {
    let mut linker = nexdb::SchemaLinker::default();

    linker
        .register_record_type("Vec3", |builder| {
            builder.add_f64("x");
            builder.add_f64("y");
            builder.add_f64("z");
        })
        .unwrap();

    linker
        .register_record_type("Vec4", |builder| {
            builder.add_f32("x");
            builder.add_f32("y");
            builder.add_f32("z");
            builder.add_f32("w");
        })
        .unwrap();

    linker
        .register_record_type("AllFields", |builder| {
            builder.add_nullable("nullable_bool", SchemaDefType::Boolean);
            builder.add_nullable(
                "nullable_vec3",
                SchemaDefType::NamedType("Vec3".to_string()),
            );
            builder.add_boolean("boolean");
            builder.add_i32("i32");
            builder.add_i64("i64");
            builder.add_u32("u32");
            builder.add_u64("u64");
            builder.add_f32("f32");
            builder.add_f64("f64");
            builder.add_string("string");
            builder.add_dynamic_array("dynamic_array_i32", SchemaDefType::I32);
            builder.add_dynamic_array(
                "dynamic_array_vec3",
                SchemaDefType::NamedType("Vec3".to_string()),
            );
        })
        .unwrap();

    linker
        .register_record_type("Transform", |builder| {
            builder.add_struct("all_fields", "AllFields");
            builder.add_struct("position", "Vec3");
            builder.add_struct("rotation", "Vec4");
            builder.add_struct("scale", "Vec3");
        })
        .unwrap();

    let mut db = nexdb::Database::default();
    db.add_linked_types(linker).unwrap();
    let transform_schema_object = db
        .find_named_type("Transform")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let prototype_obj = db.new_object(&transform_schema_object);
    let instance_obj = db.new_object_from_prototype(prototype_obj);

    db.set_property_override(prototype_obj, "position.x", nexdb::Value::F64(10.0));
    db.set_property_override(instance_obj, "position.x", nexdb::Value::F64(20.0));

    let prototype_array_element_1 =
        db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_i32");
    let prototype_array_element_2 =
        db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_i32");
    let instance_array_element_1 =
        db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_i32");
    let instance_array_element_2 =
        db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_i32");
    let instance_array_element_3 =
        db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_i32");

    let prototype_array_element_1 =
        db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_vec3");
    let prototype_array_element_2 =
        db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_vec3");
    let instance_array_element_1 =
        db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_vec3");
    let instance_array_element_2 =
        db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_vec3");
    let instance_array_element_3 =
        db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_vec3");

    TestData {
        db,
        prototype_obj,
        instance_obj,
    }
}

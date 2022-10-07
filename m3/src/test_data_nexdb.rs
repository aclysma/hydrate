use std::path::PathBuf;
use nexdb::{Schema, SchemaDefType};

pub struct TestData {
    pub db: nexdb::Database,
    pub prototype_obj: nexdb::ObjectId,
    pub instance_obj: nexdb::ObjectId,
}

pub fn setup_test_data() -> TestData {
    let mut linker = nexdb::SchemaLinker::default();
    let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/schema"));
    linker.add_source_dir(&path, "*.json").unwrap();

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

use std::path::PathBuf;
use std::sync::Arc;

use nexdb::edit_context::EditContext;
use nexdb::*;
use nexdb::json::TreeSourceDataStorageJsonSingleFile;

fn main() {
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/data/simple_example/schema"));
    let default_location = ObjectLocation::new(ObjectSourceId::new(), ObjectPath::new("test.nxt"));

    let (schema_cache, data) = {
        let mut linker = SchemaLinker::default();
        linker.add_source_dir(path.clone(), "**.json").unwrap();

        let mut schema_set = SchemaSet::default();
        schema_set.add_linked_types(linker).unwrap();
        let schema_set = Arc::new(schema_set);

        let mut edit_model = EditorModel::new(schema_set.clone());
        let mut edit_context = edit_model.root_edit_context_mut();
        //linker.finish();

        let vec3_type = edit_context
            .find_named_type("Vec3")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let vec3_obj = edit_context.new_object(&default_location, &vec3_type);
        edit_context.set_property_override(vec3_obj, "x", Value::F32(10.0));
        edit_context.set_property_override(vec3_obj, "y", Value::F32(20.0));
        edit_context.set_property_override(vec3_obj, "z", Value::F32(30.0));

        let aabb_type = edit_context
            .find_named_type("AABB")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();
        let aabb_obj = edit_context.new_object(&default_location, &aabb_type);
        edit_context.set_property_override(aabb_obj, "min.x", Value::F32(10.0));
        edit_context.set_property_override(aabb_obj, "min.y", Value::F32(20.0));
        edit_context.set_property_override(aabb_obj, "min.z", Value::F32(30.0));

        edit_context.set_property_override(aabb_obj, "max.x", Value::F32(40.0));
        edit_context.set_property_override(aabb_obj, "max.y", Value::F32(50.0));
        edit_context.set_property_override(aabb_obj, "max.z", Value::F32(60.0));

        let dyn_array_type = edit_context
            .find_named_type("DynArrayNullableTest")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();
        let dyn_array_obj = edit_context.new_object(&default_location, &dyn_array_type);

        let element1 = edit_context.add_dynamic_array_override(dyn_array_obj, "dyn_array");
        let _element2 = edit_context.add_dynamic_array_override(dyn_array_obj, "dyn_array");

        edit_context.set_override_behavior(dyn_array_obj, "dyn_array", OverrideBehavior::Replace);
        edit_context.set_null_override(
            dyn_array_obj,
            format!("dyn_array.{}", element1),
            NullOverride::SetNonNull,
        );
        edit_context.set_property_override(
            dyn_array_obj,
            format!("dyn_array.{}.value", element1),
            Value::F32(10.0),
        );

        let schema_cache = SchemaCacheSingleFile::store_string(edit_context.schema_set());
        let object_ids: Vec<_> = edit_context.all_objects().copied().collect();
        let data = TreeSourceDataStorageJsonSingleFile::store_objects_to_string(&edit_context, &object_ids);

        println!("Schema Cache: {}", schema_cache);
        println!("Data: {}", data);
        (schema_cache, data)
    };

    println!("--------------------- Restoring with linker");
    {
        let mut linker2 = SchemaLinker::default();
        linker2.add_source_dir(path, "**.json").unwrap();

        let mut schema_set = SchemaSet::default();
        schema_set.add_linked_types(linker2).unwrap();
        let schema_set = Arc::new(schema_set);


        let mut edit_model2 = EditorModel::new(schema_set.clone());
        let mut edit_context2 = edit_model2.root_edit_context_mut();

        TreeSourceDataStorageJsonSingleFile::load_objects_from_string(&mut edit_context2, default_location.clone(), &data);
        let object_ids: Vec<_> = edit_context2.all_objects().copied().collect();
        let data2 = TreeSourceDataStorageJsonSingleFile::store_objects_to_string(&edit_context2, &object_ids);
        println!("Data2: {}", data2);
        assert_eq!(data, data2);

        let schema_cache2 = SchemaCacheSingleFile::store_string(&*schema_set);
        assert_eq!(schema_cache, schema_cache2);
    }

    println!("--------------------- Restoring with schema cache");
    {
        let mut schema_set = SchemaSet::default();
        SchemaCacheSingleFile::load_string(&mut schema_set, &schema_cache);
        let schema_set = Arc::new(schema_set);

        let mut edit_model3 = EditorModel::new(schema_set.clone());
        let mut edit_context3 = edit_model3.root_edit_context_mut();

        TreeSourceDataStorageJsonSingleFile::load_objects_from_string(&mut edit_context3, default_location, &data);
        let object_ids: Vec<_> = edit_context3.all_objects().copied().collect();
        let data3 = TreeSourceDataStorageJsonSingleFile::store_objects_to_string(&edit_context3, &object_ids);
        println!("---------------------");
        println!("Data3: {}", data3);
        assert_eq!(data, data3);

        let schema_cache3 = SchemaCacheSingleFile::store_string(&schema_set);
        assert_eq!(schema_cache, schema_cache3);
    }

    let data_file_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/data/simple_example/data_file_out.json"
    ));
    std::fs::write(data_file_path, data).unwrap();

    let schema_cache_file_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/data/simple_example/schema_cache/schema_cache_file_out.json"
    ));
    std::fs::write(schema_cache_file_path, schema_cache).unwrap();
}

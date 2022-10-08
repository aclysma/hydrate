use log::MetadataBuilder;
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use serde_json::Value::Null;

use nexdb::*;

fn main() {
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/schema"));

    let (schema_cache, data) = {
        let mut linker = SchemaLinker::default();
        linker.add_source_dir(path.clone(), "*.json").unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker);
        //linker.finish();

        let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap().clone();

        let vec3_obj = db.new_object(&vec3_type);
        db.set_property_override(vec3_obj, "x", Value::F32(10.0));
        db.set_property_override(vec3_obj, "y", Value::F32(20.0));
        db.set_property_override(vec3_obj, "z", Value::F32(30.0));


        let aabb_type = db.find_named_type("AABB").unwrap().as_record().unwrap().clone();
        let aabb_obj = db.new_object(&aabb_type);
        db.set_property_override(aabb_obj, "min.x", Value::F32(10.0));
        db.set_property_override(aabb_obj, "min.y", Value::F32(20.0));
        db.set_property_override(aabb_obj, "min.z", Value::F32(30.0));

        db.set_property_override(aabb_obj, "max.x", Value::F32(40.0));
        db.set_property_override(aabb_obj, "max.y", Value::F32(50.0));
        db.set_property_override(aabb_obj, "max.z", Value::F32(60.0));


        let dyn_array_type = db.find_named_type("DynArrayNullableTest").unwrap().as_record().unwrap().clone();
        let dyn_array_obj = db.new_object(&dyn_array_type);

        let element1 = db.add_dynamic_array_override(dyn_array_obj, "dyn_array");
        let element2 = db.add_dynamic_array_override(dyn_array_obj, "dyn_array");

        db.set_override_behavior(dyn_array_obj, "dyn_array", OverrideBehavior::Replace);
        db.set_null_override(dyn_array_obj, format!("dyn_array.{}", element1), NullOverride::SetNonNull);
        db.set_property_override(dyn_array_obj, format!("dyn_array.{}.value", element1), Value::F32(10.0));

        let schema_cache = SchemaCacheSingleFile::store_string(&db);
        let data = DataStorageJsonSingleFile::store_string(&db);

        println!("Schema Cache: {}", schema_cache);
        println!("Data: {}", data);
        (schema_cache, data)
    };

    println!("---------------------");
    {
        let mut linker2 = SchemaLinker::default();
        linker2.add_source_dir(path, "*.json").unwrap();

        let mut db2 = Database::default();
        db2.add_linked_types(linker2);
        DataStorageJsonSingleFile::load_string(&mut db2, &data);
        let data2 = DataStorageJsonSingleFile::store_string(&db2);
        println!("Data2: {}", data2);
        assert_eq!(data, data2);

        let schema_cache2 = SchemaCacheSingleFile::store_string(&db2);
        assert_eq!(schema_cache, schema_cache2);
    }

    println!("---------------------");
    {
        let mut db3 = Database::default();
        SchemaCacheSingleFile::load_string(&mut db3, &schema_cache);

        DataStorageJsonSingleFile::load_string(&mut db3, &data);
        let data3 = DataStorageJsonSingleFile::store_string(&db3);
        println!("---------------------");
        println!("Data3: {}", data3);
        assert_eq!(data, data3);

        let schema_cache3 = SchemaCacheSingleFile::store_string(&db3);
        assert_eq!(schema_cache, schema_cache3);
    }

    // {
    //     let mut linker2 = SchemaCacheSingleFile::();
    //     linker2.add_source_dir(path, "*.json").unwrap();
    //
    //     let mut db2 = Database::default();
    //     db2.add_linked_types(linker2);
    //     DataStorageJsonSingleFile::load_string(&mut db2, &data);
    //     let data2 = DataStorageJsonSingleFile::store_string(&db2);
    //     println!("Data2: {}", data2);
    //     assert_eq!(data, data2);
    // }





    //SchemaCacheSingleFile::store(&db, PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/schema_cache/cache.json")));

    //DataStorageJsonSingleFile::store(&db, PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/data/database.json")));
    //let data = DataStorageJsonSingleFile::store_string(&db);
    // println!("Data: {}", data);
    //
    //
    // let mut linker2 = SchemaLinker::default();
    // linker2.add_source_dir(path, "*.json").unwrap();
    //
    // let mut db2 = Database::default();
    // db2.add_linked_types(linker2);
    // DataStorageJsonSingleFile::load_string(&mut db2, &data);
    // let data2 = DataStorageJsonSingleFile::store_string(&db2);
    // println!("Data2: {}", data2);
    // assert_eq!(data, data2);
    //
    //
    // println!("{}", env!("CARGO_MANIFEST_DIR"));
}

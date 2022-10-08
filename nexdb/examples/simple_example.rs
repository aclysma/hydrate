use log::MetadataBuilder;
use serde_json::Value as JsonValue;
use std::path::PathBuf;

use nexdb::*;

fn main() {
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    // let data = r#"
    // {
    //     "a": 3,
    //     "b": "x"
    // }
    // "#;
    //
    //
    // let v:JsonValue = serde_json::from_str(data).unwrap();
    // println!("value is {:?} {:?}", v["a"], v["b"]);
    //
    //
    // let mut db = Database::default();
    //
    // let vec3_schema_object = db.register_record_type("Vec3", |builder| {
    //     let x = builder.add_f32("x");
    //     x.add_field_alias("X");
    //     builder.add_f32("y");
    //     builder.add_f32("z");
    //
    // });
    //
    // let aabb_schema_object = db.register_record_type("AABB", |builder| {
    //     builder.add_struct("min", &vec3_schema_object);
    //     builder.add_struct("max", &vec3_schema_object);
    //     builder.add_dynamic_array("test_array", &Schema::Record(vec3_schema_object.clone()));
    // });
    //
    // let _yes_no_enum = db.register_enum_type("YesNo", |builder| {
    //     let yes = builder.add_symbol("Yes", 1);
    //     yes.add_symbol_alias("YES");
    //
    //     builder.add_symbol("No", 0);
    // });
    //
    // let _uuid_fixed = db.register_fixed_type("SomeFixedSizeField", 16, |_builder| {
    //
    // });

    let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/schema"));

    let mut linker = SchemaLinker::default();
    linker.add_source_dir(path, "*.json").unwrap();

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

    //SchemaCacheSingleFile::store(&db, PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/schema_cache/cache.json")));

    //DataStorageJsonSingleFile::store(&db, PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/data/database.json")));
    let data = DataStorageJsonSingleFile::store_string(&db);
    println!("Data: {}", data);
    DataStorageJsonSingleFile::load_string(&mut db, &data);


    println!("{}", env!("CARGO_MANIFEST_DIR"));
}

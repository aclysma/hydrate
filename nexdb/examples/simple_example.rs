use std::path::PathBuf;
use serde_json::Value as JsonValue;

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

    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/schema"
    ));

    let mut loader = SchemaLoader::default();
    loader.add_source_dir(path, "*.json").unwrap();
    loader.finish();

    println!("{}", env!("CARGO_MANIFEST_DIR"));

}

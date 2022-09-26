
use serde_json::Value as JsonValue;

use nexdb::*;

fn main() {

    let data = r#"
    {
        "a": 3,
        "b": "x"
    }
    "#;


    let v:JsonValue = serde_json::from_str(data).unwrap();
    println!("value is {:?} {:?}", v["a"], v["b"]);


    let mut db = Database::default();

    let vec3_schema_object = db.register_record_type("Vec3", |builder| {
        let x = builder.add_f32("x");
        x.add_alias("X");
        builder.add_f32("y");
        builder.add_f32("z");
    });

    let aabb_schema_object = db.register_record_type("AABB", |builder| {
        builder.add_struct("min", &vec3_schema_object);
        builder.add_struct("max", &vec3_schema_object);
    });

    let yes_no_enum = db.register_enum_type("YesNo", |builder| {
        let yes = builder.add_symbol("Yes", 1);
        yes.add_alias("YES");

        builder.add_symbol("No", 0);
    });

    println!("fingerprint {}", aabb_schema_object.fingerprint_uuid());

    let schema = db.find_schema_by_name("Vec3");
    println!("vec3 fingerprint {}", schema.unwrap().fingerprint_uuid());
}


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
        x.add_field_alias("X");
        builder.add_f32("y");
        builder.add_f32("z");
    });

    let aabb_schema_object = db.register_record_type("AABB", |builder| {
        builder.add_struct("min", &vec3_schema_object);
        builder.add_struct("max", &vec3_schema_object);
    });

    let _yes_no_enum = db.register_enum_type("YesNo", |builder| {
        let yes = builder.add_symbol("Yes", 1);
        yes.add_symbol_alias("YES");

        builder.add_symbol("No", 0);
    });

    let _uuid_fixed = db.register_fixed_type("SomeFixedSizeField", 16, |_builder| {

    });

    println!("fingerprint {}", aabb_schema_object.fingerprint_uuid());

    let schema = db.find_schema_by_name("Vec3").unwrap();
    println!("vec3 fingerprint {}", schema.fingerprint().as_uuid());


    let aabb1 = db.new_object(&aabb_schema_object);
    let aabb2 = db.new_object_from_prototype(aabb1);


    /*
    let aabb1_resolver = db.object_property_resolver(aabb1);
    let aabb2_resolver = db.object_property_resolver(aabb2);

    println!("aabb1.max.y = {}", aabb1_resolver.get_path_f32(&mut db, &["max", "y"]).unwrap());
    println!("aabb2.max.y = {}", aabb2_resolver.get_path_f32(&mut db, &["max", "y"]).unwrap());

    aabb1_resolver.set_path_f32(&mut db, &["max", "y"], 100.0).unwrap();

    println!("aabb1.max.y = {}", aabb1_resolver.get_path_f32(&mut db, &["max", "y"]).unwrap());
    println!("aabb2.max.y = {}", aabb2_resolver.get_path_f32(&mut db, &["max", "y"]).unwrap());

    aabb2_resolver.set_path_f32(&mut db, &["max", "y"], 200.0).unwrap();

    println!("aabb1.max.y = {}", aabb1_resolver.get_path_f32(&mut db, &["max", "y"]).unwrap());
    println!("aabb2.max.y = {}", aabb2_resolver.get_path_f32(&mut db, &["max", "y"]).unwrap());
    */





    //db







    //TODO: API for visiting structs within objects to set values?
    //let obj = db.create_object(&aabb_schema_object);
    //db.


}

// struct PropertyBrowser {
//     path: Vec<String>
// }
//
// impl PropertyBrowser {
//     fn get_f32(name: &str) -> f32 {
//         unimplemented!();
//     }
//
//     fn set_f32(name: &str) -> f32 {
//         unimplemented!();
//     }
//
//     fn get_struct(name: &str) -> PropertyBrowser {
//         unimplemented!();
//     }
// }

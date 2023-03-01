use std::ops::Add;
use std::path::PathBuf;
use hydrate::model::{DataSet, Schema, SchemaNamedType, SchemaRecord, SchemaRecordField, SchemaSet};

mod build_test;




fn main() {
    schema_to_rs();
}

fn schema_to_rs() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/schema"
    ));

    let mut linker = hydrate::model::SchemaLinker::default();
    linker.add_source_dir(&path, "**.json").unwrap();

    let mut schema_set = SchemaSet::default();
    schema_set.add_linked_types(linker).unwrap();

    for (fingerprint, named_type) in schema_set.schemas() {
        //println!("{:?} {:?}", fingerprint, named_type);

        match named_type {
            SchemaNamedType::Record(_) => {
                generate_struct_for_record(&schema_set, named_type.as_record().unwrap());
                //generate_impl_for_record(&schema_set, named_type.as_record().unwrap());
            },
            SchemaNamedType::Enum(_) => println!("enum"),//generate_code_for_record(schema_set, named_type.as_record().unwrap())
            SchemaNamedType::Fixed(_) => println!("fixed"),//generate_code_for_record(schema_set, named_type.as_record().unwrap())
        }
    }


    // What to generate?
    // - Typesafe wrappers around schema
    // - structs, getters, setters, load, store
    // - optionally a runtime version of struct with UUID and builder
    // - some types need to be referenced externally, some types need to be defined
    // - config file to drive where to put things?
    // - allow special flags/modifiers by path and/or by extra fields in schema?
    // - some may need defines, type uuids, etc.?
    // - some can go straight into final build, others get transformed by build step?

    // - rust STL
    // - base shared types (glam, typedefs, etc.)
    // - codegen for shaders
    // - codegen for schema types

    // - do we want to provide converters for a schema Vec3 to something like a glam::Vec3?

    //schema_set.



    // Struct with all fields
    // non-member methods to return and set individual values
    // methods to load full struct
    // methods to store full struct

}

/*
fn field_schema_to_rust_type(schema_set: &SchemaSet, field_schema: &Schema) -> String {
    match field_schema {
        Schema::Nullable(x) => format!("Option<{}>", field_schema_to_rust_type(schema_set, &*x)),
        Schema::Boolean => "bool".to_string(),
        Schema::I32 => "i32".to_string(),
        Schema::I64 => "i64".to_string(),
        Schema::U32 => "u32".to_string(),
        Schema::U64 => "u64".to_string(),
        Schema::F32 => "f32".to_string(),
        Schema::F64 => "f64".to_string(),
        Schema::Bytes => "Vec<u8>".to_string(),
        Schema::Buffer => "Vec<u8>".to_string(),
        Schema::String => "String".to_string(),
        Schema::StaticArray(x) => format!("[{}; {}]", field_schema_to_rust_type(schema_set, x.item_type()), x.length()),
        Schema::DynamicArray(x) => format!("Vec<{}>", field_schema_to_rust_type(schema_set, x.item_type())),
        Schema::Map(x) => format!("HashMap<{}, {}>", field_schema_to_rust_type(schema_set, x.key_type()), field_schema_to_rust_type(schema_set, x.value_type())),
        Schema::ObjectRef(x) => {
            //let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            //format!("Handle<{}>", inner_type.name())
            "ObjectId".to_string()
        }
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            format!("{}FromSchema", inner_type.name().to_string())
            // match inner_type {
            //     SchemaNamedType::Record(_) => {}
            //     SchemaNamedType::Enum(_) => {}
            //     SchemaNamedType::Fixed(_) => {}
            // }
        }
    }
}
*/

fn field_schema_to_field_type(schema_set: &SchemaSet, field_schema: &Schema) -> Option<String> {
    Some(match field_schema {
        Schema::Nullable(x) => format!("NullableField::<{}>", field_schema_to_field_type(schema_set, &*x)?),
        Schema::Boolean => "BooleanField".to_string(),
        Schema::I32 => "I32Field".to_string(),
        Schema::I64 => "I64Field".to_string(),
        Schema::U32 => "U32Field".to_string(),
        Schema::U64 => "U64Field".to_string(),
        Schema::F32 => "F32Field".to_string(),
        Schema::F64 => "F64Field".to_string(),
        Schema::Bytes => "BytesField".to_string(), //return None,//"Vec<u8>".to_string(),
        Schema::Buffer => unimplemented!(), //return None,//"Vec<u8>".to_string(),
        Schema::String => "StringField".to_string(),
        Schema::StaticArray(x) => unimplemented!(), //return None,//format!("[{}; {}]", field_schema_to_rust_type(schema_set, x.item_type()), x.length()),
        Schema::DynamicArray(x) => format!("DynamicArrayField::<{}>", field_schema_to_field_type(schema_set, x.item_type())?),//return None,//format!("Vec<{}>", field_schema_to_rust_type(schema_set, x.item_type())),
        Schema::Map(x) => unimplemented!(),// return None,//format!("HashMap<{}, {}>", field_schema_to_rust_type(schema_set, x.key_type()), field_schema_to_rust_type(schema_set, x.value_type())),
        Schema::ObjectRef(x) => "ObjectRefField".to_string(),
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            format!("{}Record", inner_type.name().to_string())
            // match inner_type {
            //     SchemaNamedType::Record(_) => {}
            //     SchemaNamedType::Enum(_) => {}
            //     SchemaNamedType::Fixed(_) => {}
            // }
        }
    })
}


fn generate_struct_for_record(schema_set: &SchemaSet, schema: &SchemaRecord) {
    // println!("struct {}Record {{", schema.name());
    // for field in schema.fields() {
    //     println!("    {}: {},", field.name(), field_schema_to_rust_type(schema_set, field.field_schema()));
    // }
    // println!("}}");

    let mut scope = codegen::Scope::new();

    let record_name = format!("{}Record", schema.name());
    let s = scope.new_struct(record_name.as_str()).tuple_field("PropertyPath");

    let new_impl = scope.new_impl(record_name.as_str()).impl_trait("Field");
    let new_fn = new_impl.new_fn("new").arg("property_path", "PropertyPath");
    new_fn.ret("Self");
    new_fn.line(format!("{}(property_path)", record_name));

    let mut main_impl = scope.new_impl(record_name.as_str());

    for field in schema.fields() {
        let field_type = field_schema_to_field_type(schema_set, field.field_schema());
        if let Some(field_type) = field_type {
            let mut f = codegen::Function::new(field.name());
            f.arg_ref_self();
            f.ret(&field_type);
            f.line(format!("{}::new(self.0.push(\"{}\"))", field_type, field.name()));
            main_impl.push_fn(f);
        }

    }

    println!("{}", scope.to_string());


}

// fn cast_value_to_type_fn_name(schema: &Schema) -> &str {
//     match schema {
//         Schema::Nullable(_) => unimplemented!(),
//         Schema::Boolean => "as_boolean",
//         Schema::I32 => "as_i32",
//         Schema::I64 => "as_i64",
//         Schema::U32 => "as_u32",
//         Schema::U64 => "as_u64",
//         Schema::F32 => "as_f32",
//         Schema::F64 => "as_f64",
//         Schema::Bytes => unimplemented!(),
//         Schema::Buffer => unimplemented!(),
//         Schema::String => unimplemented!(),
//         Schema::StaticArray(_) => unimplemented!(),
//         Schema::DynamicArray(_) => unimplemented!(),
//         Schema::Map(_) => unimplemented!(),
//         Schema::ObjectRef(_) => unimplemented!(),
//         Schema::NamedType(_) => unimplemented!(),
//     }
// }
/*
fn generate_resolve_property_call(schema_set: &SchemaSet, schema: &Schema, field_name: &str) -> String {
    match schema {
        Schema::Nullable(x) => {
            let mut lines = "".to_string();
            lines = lines.add(&format!("if !data_set_view.resolve_is_null(\"{}\").unwrap() {{\n", field_name));
            lines = lines.add(&format!("    data_set_view.push_property_path(\"{}\");\n", field_name));
            //println!("        Some(data_set_view.resolve_property("opt_bool.value").unwrap().as_boolean()?)", field.name(), x.name());
            //lines.generate_load_lines_for_field(schema_set, , field, property_path_prefix);
            lines = lines.add(&format!("    let value = {};\n", generate_resolve_property_call(schema_set, &*x, "value")));
            lines = lines.add("    data_set_view.pop_property_path();\n");
            lines = lines.add("    Some(value)\n");
            lines = lines.add("} else {\n");
            lines = lines.add("    None\n");
            lines = lines.add("}");
            lines
        }
        Schema::Boolean => format!("data_set_view.resolve_property(\"{}\").unwrap().as_boolean().unwrap()", field_name),
        Schema::I32 => format!("data_set_view.resolve_property(\"{}\").unwrap().as_i32().unwrap()", field_name),
        Schema::I64 => format!("data_set_view.resolve_property(\"{}\").unwrap().as_i64().unwrap()", field_name),
        Schema::U32 => format!("data_set_view.resolve_property(\"{}\").unwrap().as_u32().unwrap()", field_name),
        Schema::U64 => format!("data_set_view.resolve_property(\"{}\").unwrap().as_u64().unwrap()", field_name),
        Schema::F32 => format!("data_set_view.resolve_property(\"{}\").unwrap().as_f32().unwrap()", field_name),
        Schema::F64 => format!("data_set_view.resolve_property(\"{}\").unwrap().as_f64().unwrap()", field_name),
        Schema::Bytes => unimplemented!(),
        Schema::Buffer => unimplemented!(),
        Schema::String => format!("data_set_view.resolve_property(\"{}\").unwrap().as_string().unwrap()", field_name),
        Schema::StaticArray(_) => unimplemented!(),
        Schema::DynamicArray(_) => {
            let mut lines = "".to_string();
            lines = lines.add("{\n");

            // should it be a hash map?


            lines = lines.add("}");
            lines
        }
        Schema::Map(_) => unimplemented!(),
        Schema::ObjectRef(_) => format!("data_set_view.resolve_property(\"{}\").unwrap().as_object_ref().unwrap()", field_name),
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            match inner_type {
                SchemaNamedType::Record(x) => {
                    //generate_load_lines_for_record(schema_set, x, &format!("{}{}.", property_path_prefix, field.name()));
                    let mut lines = "".to_string();
                    lines = lines.add("{\n");
                    lines = lines.add(&format!("        data_set_view.push_property_path(\"{}\");\n", field_name));
                    lines = lines.add(&format!("        let value = {}FromSchema::load(data_set_view);\n", x.name()));
                    lines = lines.add("        data_set_view.pop_property_path();\n");
                    lines = lines.add("        value\n");

                    lines = lines.add("}");
                    lines
                }
                SchemaNamedType::Enum(_) => unimplemented!(),
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
    }
}

fn generate_load_lines_for_field(schema_set: &SchemaSet, schema: &SchemaRecord, field: &SchemaRecordField, property_path_prefix: &str) {
    let data_set = DataSet::default();
    //data_set.resolve_is_null()
    //data_set.resolve_property(schema_set, object_id, pro).unwrap().as_boolean().unwrap()
    // resolve all properties, including for child objects, not just fields...

    let property_prefix = format!("{}{}.", property_path_prefix, field.name());
    let property_path = &property_prefix[0..property_prefix.len() - 1];


    println!("        let {} = {};", field.name(), generate_resolve_property_call(schema_set, field.field_schema(), field.name()));


    // match field.field_schema() {
    //     Schema::Nullable(x) => {
    //         println!("let {} = if !data_set_view.resolve_is_null(\"{}\").unwrap() {{", field.name(), property_path);
    //         println!("        data_set_view.push_property_path(\"{}\");", field.name());
    //         //println!("        Some(data_set_view.resolve_property("opt_bool.value").unwrap().as_boolean()?)", field.name(), x.name());
    //         generate_load_lines_for_field(schema_set, , field, property_path_prefix);
    //         println!("        data_set_view.pop_property_path();");
    //         println!("}} else {{");
    //
    //         println!("}};");
    //     }
    //     Schema::Boolean => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_boolean().unwrap();", field.name(), property_path),
    //     Schema::I32 => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_i32().unwrap();", field.name(), property_path),
    //     Schema::I64 => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_i64().unwrap();", field.name(), property_path),
    //     Schema::U32 => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_u32().unwrap();", field.name(), property_path),
    //     Schema::U64 => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_u64().unwrap();", field.name(), property_path),
    //     Schema::F32 => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_f32().unwrap();", field.name(), property_path),
    //     Schema::F64 => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_f64().unwrap();", field.name(), property_path),
    //     Schema::Bytes => unimplemented!(),
    //     Schema::Buffer => unimplemented!(),
    //     Schema::String => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_string().unwrap();", field.name(), property_path),
    //     Schema::StaticArray(_) => unimplemented!(),
    //     Schema::DynamicArray(_) => unimplemented!(),
    //     Schema::Map(_) => unimplemented!(),
    //     Schema::ObjectRef(_) => println!("        let {} = data_set_view.resolve_property(\"{}\").unwrap().as_object_ref().unwrap();", field.name(), property_path),
    //     Schema::NamedType(x) => {
    //         let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
    //         match inner_type {
    //             SchemaNamedType::Record(x) => {
    //                 //generate_load_lines_for_record(schema_set, x, &format!("{}{}.", property_path_prefix, field.name()));
    //                 println!("        data_set_view.push_property_path(\"{}\");", field.name());
    //                 println!("        let {} = {}FromSchema::load(data_set_view);", field.name(), x.name());
    //                 println!("        data_set_view.pop_property_path();");
    //
    //             }
    //             SchemaNamedType::Enum(_) => unimplemented!(),
    //             SchemaNamedType::Fixed(_) => unimplemented!(),
    //         }
    //     }
    // }
}

fn generate_load_lines_for_record(schema_set: &SchemaSet, schema: &SchemaRecord, property_path_prefix: &str) {
    for field in schema.fields() {
        //println!("        // load {}{}", property_path_prefix, field.name());
        //println!("data_set.resolve_property(schema, object_id, \"position.x").unwrap().as_f32().unwrap(),
        //println!("");
        generate_load_lines_for_field(schema_set, schema, field, property_path_prefix);
    }

    println!("");

    println!("Self {{");
    for field in schema.fields() {
        println!("    {},", field.name());
    }
    println!("}}");
}

fn generate_load_fn_for_record(schema_set: &SchemaSet, schema: &SchemaRecord) {
    println!("    pub fn load(data_set_view: &mut DataSetView) -> Self {{");
    generate_load_lines_for_record(schema_set, schema, "");
    // for field in schema.fields() {
    //     println!("        // load {}", field.name());
    //     //println!("data_set.resolve_property(schema, object_id, \"position.x").unwrap().as_f32().unwrap(),
    //     generate_load_lines_for_field(schema_set, schema, field);
    // }
    println!("    }}");
}

fn generate_impl_for_record(schema_set: &SchemaSet, schema: &SchemaRecord) {
    println!("impl {}FromSchema {{", schema.name());
    generate_load_fn_for_record(schema_set, schema);
    println!("}}");
}
*/
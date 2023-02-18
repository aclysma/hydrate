use std::path::PathBuf;
use hydrate::model::{DataSet, Schema, SchemaNamedType, SchemaRecord, SchemaRecordField, SchemaSet};

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
                generate_impl_for_record(&schema_set, named_type.as_record().unwrap());
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
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            format!("Handle<{}>", inner_type.name())
        }
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            inner_type.name().to_string()
            // match inner_type {
            //     SchemaNamedType::Record(_) => {}
            //     SchemaNamedType::Enum(_) => {}
            //     SchemaNamedType::Fixed(_) => {}
            // }
        }
    }
}

fn generate_struct_for_record(schema_set: &SchemaSet, schema: &SchemaRecord) {
    println!("struct {} {{", schema.name());
    for field in schema.fields() {
        println!("    {}: {},", field.name(), field_schema_to_rust_type(schema_set, field.field_schema()));
    }
    println!("}}");
}

fn generate_load_lines_for_field(schema_set: &SchemaSet, schema: &SchemaRecord, field: &SchemaRecordField, property_path_prefix: &str) {
    let data_set = DataSet::default();
    //data_set.resolve_is_null()
    //data_set.resolve_property(schema_set, object_id, pro).unwrap().as_boolean().unwrap()
    // resolve all properties, including for child objects, not just fields...

    let property_prefix = format!("{}{}.", property_path_prefix, field.name());
    let property_path = &property_prefix[0..property_prefix.len() - 1];

    match field.field_schema() {
        Schema::Nullable(_) => {}
        Schema::Boolean => println!("let field_{} = data_set.resolve_property(schema_set, object_id, \"{}\").unwrap().as_boolean().unwrap();", field.name(), property_path),
        Schema::I32 => {}
        Schema::I64 => {}
        Schema::U32 => {}
        Schema::U64 => {}
        Schema::F32 => {}
        Schema::F64 => {}
        Schema::Bytes => {}
        Schema::Buffer => {}
        Schema::String => {}
        Schema::StaticArray(_) => {}
        Schema::DynamicArray(_) => {}
        Schema::Map(_) => {}
        Schema::ObjectRef(_) => {}
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();
            match inner_type {
                SchemaNamedType::Record(x) => {
                    generate_load_lines_for_record(schema_set, x, &format!("{}{}.", property_path_prefix, field.name()));
                }
                SchemaNamedType::Enum(_) => {}
                SchemaNamedType::Fixed(_) => {}
            }
        }
    }
}

fn generate_load_lines_for_record(schema_set: &SchemaSet, schema: &SchemaRecord, property_path_prefix: &str) {
    for field in schema.fields() {
        println!("        // load {}{}", property_path_prefix, field.name());
        //println!("data_set.resolve_property(schema, object_id, \"position.x").unwrap().as_f32().unwrap(),
        println!("");
        generate_load_lines_for_field(schema_set, schema, field, property_path_prefix);
    }
}

fn generate_load_fn_for_record(schema_set: &SchemaSet, schema: &SchemaRecord) {
    println!("    load(object_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Self {{");
    generate_load_lines_for_record(schema_set, schema, "");
    // for field in schema.fields() {
    //     println!("        // load {}", field.name());
    //     //println!("data_set.resolve_property(schema, object_id, \"position.x").unwrap().as_f32().unwrap(),
    //     generate_load_lines_for_field(schema_set, schema, field);
    // }
    println!("    }}");
}

fn generate_impl_for_record(schema_set: &SchemaSet, schema: &SchemaRecord) {
    println!("impl {} {{", schema.name());
    generate_load_fn_for_record(schema_set, schema);
    println!("}}");
}

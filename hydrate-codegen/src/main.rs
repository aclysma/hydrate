use std::ops::Add;
use std::path::PathBuf;
use hydrate::model::{DataSet, Schema, SchemaEnum, SchemaNamedType, SchemaRecord, SchemaRecordField, SchemaSet};

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

        let scope = match named_type {
            SchemaNamedType::Record(x) => generate_record(&schema_set, x),
            SchemaNamedType::Enum(x) => generate_enum(&schema_set, x),
            SchemaNamedType::Fixed(_) => unimplemented!(),//generate_code_for_record(schema_set, named_type.as_record().unwrap())
        };

        println!("{}", scope.to_string());
    }
}


fn generate_enum(schema_set: &SchemaSet, schema: &SchemaEnum) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let enum_name = format!("{}Enum", schema.name());
    let mut enumeration = scope.new_enum(&enum_name);
    for symbol in schema.symbols() {
        enumeration.push_variant(codegen::Variant::new(symbol.name()));
    }

    let mut enum_impl = scope.new_impl(&enum_name).impl_trait("Enum");
    let mut to_symbol_name_fn = enum_impl.new_fn("to_symbol_name");
    to_symbol_name_fn.arg_ref_self().ret("&'static str");
    to_symbol_name_fn.line("match self {");
    for symbol in schema.symbols() {
        to_symbol_name_fn.line(format!("    {}::{} => \"{}\",", enum_name, symbol.name(), symbol.name()));
    }
    to_symbol_name_fn.line("}");

    let mut from_symbol_name_fn = enum_impl.new_fn("from_symbol_name");
    from_symbol_name_fn.arg("str", "&str").ret(format!("Option<{}>", &enum_name));
    from_symbol_name_fn.line("match str {");
    for symbol in schema.symbols() {
        from_symbol_name_fn.line(format!("    \"{}\" => Some({}::{}),", symbol.name(), enum_name, symbol.name()));
    }
    from_symbol_name_fn.line("    _ => None,");
    from_symbol_name_fn.line("}");

    scope
}

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

            match inner_type {
                SchemaNamedType::Record(_) => format!("{}Record", inner_type.name().to_string()),
                SchemaNamedType::Enum(_) => format!("EnumField::<{}Enum>", inner_type.name().to_string()),
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
    })
}


fn generate_record(schema_set: &SchemaSet, schema: &SchemaRecord) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let record_name = format!("{}Record", schema.name());
    let s = scope.new_struct(record_name.as_str()).tuple_field("PropertyPath");
    s.derive("Default");

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

    scope
}

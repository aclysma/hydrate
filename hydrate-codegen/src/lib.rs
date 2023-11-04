use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use hydrate_model::{Schema, SchemaEnum, SchemaNamedType, SchemaRecord, SchemaSet};

//
// TODO: Validation code - we should have a fn on generated types to verify they are registered in the schema and match
// TODO: Optionally also generate code to register them as new schema types
// TODO: Could cache a ref to a linked schema
//

#[derive(StructOpt, Debug)]
pub struct HydrateCodegenArgs {
    #[structopt(name = "schema-path", long, parse(from_os_str))]
    pub schema_path: PathBuf,
    #[structopt(name = "included-schema", long, parse(from_os_str))]
    pub included_schema: Vec<PathBuf>,
    #[structopt(name = "outfile", long, parse(from_os_str))]
    pub outfile: PathBuf,
    #[structopt(name = "trace", long)]
    pub trace: bool,
}


pub fn run(args: &HydrateCodegenArgs) -> Result<(), Box<dyn Error>> {
    schema_to_rs(&args.schema_path, &args.included_schema, &args.outfile)
}


fn schema_to_rs(
    schema_path: &Path,
    referenced_schema_paths: &[PathBuf],
    outfile: &Path,
) -> Result<(), Box<dyn Error>> {

    let mut linker = hydrate_model::SchemaLinker::default();
    linker.add_source_dir(&schema_path, "**.json").unwrap();

    let named_types_to_build = linker.unlinked_type_names();

    for referenced_schema_path in referenced_schema_paths {
        linker.add_source_dir(referenced_schema_path, "**.json").unwrap();
    }

    let mut schema_set = SchemaSet::default();
    schema_set.add_linked_types(linker).unwrap();

    let mut all_schemas_to_build = Vec::default();
    for named_type_to_build in named_types_to_build {
        let named_type = schema_set.find_named_type(named_type_to_build).unwrap();
        all_schemas_to_build.push((named_type.fingerprint(), named_type));
    }

    // Sort by name so we have a deterministic output ordering for codegen
    all_schemas_to_build.sort_by(|lhs, rhs| lhs.1.name().cmp(rhs.1.name()));

    let mut code_fragments_as_string = Vec::default();

    for (fingerprint, named_type) in all_schemas_to_build {
        //println!("{:?} {:?}", fingerprint, named_type);

        let scope = match named_type {
            SchemaNamedType::Record(x) => generate_record(&schema_set, x),
            SchemaNamedType::Enum(x) => generate_enum(&schema_set, x),
            SchemaNamedType::Fixed(_) => unimplemented!(),//generate_code_for_record(schema_set, named_type.as_record().unwrap())
        };

        let code_fragment_as_string = scope.to_string();
        println!("{}", code_fragment_as_string);
        code_fragments_as_string.push(code_fragment_as_string);
    }

    //let write_path = PathBuf::from("out_codegen.rs");
    let f = std::fs::File::create(outfile)?;
    let mut writer = std::io::BufWriter::new(f);
    writeln!(writer, "// This file generated automatically by hydrate-codegen. Do not make manual edits. Use include!() to place these types in the intended location.")?;
    for code_fragment in code_fragments_as_string {
        writeln!(writer, "{}", &code_fragment)?;
    }

    writer.flush()?;
    Ok(())
}


fn generate_enum(schema_set: &SchemaSet, schema: &SchemaEnum) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let enum_name = format!("{}Enum", schema.name());
    let mut enumeration = scope.new_enum(&enum_name);
    enumeration.vis("pub");
    enumeration.derive("Copy");
    enumeration.derive("Clone");
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
        for alias in symbol.aliases() {
            from_symbol_name_fn.line(format!("    \"{}\" => Some({}::{}),", alias, enum_name, symbol.name()));
        }
    }
    from_symbol_name_fn.line("    _ => None,");
    from_symbol_name_fn.line("}");

    let mut main_impl = scope.new_impl(enum_name.as_str());
    let mut schema_name_fn = main_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.vis("pub");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    scope
}

/*
    fn from_symbol_name(str: &str, schema_set: &SchemaSet) -> Option<MeshAdvShadowMethodEnum> {
        let enum_schema = schema_set.find_named_type("MeshAdvShadowMethod").unwrap().as_enum().unwrap();
        for symbol in enum_schema.symbols() {
            let symbol_matches = str == symbol.name() || symbol.aliases().iter().any(|x| x.as_str() == str);
            if symbol_matches {

                if let Some(enum_value) = match symbol.name() {
                    "None" => Some(MeshAdvShadowMethodEnum::None),
                    "Opaque" => Some(MeshAdvShadowMethodEnum::Opaque),
                    _ => None,
                } {
                    return Some(enum_value)
                }

                for alias in symbol.aliases() {
                    if let Some(enum_value) = match alias.as_str() {
                        "None" => Some(MeshAdvShadowMethodEnum::None),
                        "Opaque" => Some(MeshAdvShadowMethodEnum::Opaque),
                        _ => None,
                    } {
                        return Some(enum_value)
                    }
                }
            }
        }

        None
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
    s.vis("pub");
    s.derive("Default");

    let field_impl = scope.new_impl(record_name.as_str()).impl_trait("Field");
    let new_fn = field_impl.new_fn("new").arg("property_path", "PropertyPath");
    new_fn.ret("Self");
    new_fn.line(format!("{}(property_path)", record_name));

    let record_impl = scope.new_impl(record_name.as_str()).impl_trait("Record");
    let mut schema_name_fn = record_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    let mut main_impl = scope.new_impl(record_name.as_str());
    for field in schema.fields() {
        let field_type = field_schema_to_field_type(schema_set, field.field_schema());
        if let Some(field_type) = field_type {
            let mut field_access_fn = main_impl.new_fn(field.name());
            field_access_fn.arg_ref_self();
            field_access_fn.ret(&field_type);
            field_access_fn.vis("pub");
            field_access_fn.line(format!("{}::new(self.0.push(\"{}\"))", field_type, field.name()));
        }
    }

    scope
}

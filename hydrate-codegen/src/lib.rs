use hydrate_model::{
    Schema, SchemaEnum, SchemaNamedType, SchemaRecord, SchemaSet, SchemaSetBuilder,
};
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

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
    linker
        .add_source_dir(&schema_path, "**.json")
        .map_err(|x| Box::new(x))?;

    let named_types_to_build = linker.unlinked_type_names();

    for referenced_schema_path in referenced_schema_paths {
        linker
            .add_source_dir(referenced_schema_path, "**.json")
            .map_err(|x| Box::new(x))?;
    }

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder
        .add_linked_types(linker)
        .map_err(|x| Box::new(x))?;
    let schema_set = schema_set_builder.build();

    let mut all_schemas_to_build = Vec::default();
    for named_type_to_build in named_types_to_build {
        let named_type = schema_set
            .find_named_type(named_type_to_build)
            .expect("Cannot find linked type in built schema");
        all_schemas_to_build.push((named_type.fingerprint(), named_type));
    }

    // Sort by name so we have a deterministic output ordering for codegen
    all_schemas_to_build.sort_by(|lhs, rhs| lhs.1.name().cmp(rhs.1.name()));

    let mut code_fragments_as_string = Vec::default();

    for (_fingerprint, named_type) in all_schemas_to_build {
        //println!("{:?} {:?}", fingerprint, named_type);

        let scopes = match named_type {
            SchemaNamedType::Record(x) => vec![
                generate_accessor(&schema_set, x),
                generate_reader(&schema_set, x),
                generate_writer(&schema_set, x),
                generate_owned(&schema_set, x),
            ],
            SchemaNamedType::Enum(x) => vec![generate_enum(&schema_set, x)],
            SchemaNamedType::Fixed(_) => unimplemented!(),
        };

        for scope in scopes {
            let code_fragment_as_string = scope.to_string();
            println!("{}\n", code_fragment_as_string);
            code_fragments_as_string.push(code_fragment_as_string);
        }
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

fn generate_enum(
    _schema_set: &SchemaSet,
    schema: &SchemaEnum,
) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let enum_name = format!("{}Enum", schema.name());
    let enumeration = scope.new_enum(&enum_name);
    enumeration.vis("pub");
    enumeration.derive("Copy");
    enumeration.derive("Clone");
    for symbol in schema.symbols() {
        enumeration.push_variant(codegen::Variant::new(symbol.name()));
    }

    let enum_impl = scope.new_impl(&enum_name).impl_trait("Enum");

    let to_symbol_name_fn = enum_impl.new_fn("to_symbol_name");
    to_symbol_name_fn.arg_ref_self().ret("&'static str");
    to_symbol_name_fn.line("match self {");
    for symbol in schema.symbols() {
        to_symbol_name_fn.line(format!(
            "    {}::{} => \"{}\",",
            enum_name,
            symbol.name(),
            symbol.name()
        ));
    }
    to_symbol_name_fn.line("}");

    let from_symbol_name_fn = enum_impl.new_fn("from_symbol_name");
    from_symbol_name_fn
        .arg("str", "&str")
        .ret(format!("Option<{}>", &enum_name));
    from_symbol_name_fn.line("match str {");
    for symbol in schema.symbols() {
        from_symbol_name_fn.line(format!(
            "    \"{}\" => Some({}::{}),",
            symbol.name(),
            enum_name,
            symbol.name()
        ));
        for alias in symbol.aliases() {
            from_symbol_name_fn.line(format!(
                "    \"{}\" => Some({}::{}),",
                alias,
                enum_name,
                symbol.name()
            ));
        }
    }
    from_symbol_name_fn.line("    _ => None,");
    from_symbol_name_fn.line("}");

    let main_impl = scope.new_impl(enum_name.as_str());
    let schema_name_fn = main_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.vis("pub");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    scope
}

fn field_schema_to_field_type(
    schema_set: &SchemaSet,
    field_schema: &Schema,
) -> Option<String> {
    Some(match field_schema {
        Schema::Nullable(x) => format!(
            "NullableFieldAccessor::<{}>",
            field_schema_to_field_type(schema_set, &*x)?
        ),
        Schema::Boolean => "BooleanFieldAccessor".to_string(),
        Schema::I32 => "I32FieldAccessor".to_string(),
        Schema::I64 => "I64FieldAccessor".to_string(),
        Schema::U32 => "U32FieldAccessor".to_string(),
        Schema::U64 => "U64FieldAccessor".to_string(),
        Schema::F32 => "F32FieldAccessor".to_string(),
        Schema::F64 => "F64FieldAccessor".to_string(),
        Schema::Bytes => "BytesFieldAccessor".to_string(), //return None,//"Vec<u8>".to_string(),
        Schema::String => "StringFieldAccessor".to_string(),
        Schema::StaticArray(_x) => unimplemented!(), //return None,//format!("[{}; {}]", field_schema_to_rust_type(schema_set, x.item_type()), x.length()),
        Schema::DynamicArray(x) => format!(
            "DynamicArrayFieldAccessor::<{}>",
            field_schema_to_field_type(schema_set, x.item_type())?
        ), //return None,//format!("Vec<{}>", field_schema_to_rust_type(schema_set, x.item_type())),
        Schema::Map(_x) => unimplemented!(), // return None,//format!("HashMap<{}, {}>", field_schema_to_rust_type(schema_set, x.key_type()), field_schema_to_rust_type(schema_set, x.value_type())),
        Schema::AssetRef(_x) => "AssetRefFieldAccessor".to_string(),
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();

            match inner_type {
                SchemaNamedType::Record(_) => format!("{}Accessor", inner_type.name().to_string()),
                SchemaNamedType::Enum(_) => {
                    format!("EnumFieldAccessor::<{}Enum>", inner_type.name().to_string())
                }
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
    })
}

fn generate_accessor(
    schema_set: &SchemaSet,
    schema: &SchemaRecord,
) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let accessor_name = format!("{}Accessor", schema.name());
    let s = scope
        .new_struct(accessor_name.as_str())
        .tuple_field("PropertyPath");
    s.vis("pub");
    s.derive("Default");

    let field_impl = scope
        .new_impl(accessor_name.as_str())
        .impl_trait("FieldAccessor");
    let new_fn = field_impl
        .new_fn("new")
        .arg("property_path", "PropertyPath");
    new_fn.ret("Self");
    new_fn.line(format!("{}(property_path)", accessor_name));

    let accessor_impl = scope
        .new_impl(accessor_name.as_str())
        .impl_trait("RecordAccessor");
    let schema_name_fn = accessor_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    let main_impl = scope.new_impl(accessor_name.as_str());
    for field in schema.fields() {
        let field_type = field_schema_to_field_type(schema_set, field.field_schema());
        if let Some(field_type) = field_type {
            let field_access_fn = main_impl.new_fn(field.name());
            field_access_fn.arg_ref_self();
            field_access_fn.ret(&field_type);
            field_access_fn.vis("pub");
            field_access_fn.line(format!(
                "{}::new(self.0.push(\"{}\"))",
                field_type,
                field.name()
            ));
        }
    }

    scope
}

fn field_schema_to_reader_type(
    schema_set: &SchemaSet,
    field_schema: &Schema,
) -> Option<String> {
    Some(match field_schema {
        Schema::Nullable(x) => format!(
            "NullableFieldReader::<{}>",
            field_schema_to_reader_type(schema_set, &*x)?
        ),
        Schema::Boolean => "BooleanFieldReader".to_string(),
        Schema::I32 => "I32FieldReader".to_string(),
        Schema::I64 => "I64FieldReader".to_string(),
        Schema::U32 => "U32FieldReader".to_string(),
        Schema::U64 => "U64FieldReader".to_string(),
        Schema::F32 => "F32FieldReader".to_string(),
        Schema::F64 => "F64FieldReader".to_string(),
        Schema::Bytes => "BytesFieldReader".to_string(), //return None,//"Vec<u8>".to_string(),
        Schema::String => "StringFieldReader".to_string(),
        Schema::StaticArray(_x) => unimplemented!(), //return None,//format!("[{}; {}]", field_schema_to_rust_type(schema_set, x.item_type()), x.length()),
        Schema::DynamicArray(x) => format!(
            "DynamicArrayFieldReader::<{}>",
            field_schema_to_reader_type(schema_set, x.item_type())?
        ), //return None,//format!("Vec<{}>", field_schema_to_rust_type(schema_set, x.item_type())),
        Schema::Map(_x) => unimplemented!(), // return None,//format!("HashMap<{}, {}>", field_schema_to_rust_type(schema_set, x.key_type()), field_schema_to_rust_type(schema_set, x.value_type())),
        Schema::AssetRef(_x) => "AssetRefFieldReader".to_string(),
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();

            match inner_type {
                SchemaNamedType::Record(_) => format!("{}Reader", inner_type.name().to_string()),
                SchemaNamedType::Enum(_) => {
                    format!("EnumFieldReader::<{}Enum>", inner_type.name().to_string())
                }
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
    })
}

fn generate_reader(
    schema_set: &SchemaSet,
    schema: &SchemaRecord,
) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let record_name = format!("{}Reader<'a>", schema.name());
    let record_name_without_generic = format!("{}Reader", schema.name());
    let s = scope
        .new_struct(record_name.as_str())
        .tuple_field("PropertyPath")
        .tuple_field("DataContainerRef<'a>");
    s.vis("pub");

    let field_impl = scope
        .new_impl(record_name.as_str())
        .generic("'a")
        .impl_trait("FieldReader<'a>");
    let new_fn = field_impl
        .new_fn("new")
        .arg("property_path", "PropertyPath")
        .arg("data_container", "DataContainerRef<'a>");
    new_fn.ret("Self");
    new_fn.line(format!(
        "{}(property_path, data_container)",
        record_name_without_generic
    ));

    let record_impl = scope
        .new_impl(record_name.as_str())
        .generic("'a")
        .impl_trait("RecordReader");
    let schema_name_fn = record_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    let main_impl = scope.new_impl(record_name.as_str()).generic("'a");
    for field in schema.fields() {
        let field_type = field_schema_to_reader_type(schema_set, field.field_schema());
        if let Some(field_type) = field_type {
            let field_access_fn = main_impl.new_fn(field.name());
            field_access_fn.arg_ref_self();
            field_access_fn.ret(&field_type);
            field_access_fn.vis("pub");
            field_access_fn.line(format!(
                "{}::new(self.0.push(\"{}\"), self.1)",
                field_type,
                field.name()
            ));
        }
    }

    scope
}

fn field_schema_to_writer_type(
    schema_set: &SchemaSet,
    field_schema: &Schema,
) -> Option<String> {
    Some(match field_schema {
        Schema::Nullable(x) => format!(
            "NullableFieldWriter::<{}>",
            field_schema_to_writer_type(schema_set, &*x)?
        ),
        Schema::Boolean => "BooleanFieldWriter".to_string(),
        Schema::I32 => "I32FieldWriter".to_string(),
        Schema::I64 => "I64FieldWriter".to_string(),
        Schema::U32 => "U32FieldWriter".to_string(),
        Schema::U64 => "U64FieldWriter".to_string(),
        Schema::F32 => "F32FieldWriter".to_string(),
        Schema::F64 => "F64FieldWriter".to_string(),
        Schema::Bytes => "BytesFieldWriter".to_string(), //return None,//"Vec<u8>".to_string(),
        Schema::String => "StringFieldWriter".to_string(),
        Schema::StaticArray(_x) => unimplemented!(), //return None,//format!("[{}; {}]", field_schema_to_rust_type(schema_set, x.item_type()), x.length()),
        Schema::DynamicArray(x) => format!(
            "DynamicArrayFieldWriter::<{}>",
            field_schema_to_writer_type(schema_set, x.item_type())?
        ), //return None,//format!("Vec<{}>", field_schema_to_rust_type(schema_set, x.item_type())),
        Schema::Map(_x) => unimplemented!(), // return None,//format!("HashMap<{}, {}>", field_schema_to_rust_type(schema_set, x.key_type()), field_schema_to_rust_type(schema_set, x.value_type())),
        Schema::AssetRef(_x) => "AssetRefFieldWriter".to_string(),
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();

            match inner_type {
                SchemaNamedType::Record(_) => format!("{}Writer", inner_type.name().to_string()),
                SchemaNamedType::Enum(_) => {
                    format!("EnumFieldWriter::<{}Enum>", inner_type.name().to_string())
                }
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
    })
}

fn generate_writer(
    schema_set: &SchemaSet,
    schema: &SchemaRecord,
) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let record_name = format!("{}Writer<'a>", schema.name());
    let record_name_without_generic = format!("{}Writer", schema.name());
    let s = scope
        .new_struct(record_name.as_str())
        .tuple_field("PropertyPath")
        .tuple_field("Rc<RefCell<DataContainerRefMut<'a>>>");
    s.vis("pub");

    let field_impl = scope
        .new_impl(record_name.as_str())
        .generic("'a")
        .impl_trait("FieldWriter<'a>");
    let new_fn = field_impl
        .new_fn("new")
        .arg("property_path", "PropertyPath")
        .arg("data_container", "&Rc<RefCell<DataContainerRefMut<'a>>>");
    new_fn.ret("Self");
    new_fn.line(format!(
        "{}(property_path, data_container.clone())",
        record_name_without_generic
    ));

    let record_impl = scope
        .new_impl(record_name.as_str())
        .generic("'a")
        .impl_trait("RecordWriter");
    let schema_name_fn = record_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    let main_impl = scope.new_impl(record_name.as_str()).generic("'a");
    for field in schema.fields() {
        let field_type = field_schema_to_writer_type(schema_set, field.field_schema());
        if let Some(field_type) = field_type {
            let field_access_fn = main_impl.new_fn(field.name());
            //field_access_fn.arg_ref_self();
            field_access_fn.arg("self", "&'a Self");
            field_access_fn.ret(&field_type);
            field_access_fn.vis("pub");
            field_access_fn.line(format!(
                "{}::new(self.0.push(\"{}\"), &self.1)",
                field_type,
                field.name()
            ));
        }
    }

    scope
}

fn field_schema_to_owned_type(
    schema_set: &SchemaSet,
    field_schema: &Schema,
) -> Option<String> {
    Some(match field_schema {
        Schema::Nullable(x) => format!(
            "NullableFieldOwned::<{}>",
            field_schema_to_owned_type(schema_set, &*x)?
        ),
        Schema::Boolean => "BooleanFieldOwned".to_string(),
        Schema::I32 => "I32FieldOwned".to_string(),
        Schema::I64 => "I64FieldOwned".to_string(),
        Schema::U32 => "U32FieldOwned".to_string(),
        Schema::U64 => "U64FieldOwned".to_string(),
        Schema::F32 => "F32FieldOwned".to_string(),
        Schema::F64 => "F64FieldOwned".to_string(),
        Schema::Bytes => "BytesFieldOwned".to_string(), //return None,//"Vec<u8>".to_string(),
        Schema::String => "StringFieldOwned".to_string(),
        Schema::StaticArray(_x) => unimplemented!(), //return None,//format!("[{}; {}]", field_schema_to_rust_type(schema_set, x.item_type()), x.length()),
        Schema::DynamicArray(x) => format!(
            "DynamicArrayFieldOwned::<{}>",
            field_schema_to_owned_type(schema_set, x.item_type())?
        ), //return None,//format!("Vec<{}>", field_schema_to_rust_type(schema_set, x.item_type())),
        Schema::Map(_x) => unimplemented!(), // return None,//format!("HashMap<{}, {}>", field_schema_to_rust_type(schema_set, x.key_type()), field_schema_to_rust_type(schema_set, x.value_type())),
        Schema::AssetRef(_x) => "AssetRefFieldOwned".to_string(),
        Schema::NamedType(x) => {
            let inner_type = schema_set.find_named_type_by_fingerprint(*x).unwrap();

            match inner_type {
                SchemaNamedType::Record(_) => format!("{}Owned", inner_type.name().to_string()),
                SchemaNamedType::Enum(_) => {
                    format!("EnumFieldOwned::<{}Enum>", inner_type.name().to_string())
                }
                SchemaNamedType::Fixed(_) => unimplemented!(),
            }
        }
    })
}

fn generate_owned(
    schema_set: &SchemaSet,
    schema: &SchemaRecord,
) -> codegen::Scope {
    let mut scope = codegen::Scope::new();

    let record_name = format!("{}Owned", schema.name());
    let record_name_without_generic = format!("{}Owned", schema.name());
    let s = scope
        .new_struct(record_name.as_str())
        .tuple_field("PropertyPath")
        .tuple_field("Rc<RefCell<Option<DataContainerOwned>>>");
    s.vis("pub");

    let field_impl = scope
        .new_impl(record_name.as_str())
        .impl_trait("FieldOwned");
    let new_fn = field_impl
        .new_fn("new")
        .arg("property_path", "PropertyPath")
        .arg("data_container", "&Rc<RefCell<Option<DataContainerOwned>>>");
    new_fn.ret("Self");
    new_fn.line(format!(
        "{}(property_path, data_container.clone())",
        record_name_without_generic
    ));

    let record_impl = scope
        .new_impl(record_name.as_str())
        .impl_trait("RecordOwned");
    let schema_name_fn = record_impl.new_fn("schema_name");
    schema_name_fn.ret("&'static str");
    schema_name_fn.line(format!("\"{}\"", schema.name()));

    let main_impl = scope.new_impl(record_name.as_str());
    for field in schema.fields() {
        let field_type = field_schema_to_owned_type(schema_set, field.field_schema());
        if let Some(field_type) = field_type {
            let field_access_fn = main_impl.new_fn(field.name());
            //field_access_fn.arg_ref_self();
            field_access_fn.arg("self", "&Self");
            field_access_fn.ret(&field_type);
            field_access_fn.vis("pub");
            field_access_fn.line(format!(
                "{}::new(self.0.push(\"{}\"), &self.1)",
                field_type,
                field.name()
            ));
        }
    }

    scope
}

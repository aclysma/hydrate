use super::*;

fn parse_json_schema_type_ref(
    json_value: &serde_json::Value,
    error_prefix: &str,
) -> SchemaDefParserResult<SchemaDefType> {
    let name;
    match json_value {
        serde_json::Value::String(type_name) => {
            name = type_name.as_str();
        }
        serde_json::Value::Object(json_object) => {
            name = json_object
                .get("name")
                .map(|x| x.as_str())
                .flatten()
                .ok_or_else(|| {
                    SchemaDefParserError::String(format!(
                        "{}Record field types must have a name",
                        error_prefix
                    ))
                })?;
        }
        _ => {
            return Err(SchemaDefParserError::String(format!(
                "{}Type references must be a string or json object",
                error_prefix
            )))
        }
    }

    Ok(match name {
        "nullable" => {
            let inner_type = json_value.get("inner_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All nullable types must has an inner_type",
                    error_prefix
                ))
            })?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;

            SchemaDefType::Nullable(Box::new(inner_type))
        }
        "bool" => SchemaDefType::Boolean,
        "i32" => SchemaDefType::I32,
        "i64" => SchemaDefType::I64,
        "u32" => SchemaDefType::U32,
        "u64" => SchemaDefType::U64,
        "f32" => SchemaDefType::F32,
        "f64" => SchemaDefType::F64,
        "bytes" => SchemaDefType::Bytes,
        "buffer" => SchemaDefType::Buffer,
        "string" => SchemaDefType::String,
        "static_array" => {
            unimplemented!()
        }
        "dynamic_array" => {
            let inner_type = json_value.get("inner_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All dynamic_array types must has an inner_type",
                    error_prefix
                ))
            })?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;

            SchemaDefType::DynamicArray(SchemaDefDynamicArray {
                item_type: Box::new(inner_type),
            })
        }
        "map" => {
            unimplemented!()
        }
        "object_ref" => {
            let inner_type = json_value.get("inner_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All object_ref types must has an inner_type",
                    error_prefix
                ))
            })?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;
            match inner_type {
                SchemaDefType::NamedType(x) => SchemaDefType::ObjectRef(x),
                _ => {
                    Err(SchemaDefParserError::String(format!(
                        "{}All object_ref types must has an inner_type that is not a built-in type",
                        error_prefix
                    )))?;

                    // We will error above
                    unreachable!();
                }
            }
        }
        // StaticArray(SchemaDefStaticArray),
        // DynamicArray(SchemaDefDynamicArray),
        // Map(SchemaDefMap),
        // RecordRef(String),
        // Record(SchemaDefRecord),
        // Enum(SchemaDefEnum),
        // Fixed(SchemaDefFixed),
        _ => SchemaDefType::NamedType(name.to_string()),
    })
}

// fn parse_alias_list(json_value: &serde_json::Value, error_prefix: &str) -> SchemaDefParserResult<Vec<String>> {
//     let values = json_value.as_array().ok_or_else(|| SchemaDefParserError::String(format!("{}Aliases must be an array of strings", error_prefix)))?;
//
//     let mut strings = Vec::with_capacity(values.len());
//     for value in values {
//         strings.push(value.as_str().ok_or_else(|| SchemaDefParserError::String(format!("{}Aliases must be an array of strings", error_prefix)))?.to_string());
//     }
//
//     Ok(strings)
// }

fn parse_json_schema_def_record_field(
    json_object: &serde_json::Value,
    error_prefix: &str,
) -> SchemaDefParserResult<SchemaDefRecordField> {
    let object = json_object.as_object().ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Record schema fields must be a json object",
            error_prefix
        ))
    })?;

    let field_name = object
        .get("name")
        .map(|x| x.as_str())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Record fields must be a name that is a string",
                error_prefix
            ))
        })?
        .to_string();
    let json_aliases = object.get("aliases").map(|x| x.as_array()).flatten();
    let mut aliases = vec![];
    if let Some(json_aliases) = json_aliases {
        for json_alias in json_aliases {
            aliases.push(
                json_alias
                    .as_str()
                    .ok_or_else(|| {
                        SchemaDefParserError::String(format!(
                            "{}Fields's aliases must be strings",
                            error_prefix
                        ))
                    })?
                    .to_string(),
            )
        }
    }
    // let aliases = if let Some(json_aliases) = object.get("aliases") {
    //     Self::parse_alias_list(json_aliases, error_prefix)?
    // } else {
    //     vec![]
    // };
    let error_prefix = format!("{}[Field {}]", error_prefix, field_name);
    //let field_schema = object.get("type").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaDefParserError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();
    //let field_schema = object.get("type").ok_or_else(|| SchemaDefParserError::Str("Schema file record fields must have a type of string or json object"))?;
    let field_type_json_value = object.get("type").ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Record fields must have a type of string or json object",
            error_prefix
        ))
    })?;
    let field_type = parse_json_schema_type_ref(field_type_json_value, &error_prefix)?;

    Ok(SchemaDefRecordField {
        field_name,
        aliases,
        field_type,
    })
}

fn parse_json_schema_def_record(
    json_object: &serde_json::Map<String, serde_json::Value>,
    error_prefix: &str,
) -> SchemaDefParserResult<SchemaDefRecord> {
    let name = json_object.get("name").ok_or_else(|| {
        SchemaDefParserError::String(format!("{}Records must have a name", error_prefix))
    })?;
    let name_str = name.as_str().ok_or_else(|| {
        SchemaDefParserError::String(format!("{}Records must have a name", error_prefix))
    })?;

    let error_prefix = format!("{}[Record {}]", error_prefix, name_str);
    log::debug!("Parsing record named '{}'", name_str);

    let json_aliases = json_object.get("aliases").map(|x| x.as_array()).flatten();
    let mut aliases = vec![];
    if let Some(json_aliases) = json_aliases {
        for json_alias in json_aliases {
            aliases.push(
                json_alias
                    .as_str()
                    .ok_or_else(|| {
                        SchemaDefParserError::String(format!(
                            "{}Record's aliases must be strings",
                            error_prefix
                        ))
                    })?
                    .to_string(),
            )
        }
    }

    let json_fields = json_object
        .get("fields")
        .map(|x| x.as_array())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Records must have an array of fields",
                error_prefix
            ))
        })?;
    let mut fields = vec![];
    for json_field in json_fields {
        fields.push(parse_json_schema_def_record_field(
            json_field,
            &error_prefix,
        )?);
    }

    Ok(SchemaDefRecord {
        type_name: name_str.to_string(),
        aliases,
        fields,
    })
}

fn parse_json_schema_def_enum_symbol(
    json_object: &serde_json::Value,
    error_prefix: &str,
) -> SchemaDefParserResult<SchemaDefEnumSymbol> {
    let object = json_object.as_object().ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Enum schema symbols must be a json object",
            error_prefix
        ))
    })?;

    let symbol_name = object
        .get("name")
        .map(|x| x.as_str())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Record symbols must be a name that is a string",
                error_prefix
            ))
        })?
        .to_string();
    let json_aliases = object.get("aliases").map(|x| x.as_array()).flatten();
    let mut aliases = vec![];
    if let Some(json_aliases) = json_aliases {
        for json_alias in json_aliases {
            aliases.push(
                json_alias
                    .as_str()
                    .ok_or_else(|| {
                        SchemaDefParserError::String(format!(
                            "{}Fields's aliases must be strings",
                            error_prefix
                        ))
                    })?
                    .to_string(),
            )
        }
    }
    // let aliases = if let Some(json_aliases) = object.get("aliases") {
    //     Self::parse_alias_list(json_aliases, error_prefix)?
    // } else {
    //     vec![]
    // };
    //let error_prefix = format!("{}[Field {}]", error_prefix, symbol_name);

    Ok(SchemaDefEnumSymbol {
        symbol_name,
        aliases,
        //field_type,
    })
}

fn parse_json_schema_def_enum(
    json_object: &serde_json::Map<String, serde_json::Value>,
    error_prefix: &str,
) -> SchemaDefParserResult<SchemaDefEnum> {
    let name = json_object.get("name").ok_or_else(|| {
        SchemaDefParserError::String(format!("{}Enums must have a name", error_prefix))
    })?;
    let name_str = name.as_str().ok_or_else(|| {
        SchemaDefParserError::String(format!("{}Enums must have a name", error_prefix))
    })?;

    let error_prefix = format!("{}[Enum {}]", error_prefix, name_str);
    log::debug!("Parsing enum named '{}'", name_str);

    let json_aliases = json_object.get("aliases").map(|x| x.as_array()).flatten();
    let mut aliases = vec![];
    if let Some(json_aliases) = json_aliases {
        for json_alias in json_aliases {
            aliases.push(
                json_alias
                    .as_str()
                    .ok_or_else(|| {
                        SchemaDefParserError::String(format!(
                            "{}Enum's aliases must be strings",
                            error_prefix
                        ))
                    })?
                    .to_string(),
            )
        }
    }

    let json_symbols = json_object
        .get("symbols")
        .map(|x| x.as_array())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Records must have an array of fields",
                error_prefix
            ))
        })?;
    let mut symbols = vec![];
    for json_symbol in json_symbols {
        symbols.push(parse_json_schema_def_enum_symbol(
            json_symbol,
            &error_prefix,
        )?);
    }

    Ok(SchemaDefEnum {
        type_name: name_str.to_string(),
        aliases,
        symbols,
    })
}

pub(super) fn parse_json_schema_def(
    json_value: &serde_json::Value,
    error_prefix: &str,
) -> SchemaDefParserResult<SchemaDefNamedType> {
    let object = json_value.as_object().ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Schema file must be an array of json objects",
            error_prefix
        ))
    })?;

    let object_type = object.get("type").ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Schema file objects must have a type field",
            error_prefix
        ))
    })?;
    let object_type_str = object_type.as_str().ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Schema file objects must have a type field that is a string",
            error_prefix
        ))
    })?;
    match object_type_str {
        "record" => {
            let record = parse_json_schema_def_record(object, error_prefix)?;
            Ok(SchemaDefNamedType::Record(record))
        },
        "enum" => {
            let enumeration = parse_json_schema_def_enum(object, error_prefix)?;
            Ok(SchemaDefNamedType::Enum(enumeration))
        }
        _ => Err(SchemaDefParserError::String(format!(
            "Schema file object has a type field that is unrecognized {:?}",
            object_type_str
        ))),
    }
}

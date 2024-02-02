use std::path::Path;
use uuid::Uuid;
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
        "string" => SchemaDefType::String,
        "static_array" => {
            let inner_type = json_value.get("inner_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All static_array types must has an inner_type",
                    error_prefix
                ))
            })?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;

            let length = json_value
                .get("length")
                .ok_or_else(|| {
                    SchemaDefParserError::String(format!(
                    "{}All static_array types must has a length with a non-negative whole number",
                    error_prefix
                ))
                })?
                .as_u64()
                .ok_or_else(|| {
                    SchemaDefParserError::String(format!(
                    "{}All static_array types must has a length with a non-negative whole number",
                    error_prefix
                ))
                })?;

            SchemaDefType::StaticArray(SchemaDefStaticArray {
                item_type: Box::new(inner_type),
                length: length as usize,
            })
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
            let key_type = json_value.get("key_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All dynamic_array types must has an key_type",
                    error_prefix
                ))
            })?;
            let key_type = parse_json_schema_type_ref(key_type, error_prefix)?;
            match &key_type {
                SchemaDefType::Boolean
                | SchemaDefType::I32
                | SchemaDefType::I64
                | SchemaDefType::U32
                | SchemaDefType::U64
                | SchemaDefType::String
                | SchemaDefType::AssetRef(_) => {
                    // value types are fine other than float
                    Ok(())
                }
                SchemaDefType::NamedType(_) => {
                    // must be enum, but we have to catch that when we link the schemas
                    Ok(())
                }
                _ => {
                    // No floats
                    // No containers
                    // No nullables
                    // Bytes is unbounded in size, bad idea
                    Err(SchemaDefParserError::String(format!(
                        "{}Any map key_type must be a boolean, i32, i64, u32, u64, string, asset reference, or enum",
                        error_prefix
                    )))
                }
            }?;

            let value_type = json_value.get("value_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All dynamic_array types must has an value_type",
                    error_prefix
                ))
            })?;
            let value_type = parse_json_schema_type_ref(value_type, error_prefix)?;

            SchemaDefType::Map(SchemaDefMap {
                key_type: Box::new(key_type),
                value_type: Box::new(value_type),
            })
        }
        "asset_ref" => {
            let inner_type = json_value.get("inner_type").ok_or_else(|| {
                SchemaDefParserError::String(format!(
                    "{}All asset_ref types must has an inner_type",
                    error_prefix
                ))
            })?;
            let inner_type = parse_json_schema_type_ref(inner_type, error_prefix)?;
            match inner_type {
                SchemaDefType::NamedType(x) => SchemaDefType::AssetRef(x),
                _ => {
                    Err(SchemaDefParserError::String(format!(
                        "{}All asset_ref types must has an inner_type that is not a built-in type",
                        error_prefix
                    )))?;

                    // We will error above
                    unreachable!();
                }
            }
        }
        // StaticArray(SchemaDefStaticArray),
        // Map(SchemaDefMap),
        // Fixed(SchemaDefFixed),
        _ => SchemaDefType::NamedType(name.to_string()),
    })
}

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

    let field_uuid = object
        .get("uuid")
        .map(|x| x.as_str().map(|y| Uuid::parse_str(y).ok()).flatten())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Field uuids must be a UUID",
                error_prefix
            ))
        })?;

    let field_name = object
        .get("name")
        .map(|x| x.as_str())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Field names must be a name that is a string",
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
    // let aliases = if let Some(json_aliases) = asset.get("aliases") {
    //     Self::parse_alias_list(json_aliases, error_prefix)?
    // } else {
    //     vec![]
    // };
    let error_prefix = format!("{}[Field {}]", error_prefix, field_name);
    //let field_schema = asset.get("type").map(|x| x.as_str()).flatten().ok_or_else(|| SchemaDefParserError::Str("Schema file record schema fields must be a name that is a string"))?.to_string();
    //let field_schema = asset.get("type").ok_or_else(|| SchemaDefParserError::Str("Schema file record fields must have a type of string or json object"))?;
    let field_type_json_value = object.get("type").ok_or_else(|| {
        SchemaDefParserError::String(format!(
            "{}Fields must have a type of string or json object",
            error_prefix
        ))
    })?;
    let field_type = parse_json_schema_type_ref(field_type_json_value, &error_prefix)?;
    let mut markup = SchemaDefRecordFieldMarkup::default();

    if let Some(display_name) = object.get("display_name") {
        markup.display_name = Some(
            display_name
                .as_str()
                .ok_or_else(|| {
                    SchemaDefParserError::String("display_name must be a string".to_string())
                })?
                .to_string(),
        );
    }

    if let Some(category) = object.get("category") {
        markup.category = Some(
            category
                .as_str()
                .ok_or_else(|| {
                    SchemaDefParserError::String("category must be a string".to_string())
                })?
                .to_string(),
        );
    }

    if let Some(description) = object.get("description") {
        markup.description = Some(
            description
                .as_str()
                .ok_or_else(|| {
                    SchemaDefParserError::String("description must be a string".to_string())
                })?
                .to_string(),
        );
    }

    if let Some(ui_min) = object.get("ui_min") {
        markup.ui_min =
            Some(ui_min.as_f64().ok_or_else(|| {
                SchemaDefParserError::String("ui_min must be a number".to_string())
            })?);
    }

    if let Some(ui_max) = object.get("ui_max") {
        markup.ui_max =
            Some(ui_max.as_f64().ok_or_else(|| {
                SchemaDefParserError::String("ui_max must be a number".to_string())
            })?);
    }

    if let Some(clamp_min) = object.get("clamp_min") {
        markup.clamp_min = Some(clamp_min.as_f64().ok_or_else(|| {
            SchemaDefParserError::String("clamp_min must be a number".to_string())
        })?);
    }

    if let Some(clamp_max) = object.get("clamp_max") {
        markup.clamp_max = Some(clamp_max.as_f64().ok_or_else(|| {
            SchemaDefParserError::String("clamp_max must be a number".to_string())
        })?);
    }

    if markup.clamp_min.unwrap_or(f64::MIN) > markup.ui_min.unwrap_or(f64::MIN) {
        Err(SchemaDefParserError::String(
            "clamp_min must be <= ui_min".to_string(),
        ))?
    }

    if markup.clamp_max.unwrap_or(f64::MAX) < markup.ui_max.unwrap_or(f64::MAX) {
        Err(SchemaDefParserError::String(
            "clamp_max must be >= ui_max".to_string(),
        ))?
    }

    if markup.ui_min.unwrap_or(f64::MIN) > markup.ui_max.unwrap_or(f64::MAX) {
        Err(SchemaDefParserError::String(
            "ui_min must be <= ui_max".to_string(),
        ))?
    }

    if markup.clamp_min.unwrap_or(f64::MIN) > markup.clamp_max.unwrap_or(f64::MAX) {
        Err(SchemaDefParserError::String(
            "clamp_min must be <= clamp_max".to_string(),
        ))?
    }

    Ok(SchemaDefRecordField {
        field_name,
        field_uuid,
        aliases,
        field_type,
        markup,
    })
}

fn parse_json_schema_def_record(
    json_object: &serde_json::Map<String, serde_json::Value>,
    error_prefix: &str,
    json_file_absolute_path: &Path,
) -> SchemaDefParserResult<SchemaDefRecord> {
    let name = json_object.get("name").ok_or_else(|| {
        SchemaDefParserError::String(format!("{}Records must have a name", error_prefix))
    })?;
    let name_str = name.as_str().ok_or_else(|| {
        SchemaDefParserError::String(format!("{}Records must have a name", error_prefix))
    })?;

    let error_prefix = format!("{}[Record {}]", error_prefix, name_str);
    log::trace!("Parsing record named '{}'", name_str);

    let type_uuid = json_object
        .get("uuid")
        .map(|x| x.as_str().map(|y| Uuid::parse_str(y).ok()).flatten())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Record type uuid must be a UUID",
                error_prefix
            ))
        })?;

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

    let mut markup = SchemaDefRecordMarkup::default();

    if let Some(display_name) = json_object.get("display_name") {
        markup.display_name = Some(
            display_name
                .as_str()
                .ok_or_else(|| {
                    SchemaDefParserError::String("display_name must be a string".to_string())
                })?
                .to_string(),
        );
    }

    if let Some(default_thumbnail) = json_object.get("default_thumbnail") {
        let default_thumbnail_str = Some(
            default_thumbnail
                .as_str()
                .ok_or_else(|| {
                    SchemaDefParserError::String("default_thumbnail must be a string".to_string())
                })?
                .to_string(),
        );

        if let Some(default_thumbnail_str) = default_thumbnail_str {
            let default_thumbnail_path = Path::new(&default_thumbnail_str);
            markup.default_thumbnail = Some(if default_thumbnail_path.is_relative() {
                json_file_absolute_path.parent().unwrap().join(default_thumbnail_path)
            } else {
                default_thumbnail_path.to_path_buf()
            });
        }
    }

    if let Some(tags) = json_object.get("tags") {
        let tags = tags.as_array().ok_or_else(|| {
            SchemaDefParserError::String("tags must be an array of strings".to_string())
        })?;
        for tag in tags {
            markup.tags.insert(
                tag.as_str()
                    .ok_or_else(|| {
                        SchemaDefParserError::String("tags must be an array of strings".to_string())
                    })?
                    .to_string(),
            );
        }
    }

    Ok(SchemaDefRecord {
        type_name: name_str.to_string(),
        type_uuid,
        aliases,
        fields,
        markup,
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
                "{}Enum symbols must be a name that is a string",
                error_prefix
            ))
        })?
        .to_string();

    let symbol_uuid = json_object
        .get("uuid")
        .map(|x| x.as_str().map(|y| Uuid::parse_str(y).ok()).flatten())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Enum symbol uuid must be a UUID",
                error_prefix
            ))
        })?;

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
    // let aliases = if let Some(json_aliases) = asset.get("aliases") {
    //     Self::parse_alias_list(json_aliases, error_prefix)?
    // } else {
    //     vec![]
    // };
    //let error_prefix = format!("{}[Field {}]", error_prefix, symbol_name);

    Ok(SchemaDefEnumSymbol {
        symbol_name,
        aliases,
        symbol_uuid,
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
    log::trace!("Parsing enum named '{}'", name_str);

    let type_uuid = json_object
        .get("uuid")
        .map(|x| x.as_str().map(|y| Uuid::parse_str(y).ok()).flatten())
        .flatten()
        .ok_or_else(|| {
            SchemaDefParserError::String(format!(
                "{}Enum type uuid must be a UUID",
                error_prefix
            ))
        })?;

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
        type_uuid,
        aliases,
        symbols,
    })
}

pub(super) fn parse_json_schema_def(
    json_value: &serde_json::Value,
    error_prefix: &str,
    json_file_absolute_path: &Path,
) -> SchemaDefParserResult<SchemaDefNamedType> {
    assert!(json_file_absolute_path.is_absolute());

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
            let record = parse_json_schema_def_record(object, error_prefix, json_file_absolute_path)?;
            Ok(SchemaDefNamedType::Record(record))
        }
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

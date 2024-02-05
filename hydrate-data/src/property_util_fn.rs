use crate::{
    DataSetError, DataSetResult, HashMap, Schema, SchemaFingerprint, SchemaNamedType, SchemaRecord,
};

pub(super) fn truncate_property_path(
    path: impl AsRef<str>,
    max_segment_count: usize,
) -> String {
    let mut shortened_path = String::default();
    //TODO: Escape map keys (and probably avoid path strings anyways)
    let split_path = path.as_ref().split(".");
    for (i, path_segment) in split_path.enumerate() {
        if i > max_segment_count {
            break;
        }

        if i == 0 {
            shortened_path = path_segment.to_string();
        } else {
            shortened_path = format!("{}.{}", shortened_path, path_segment);
        }
    }

    shortened_path
}

pub(super) fn property_schema_and_path_ancestors_to_check<'a>(
    named_type: &'a SchemaRecord,
    path: impl AsRef<str>,
    named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    accessed_nullable_keys: &mut Vec<String>,
    accessed_dynamic_array_keys: &mut Vec<(String, String)>,
    accessed_static_array_keys: &mut Vec<(String, String)>,
    accessed_map_keys: &mut Vec<(String, String)>,
) -> DataSetResult<Schema> {
    let mut schema = Schema::Record(named_type.fingerprint());

    //TODO: Escape map keys (and probably avoid path strings anyways)
    let split_path: Vec<_> = path.as_ref().split(".").collect();

    for (i, path_segment) in split_path[0..split_path.len() - 1].iter().enumerate() {
        // If failing to find the schema, check that code is querying a property that actually exists
        let child_schema = schema
            .find_field_schema(path_segment, named_types)
            .ok_or(DataSetError::SchemaNotFound)?;

        match schema {
            Schema::Nullable(_) => {
                accessed_nullable_keys.push(truncate_property_path(path.as_ref(), i - 1));
            }
            Schema::StaticArray(_) => {
                accessed_static_array_keys.push((
                    truncate_property_path(path.as_ref(), i - 1),
                    path_segment.to_string(),
                ));
            }
            Schema::DynamicArray(_) => {
                accessed_dynamic_array_keys.push((
                    truncate_property_path(path.as_ref(), i - 1),
                    path_segment.to_string(),
                ));
            }
            Schema::Map(_) => {
                accessed_map_keys.push((
                    truncate_property_path(path.as_ref(), i - 1),
                    path_segment.to_string(),
                ));
            }
            _ => {}
        }

        schema = child_schema.clone();
    }

    if let Some(last_path_segment) = split_path.last() {
        schema = schema
            .find_field_schema(last_path_segment, named_types)
            .ok_or(DataSetError::SchemaNotFound)?
            .clone();
    }

    Ok(schema)
}

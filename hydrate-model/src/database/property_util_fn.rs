use crate::{HashMap, Schema, SchemaFingerprint, SchemaNamedType, SchemaRecord};

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
    nullable_ancestors: &mut Vec<String>,
    dynamic_array_ancestors: &mut Vec<String>,
    map_ancestors: &mut Vec<String>,
    accessed_dynamic_array_keys: &mut Vec<(String, String)>,
) -> Option<Schema> {
    let mut schema = Schema::NamedType(named_type.fingerprint());

    //TODO: Escape map keys (and probably avoid path strings anyways)
    let split_path: Vec<_> = path.as_ref().split(".").collect();

    let mut parent_is_dynamic_array = false;

    for (i, path_segment) in split_path[0..split_path.len() - 1].iter().enumerate() {
        //.as_ref().split(".").enumerate() {
        let s = schema.find_field_schema(path_segment, named_types)?;
        //println!("  next schema {:?}", s);

        // current path needs to be verified as existing
        if parent_is_dynamic_array {
            accessed_dynamic_array_keys.push((
                super::truncate_property_path(path.as_ref(), i - 1),
                path_segment.to_string(),
            ));
        }

        parent_is_dynamic_array = false;

        //if let Some(s) = s {
        // If it's nullable, we need to check for value being null before looking up the prototype chain
        // If it's a map or dynamic array, we need to check for append mode before looking up the prototype chain
        match s {
            Schema::Nullable(_) => {
                let shortened_path = super::truncate_property_path(path.as_ref(), i);
                nullable_ancestors.push(shortened_path);
            }
            Schema::DynamicArray(_) => {
                let shortened_path = super::truncate_property_path(path.as_ref(), i);
                dynamic_array_ancestors.push(shortened_path.clone());

                parent_is_dynamic_array = true;
            }
            Schema::Map(_) => {
                let shortened_path = super::truncate_property_path(path.as_ref(), i);
                map_ancestors.push(shortened_path);
            }
            _ => {}
        }

        schema = s.clone();
        //} else {
        //    return None;
        //}
    }

    if let Some(last_path_segment) = split_path.last() {
        schema = schema
            .find_field_schema(last_path_segment, named_types)?
            .clone();
    }

    Some(schema)
}
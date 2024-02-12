mod dynamic_array;
pub use dynamic_array::*;

mod r#enum;
pub use r#enum::*;

//mod interface;
//pub use interface::*;

mod map;
pub use map::*;

mod record;
pub use record::*;

//mod ref_constraint;
//pub use ref_constraint::*;

mod static_array;
pub use static_array::*;

use crate::{DataSetError, DataSetResult, HashMap};
use crate::{HashSet, PropertyPath, SchemaFingerprint};
use std::hash::Hash;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SchemaId(u128);

#[derive(Clone, Debug)]
pub enum SchemaNamedType {
    Record(SchemaRecord),
    Enum(SchemaEnum),
}

impl SchemaNamedType {
    pub fn fingerprint(&self) -> SchemaFingerprint {
        match self {
            SchemaNamedType::Record(x) => x.fingerprint(),
            SchemaNamedType::Enum(x) => x.fingerprint(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SchemaNamedType::Record(x) => x.name(),
            SchemaNamedType::Enum(x) => x.name(),
        }
    }

    pub fn type_uuid(&self) -> Uuid {
        match self {
            SchemaNamedType::Record(x) => x.type_uuid(),
            SchemaNamedType::Enum(x) => x.type_uuid(),
        }
    }

    pub fn as_record(&self) -> DataSetResult<&SchemaRecord> {
        Ok(self.try_as_record().ok_or(DataSetError::InvalidSchema)?)
    }

    pub fn try_as_record(&self) -> Option<&SchemaRecord> {
        match self {
            SchemaNamedType::Record(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> DataSetResult<&SchemaEnum> {
        Ok(self.try_as_enum().ok_or(DataSetError::InvalidSchema)?)
    }

    pub fn try_as_enum(&self) -> Option<&SchemaEnum> {
        match self {
            SchemaNamedType::Enum(x) => Some(x),
            _ => None,
        }
    }

    // How migration works:
    // - Just about everything is stored in property paths like control_point.position.x
    // - The asset has some root named type (and it is a record)
    // - We iteratively walk through the property path, verifying that the target schema is the
    //   same type UUID, and any record field's types are interchangable (see Schema::types_are_interchangeable)

    pub fn find_post_migration_property_path(
        old_root_named_type: &SchemaNamedType,
        old_path: impl AsRef<str>,
        old_named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        new_root_named_type: &SchemaNamedType,
        new_named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        new_named_types_by_uuid: &HashMap<Uuid, SchemaFingerprint>,
    ) -> Option<String> {
        let mut old_schema = Schema::Record(old_root_named_type.fingerprint());
        let mut new_schema = Schema::Record(new_root_named_type.fingerprint());

        log::trace!("migrate property name {:?}", old_path.as_ref());
        let old_split_path = old_path.as_ref().split(".");
        let mut new_path = PropertyPath::default();

        // Iterate the path segments to find

        for old_path_segment in old_split_path {
            let new_path_segment = Schema::find_post_migration_field_name(
                &old_schema,
                old_path_segment,
                old_named_types,
                &new_schema,
                new_named_types,
                new_named_types_by_uuid,
            )?;

            new_path = new_path.push(&new_path_segment);
            let old_s = old_schema.find_field_schema(old_path_segment, old_named_types);
            let new_s = new_schema.find_field_schema(new_path_segment, new_named_types);

            if let (Some(old_s), Some(new_s)) = (old_s, new_s) {
                if !Schema::types_are_interchangeable(
                    old_s,
                    new_s,
                    old_named_types,
                    new_named_types,
                ) {
                    return None;
                }

                old_schema = old_s.clone();
                new_schema = new_s.clone();
            } else {
                return None;
            }
        }

        Some(new_path.path().to_string())
    }

    pub fn find_property_schema(
        &self,
        path: impl AsRef<str>,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> Option<Schema> {
        let mut schema = Schema::Record(self.fingerprint());

        let split_path = path.as_ref().split(".");

        // Iterate the path segments to find
        for path_segment in split_path {
            let s = schema.find_field_schema(path_segment, named_types);
            if let Some(s) = s {
                schema = s.clone();
            } else {
                return None;
            }
        }

        Some(schema)
    }
}

/// Describes format of data, either a single primitive value or complex layout comprised of
/// potentially many values
#[derive(Clone, Debug, PartialEq)]
pub enum Schema {
    /// Marks the field as possible to be null
    Nullable(Box<Schema>),
    Boolean,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    /// Variable amount of bytes stored within the asset
    Bytes,
    /// Variable-length UTF-8 String
    String,
    /// Fixed-size array of values
    StaticArray(SchemaStaticArray),
    DynamicArray(SchemaDynamicArray),
    Map(SchemaMap),
    AssetRef(SchemaFingerprint),
    /// Named type, it could be an enum, record, etc.
    Record(SchemaFingerprint),
    Enum(SchemaFingerprint),
}

impl Schema {
    pub fn is_nullable(&self) -> bool {
        match self {
            Schema::Nullable(_) => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Schema::Boolean => true,
            _ => false,
        }
    }

    pub fn is_i32(&self) -> bool {
        match self {
            Schema::I32 => true,
            _ => false,
        }
    }

    pub fn is_i64(&self) -> bool {
        match self {
            Schema::I64 => true,
            _ => false,
        }
    }

    pub fn is_u32(&self) -> bool {
        match self {
            Schema::U32 => true,
            _ => false,
        }
    }

    pub fn is_u64(&self) -> bool {
        match self {
            Schema::U64 => true,
            _ => false,
        }
    }

    pub fn is_f32(&self) -> bool {
        match self {
            Schema::F32 => true,
            _ => false,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self {
            Schema::F64 => true,
            _ => false,
        }
    }

    pub fn is_bytes(&self) -> bool {
        match self {
            Schema::Bytes => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Schema::String => true,
            _ => false,
        }
    }

    pub fn is_static_array(&self) -> bool {
        match self {
            Schema::StaticArray(_) => true,
            _ => false,
        }
    }

    pub fn is_dynamic_array(&self) -> bool {
        match self {
            Schema::DynamicArray(_) => true,
            _ => false,
        }
    }

    pub fn is_map(&self) -> bool {
        match self {
            Schema::Map(_) => true,
            _ => false,
        }
    }

    pub fn is_asset_ref(&self) -> bool {
        match self {
            Schema::AssetRef(_) => true,
            _ => false,
        }
    }

    pub fn is_record(&self) -> bool {
        match self {
            Schema::Record(_) => true,
            _ => false,
        }
    }

    pub fn is_enum(&self) -> bool {
        match self {
            Schema::Enum(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Schema::I32 | Schema::I64 | Schema::U32 | Schema::U64 | Schema::F32 | Schema::F64 => {
                true
            }
            _ => false,
        }
    }

    pub fn types_are_interchangeable(
        old_parent_schema: &Schema,
        new_parent_schema: &Schema,
        old_named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        new_named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> bool {
        // Covers strings/bytes
        if old_parent_schema == new_parent_schema {
            return true;
        }

        if old_parent_schema.is_number() && new_parent_schema.is_number() {
            return true;
        }

        match old_parent_schema {
            Schema::Nullable(old_inner) => {
                //TODO: Would be nice if we could handle nullable being added/removed on existing properties
                if let Schema::Nullable(new_inner) = new_parent_schema {
                    Self::types_are_interchangeable(
                        &*old_inner,
                        &*new_inner,
                        old_named_types,
                        new_named_types,
                    )
                } else {
                    false
                }
            }
            Schema::StaticArray(old_inner) => {
                if let Schema::StaticArray(new_inner) = new_parent_schema {
                    Self::types_are_interchangeable(
                        old_inner.item_type(),
                        new_inner.item_type(),
                        old_named_types,
                        new_named_types,
                    )
                } else {
                    false
                }
            }
            Schema::DynamicArray(old_inner) => {
                if let Schema::DynamicArray(new_inner) = new_parent_schema {
                    Self::types_are_interchangeable(
                        old_inner.item_type(),
                        new_inner.item_type(),
                        old_named_types,
                        new_named_types,
                    )
                } else {
                    false
                }
            }
            Schema::Map(old_inner) => {
                if let Schema::Map(new_inner) = new_parent_schema {
                    let keys_are_interchangage = Self::types_are_interchangeable(
                        old_inner.key_type(),
                        new_inner.key_type(),
                        old_named_types,
                        new_named_types,
                    );
                    let values_are_interchangable = Self::types_are_interchangeable(
                        old_inner.value_type(),
                        new_inner.value_type(),
                        old_named_types,
                        new_named_types,
                    );
                    keys_are_interchangage && values_are_interchangable
                } else {
                    false
                }
            }
            Schema::AssetRef(_) => {
                if let Schema::AssetRef(_) = new_parent_schema {
                    // won't enforce any type constraints here, we can leave that for schema validation
                    // later, which allows users to fix any problems
                    true
                } else {
                    false
                }
            }
            Schema::Record(old_inner) => {
                if let Schema::Record(new_inner) = new_parent_schema {
                    let old_named_type = old_named_types.get(old_inner).unwrap();
                    let new_named_type = new_named_types.get(new_inner).unwrap();

                    // TODO: Could see support for specific type transformations in the future
                    old_named_type.type_uuid() == new_named_type.type_uuid()
                } else {
                    false
                }
            }
            Schema::Enum(old_inner) => {
                if let Schema::Enum(new_inner) = new_parent_schema {
                    let old_named_type = old_named_types.get(old_inner).unwrap();
                    let new_named_type = new_named_types.get(new_inner).unwrap();

                    old_named_type.type_uuid() == new_named_type.type_uuid()
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    // This looks for equivalent field name in new types as existed in old types
    pub fn find_post_migration_field_name<'a>(
        old_parent_schema: &Schema,
        old_property_name: &'a str,
        old_named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        _new_parent_schema: &Schema,
        new_named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        new_named_types_by_uuid: &HashMap<Uuid, SchemaFingerprint>,
    ) -> Option<String> {
        match old_parent_schema {
            Schema::Nullable(_) => {
                if old_property_name == "value" {
                    Some(old_property_name.to_string())
                } else {
                    None
                }
            }
            Schema::Record(old_schema_fingerprint) => {
                let old_named_type = old_named_types.get(old_schema_fingerprint).unwrap();
                let old_schema_record = old_named_type.as_record().unwrap();
                let old_field = old_schema_record
                    .find_field_from_name(old_property_name.as_ref())
                    .unwrap();
                let old_record_type_uuid = old_named_type.type_uuid();

                // This is just finding the field with same UUID. No validation here that the schemas
                // are the same.
                let new_schema_fingerprint =
                    new_named_types_by_uuid.get(&old_record_type_uuid).unwrap();
                let new_named_type = new_named_types.get(new_schema_fingerprint).unwrap();
                let new_schema_record = new_named_type.as_record().unwrap();

                // This may fail to find the new field, in which case the field is probably removed
                new_schema_record
                    .find_field_from_field_uuid(old_field.field_uuid())
                    .map(|x| x.name().to_string())
            }
            Schema::StaticArray(_) => {
                if old_property_name.parse::<u32>().is_ok() {
                    Some(old_property_name.to_string())
                } else {
                    None
                }
            }
            Schema::DynamicArray(_) => {
                // We could validate that name is a valid UUID
                Uuid::from_str(old_property_name.as_ref()).ok()?;
                Some(old_property_name.to_string())
            }
            Schema::Map(_) => {
                if old_property_name.ends_with(":key") {
                    Uuid::from_str(&old_property_name[0..old_property_name.len() - 4]).ok()?;
                    Some(old_property_name.to_string())
                } else if old_property_name.ends_with(":value") {
                    Uuid::from_str(&old_property_name[0..old_property_name.len() - 6]).ok()?;
                    Some(old_property_name.to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // This looks for direct descendent field with given name
    pub fn find_field_schema<'a>(
        &'a self,
        name: impl AsRef<str>,
        named_types: &'a HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> Option<&'a Schema> {
        match self {
            Schema::Nullable(x) => {
                if name.as_ref() == "value" {
                    Some(&*x)
                } else {
                    // "null_value" special property name is purposefully omitted here
                    None
                }
            }
            Schema::Record(named_type_id) => {
                let named_type = named_types.get(named_type_id).unwrap();
                match named_type {
                    SchemaNamedType::Record(x) => x.field_schema(name),
                    SchemaNamedType::Enum(_) => None,
                }
            }
            Schema::StaticArray(x) => {
                if name.as_ref().parse::<u32>().is_ok() {
                    Some(x.item_type())
                } else {
                    None
                }
            }
            Schema::DynamicArray(x) => {
                // "replace" special property name is purposefully omitted here
                // We could validate that name is a valid UUID
                Uuid::from_str(name.as_ref()).ok()?;
                Some(x.item_type())
            }
            Schema::Map(x) => {
                if name.as_ref().ends_with(":key") {
                    Uuid::from_str(&name.as_ref()[0..name.as_ref().len() - 4]).ok()?;
                    Some(x.key_type())
                } else if name.as_ref().ends_with(":value") {
                    Uuid::from_str(&name.as_ref()[0..name.as_ref().len() - 6]).ok()?;
                    Some(x.value_type())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // Given a schema (that is likely a record with fields), depth-first search
    // it to find all the schemas that are used within it
    pub fn find_referenced_schemas<'a>(
        named_types: &'a HashMap<SchemaFingerprint, SchemaNamedType>,
        schema: &'a Schema,
        referenced_schema_fingerprints: &mut HashSet<SchemaFingerprint>,
        visit_stack: &mut Vec<&'a Schema>,
    ) {
        if visit_stack.contains(&schema) {
            return;
        }

        visit_stack.push(&schema);
        //referenced_schema_fingerprints.insert(schema)
        match schema {
            Schema::Nullable(inner) => Self::find_referenced_schemas(
                named_types,
                &*inner,
                referenced_schema_fingerprints,
                visit_stack,
            ),
            Schema::Boolean => {}
            Schema::I32 => {}
            Schema::I64 => {}
            Schema::U32 => {}
            Schema::U64 => {}
            Schema::F32 => {}
            Schema::F64 => {}
            Schema::Bytes => {}
            Schema::String => {}
            Schema::StaticArray(inner) => Self::find_referenced_schemas(
                named_types,
                inner.item_type(),
                referenced_schema_fingerprints,
                visit_stack,
            ),
            Schema::DynamicArray(inner) => Self::find_referenced_schemas(
                named_types,
                inner.item_type(),
                referenced_schema_fingerprints,
                visit_stack,
            ),
            Schema::Map(inner) => {
                Self::find_referenced_schemas(
                    named_types,
                    inner.key_type(),
                    referenced_schema_fingerprints,
                    visit_stack,
                );
                Self::find_referenced_schemas(
                    named_types,
                    inner.value_type(),
                    referenced_schema_fingerprints,
                    visit_stack,
                );
            }
            Schema::AssetRef(_) => {}
            Schema::Record(inner) => {
                referenced_schema_fingerprints.insert(*inner);
                let record = named_types.get(inner).unwrap().try_as_record().unwrap();
                for field in record.fields() {
                    Self::find_referenced_schemas(
                        named_types,
                        field.field_schema(),
                        referenced_schema_fingerprints,
                        visit_stack,
                    );
                }
            }
            Schema::Enum(inner) => {
                referenced_schema_fingerprints.insert(*inner);
            }
        }
        visit_stack.pop();
    }
}

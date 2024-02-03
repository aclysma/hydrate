mod dynamic_array;
pub use dynamic_array::*;

mod r#enum;
pub use r#enum::*;

mod interface;
pub use interface::*;

mod map;
pub use map::*;

mod record;
pub use record::*;

mod ref_constraint;
pub use ref_constraint::*;

mod static_array;
pub use static_array::*;

use crate::{HashSet, SchemaFingerprint};
use crate::{DataSetError, DataSetResult, HashMap};
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

    pub fn find_property_schema(
        &self,
        path: impl AsRef<str>,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> Option<Schema> {
        let mut schema = Schema::Record(self.fingerprint());

        //TODO: Escape map keys (and probably avoid path strings anyways)
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



    pub fn find_schemas_used_in_property_path(
        &self,
        path: impl AsRef<str>,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        used_schemas: &mut HashSet<SchemaFingerprint>
    ) {
        let mut schema = Schema::Record(self.fingerprint());

        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path = path.as_ref().split(".");

        // Iterate the path segments to find
        for path_segment in split_path {
            let s = schema.find_field_schema(path_segment, named_types);
            if let Some(s) = s {
                match s {
                    Schema::Record(fingerprint) => { used_schemas.insert(*fingerprint); }
                    Schema::Enum(fingerprint) => { used_schemas.insert(*fingerprint); }
                    _ => {},
                }

                schema = s.clone();
            } else {
                return;
            }
        }
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

    // This recursively finds the schema through a full path
    pub fn find_property_schema(
        &self,
        path: impl AsRef<str>,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> Option<Schema> {
        let mut schema = self;
        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path = path.as_ref().split(".");

        // Iterate the path segments to find
        for path_segment in split_path {
            let s = self.find_field_schema(path_segment, named_types);
            if let Some(s) = s {
                schema = s;
            } else {
                return None;
            }
        }

        Some(schema.clone())
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
        visit_stack: &mut Vec<&'a Schema>
    ) {
        if visit_stack.contains(&schema) {
            return;
        }

        visit_stack.push(&schema);
        //referenced_schema_fingerprints.insert(schema)
        match schema {
            Schema::Nullable(inner) => Self::find_referenced_schemas(named_types, &*inner, referenced_schema_fingerprints, visit_stack),
            Schema::Boolean => {}
            Schema::I32 => {}
            Schema::I64 => {}
            Schema::U32 => {}
            Schema::U64 => {}
            Schema::F32 => {}
            Schema::F64 => {}
            Schema::Bytes => {}
            Schema::String => {}
            Schema::StaticArray(inner) => Self::find_referenced_schemas(named_types, inner.item_type(), referenced_schema_fingerprints, visit_stack),
            Schema::DynamicArray(inner) => Self::find_referenced_schemas(named_types, inner.item_type(), referenced_schema_fingerprints, visit_stack),
            Schema::Map(inner) => {
                Self::find_referenced_schemas(named_types, inner.key_type(), referenced_schema_fingerprints, visit_stack);
                Self::find_referenced_schemas(named_types, inner.value_type(), referenced_schema_fingerprints, visit_stack);
            }
            Schema::AssetRef(_) => {}
            Schema::Record(inner) => {
                referenced_schema_fingerprints.insert(*inner);
                let record = named_types.get(inner).unwrap().try_as_record().unwrap();
                for field in record.fields() {
                    Self::find_referenced_schemas(named_types, field.field_schema(), referenced_schema_fingerprints, visit_stack);
                }
            }
            Schema::Enum(inner) => {
                referenced_schema_fingerprints.insert(*inner);
            }
        }
        visit_stack.pop();
    }
}

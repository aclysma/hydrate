
mod dynamic_array;
pub use dynamic_array::*;

mod r#enum;
pub use r#enum::*;

mod fixed;
pub use fixed::*;

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

use std::hash::{Hash, Hasher};
use siphasher::sip128::Hasher128;
use crate::SchemaFingerprint;

// Defines a unique number for each variant for hashing/fingerprinting purposes, the number is
// completely arbitrary
#[derive(Copy, Clone)]
enum SchemaTypeIndex {
    Nullable = 0,
    Boolean = 1,
    I32 = 2,
    I64 = 3,
    U32 = 4,
    U64 = 5,
    F32 = 6,
    F64 = 7,
    Bytes = 8,
    Buffer = 9,
    String = 10,
    StaticArray = 11,
    DynamicArray = 12,
    Map = 13,
    RecordRef = 14,
    Record = 15,
    Enum = 16,
    Fixed = 17,
}

impl SchemaTypeIndex {
    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        (*self as u32).hash(hasher);
    }
}

/// Describes format of data, either a single primitive value or complex layout comprised of
/// potentially many values
#[derive(Clone, Debug)]
pub enum Schema {
    //
    // Anonymous Types
    //

    /// Marks the field as possible to be null
    Nullable(Box<Schema>),
    Boolean,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    /// Variable amount of bytes stored within the object, intended to be relatively small
    Bytes,
    /// A variable amount of bytes stored on a reference-counted heap and shared. Can be large (MBs)
    Buffer,
    /// Variable-length UTF-8 String
    String,
    /// Fixed-size array of values
    StaticArray(SchemaStaticArray),
    DynamicArray(SchemaDynamicArray),
    Map(SchemaMap),
    RecordRef(SchemaRefConstraint),

    //
    // Named Types
    //

    /// An object or inlined struct within an object
    Record(SchemaRecord),
    Enum(SchemaEnum),
    Fixed(SchemaFixed),
}

impl Schema {
    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        match self {
            Schema::Nullable(inner) => {
                SchemaTypeIndex::Nullable.fingerprint_hash(hasher);
                inner.fingerprint_hash(hasher)
            },
            Schema::Boolean => SchemaTypeIndex::Boolean.fingerprint_hash(hasher),
            Schema::I32 => SchemaTypeIndex::I32.fingerprint_hash(hasher),
            Schema::I64 => SchemaTypeIndex::I64.fingerprint_hash(hasher),
            Schema::U32 => SchemaTypeIndex::U32.fingerprint_hash(hasher),
            Schema::U64 => SchemaTypeIndex::U64.fingerprint_hash(hasher),
            Schema::F32 => SchemaTypeIndex::F32.fingerprint_hash(hasher),
            Schema::F64 => SchemaTypeIndex::F64.fingerprint_hash(hasher),
            Schema::Bytes => SchemaTypeIndex::Bytes.fingerprint_hash(hasher),
            Schema::Buffer => SchemaTypeIndex::Buffer.fingerprint_hash(hasher),
            Schema::String => SchemaTypeIndex::String.fingerprint_hash(hasher),
            Schema::StaticArray(inner) => inner.fingerprint_hash(hasher),
            Schema::DynamicArray(inner) => inner.fingerprint_hash(hasher),
            Schema::Map(inner) => inner.fingerprint_hash(hasher),
            Schema::RecordRef(inner) => inner.fingerprint_hash(hasher),
            Schema::Record(inner) => inner.fingerprint_hash(hasher),
            Schema::Enum(inner) => inner.fingerprint_hash(hasher),
            Schema::Fixed(inner) => inner.fingerprint_hash(hasher),
        }
    }

    pub fn fingerprint(&self) -> SchemaFingerprint {
        let mut hasher = siphasher::sip128::SipHasher::default();
        self.fingerprint_hash(&mut hasher);
        SchemaFingerprint(hasher.finish128().as_u128())
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            Schema::Nullable(_) => true,
            _ => false
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Schema::Nullable(x) => x.is_boolean(),
            Schema::Boolean => true,
            _ => false
        }
    }

    pub fn is_i32(&self) -> bool {
        match self {
            Schema::I32 => true,
            _ => false
        }
    }

    pub fn is_u32(&self) -> bool {
        match self {
            Schema::U32 => true,
            _ => false
        }
    }

    pub fn is_i64(&self) -> bool {
        match self {
            Schema::I64 => true,
            _ => false
        }
    }

    pub fn is_u64(&self) -> bool {
        match self {
            Schema::U64 => true,
            _ => false
        }
    }

    pub fn is_f32(&self) -> bool {
        match self {
            Schema::F32 => true,
            _ => false
        }
    }

    pub fn is_f64(&self) -> bool {
        match self {
            Schema::F64 => true,
            _ => false
        }
    }

    pub fn find_property_schema(&self, name: impl AsRef<str>) -> Option<&Schema> {
        let mut record = None;
        match self {
            Schema::Nullable(x) => {
                if let Schema::Record(x) = &**x {
                    record = Some(x);
                }
            },
            Schema::Record(x) => {
                record = Some(x);
            },
            _ => {}
        }

        record.map(|x| x.field_schema(name)).flatten()
    }

    pub fn find_property_path_schema<T: AsRef<str>>(&self, path: &[T]) -> Option<&Schema> {
        let mut schema = self;

        for path_element in path {
            let s = schema.find_property_schema(path_element);
            if let Some(s) = s {
                schema = s;
            } else {
                return None;
            }
        }

        Some(schema)
    }

    pub fn as_record(&self) -> Option<&SchemaRecord> {
        if let Schema::Record(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Schema, SchemaRecord, SchemaRecordField};

    // We want the same fingerprint out of a record as a Schema::Record(record)
    #[test]
    fn record_fingerprint_equivalency() {
        let vec3_schema_record = SchemaRecord::new("Vec3".to_string(), vec![].into_boxed_slice(), vec![
            SchemaRecordField::new("x".to_string(), vec![].into_boxed_slice(), Schema::F32),
            SchemaRecordField::new("y".to_string(), vec![].into_boxed_slice(), Schema::F32),
            SchemaRecordField::new("z".to_string(), vec![].into_boxed_slice(), Schema::F32)
        ].into_boxed_slice());

        // Fingerprint of a Schema::Record is == to fingerprint of wrapped SchemaRecord
        assert_eq!(vec3_schema_record.fingerprint(), Schema::Record(vec3_schema_record).fingerprint());
    }

    #[test]
    fn test_property_path() {
        let vec3_schema_record = SchemaRecord::new("Vec3".to_string(), vec![].into_boxed_slice(), vec![
            SchemaRecordField::new("x".to_string(), vec![].into_boxed_slice(), Schema::F32),
            SchemaRecordField::new("y".to_string(), vec![].into_boxed_slice(), Schema::F32),
            SchemaRecordField::new("z".to_string(), vec![].into_boxed_slice(), Schema::F32)
        ].into_boxed_slice());

        let aabb_schema_record = SchemaRecord::new("AABB".to_string(), vec![].into_boxed_slice(), vec![
            SchemaRecordField::new("min".to_string(), vec![].into_boxed_slice(), Schema::Record(vec3_schema_record.clone())),
            SchemaRecordField::new("max".to_string(), vec![].into_boxed_slice(), Schema::Record(vec3_schema_record.clone()))
        ].into_boxed_slice());

        let aabb_schema = Schema::Record(aabb_schema_record);

        // Access properties
        assert_eq!(aabb_schema.find_property_path_schema::<&str>(&[]).unwrap().fingerprint(), aabb_schema.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["min"]).unwrap().fingerprint(), Schema::Record(vec3_schema_record.clone()).fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["max"]).unwrap().fingerprint(), Schema::Record(vec3_schema_record.clone()).fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["min", "x"]).unwrap().fingerprint(), Schema::F32.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["min", "y"]).unwrap().fingerprint(), Schema::F32.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["min", "z"]).unwrap().fingerprint(), Schema::F32.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["max", "x"]).unwrap().fingerprint(), Schema::F32.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["max", "y"]).unwrap().fingerprint(), Schema::F32.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["max", "z"]).unwrap().fingerprint(), Schema::F32.fingerprint());
        assert_eq!(aabb_schema.find_property_path_schema(&["max"]).unwrap().find_property_path_schema(&["x"]).unwrap().fingerprint(), Schema::F32.fingerprint());

        // Fail at accessing non-existent properties
        assert_eq!(aabb_schema.find_property_path_schema(&["min", "A"]).map(|x| x.fingerprint()), None);
        assert_eq!(aabb_schema.find_property_path_schema(&["min", "x", "asdfs"]).map(|x| x.fingerprint()), None);
        assert_eq!(aabb_schema.find_property_path_schema(&["aa", "x"]).map(|x| x.fingerprint()), None);
    }
}


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
use uuid::Uuid;

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

    pub fn fingerprint128(&self) -> u128 {
        let mut hasher = siphasher::sip128::SipHasher::default();
        self.fingerprint_hash(&mut hasher);
        hasher.finish128().as_u128()
    }

    pub fn fingerprint_uuid(&self) -> Uuid {
        Uuid::from_u128(self.fingerprint128())
    }
}

#[cfg(test)]
mod test {
    use crate::{Schema, SchemaRecord, SchemaRecordField, SchemaStaticArray};

    // We want the same fingerprint out of a record as a Schema::Record(record)
    #[test]
    fn record_fingerprint_equivalency() {
        let static_array = SchemaRecord::new("Vec3".to_string(), vec![].into_boxed_slice(), vec![
            SchemaRecordField::new("x".to_string(), vec![].into_boxed_slice(), Schema::F32),
            SchemaRecordField::new("y".to_string(), vec![].into_boxed_slice(), Schema::F32),
            SchemaRecordField::new("z".to_string(), vec![].into_boxed_slice(), Schema::F32)
        ].into_boxed_slice());

        assert_eq!(static_array.fingerprint128(), Schema::Record(static_array).fingerprint128());
    }
}

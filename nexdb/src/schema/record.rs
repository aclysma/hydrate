use super::Schema;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use siphasher::sip128::Hasher128;
use uuid::Uuid;
use crate::schema::SchemaTypeIndex;

//
// Named Types
//
#[derive(Debug)]
pub struct SchemaRecordField {
    name: String,
    aliases: Box<[String]>,
    field_type: Schema,
}

impl SchemaRecordField {
    pub fn new(
        name: String,
        aliases: Box<[String]>,
        field_type: Schema,

    ) -> Self {
        SchemaRecordField {
            name,
            aliases,
            field_type
        }
    }

    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        self.name.hash(hasher);
        self.field_type.fingerprint_hash(hasher);
    }
}

#[derive(Debug)]
pub struct SchemaRecordInner {
    name: String,
    fingerprint: u128,
    aliases: Box<[String]>,
    fields: Box<[SchemaRecordField]>,
}

#[derive(Clone, Debug)]
pub struct SchemaRecord {
    inner: Arc<SchemaRecordInner>
}

impl Deref for SchemaRecord {
    type Target = SchemaRecordInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

fn record_fingerprint_hash<T: Hasher>(hasher: &mut T, name: &str, fields: &[SchemaRecordField]) {
    SchemaTypeIndex::Record.fingerprint_hash(hasher);
    name.hash(hasher);
    for field in &*fields {
        field.fingerprint_hash(hasher);
    }
}

impl SchemaRecord {
    pub fn new(name: String, aliases: Box<[String]>, fields: Box<[SchemaRecordField]>) -> Self {
        // Check names are unique
        for i in 0..fields.len() {
            for j in 0..i {
                assert_ne!(fields[i].name, fields[j].name);
            }
        }

        let mut hasher = siphasher::sip128::SipHasher::default();
        record_fingerprint_hash(&mut hasher, &name, &*fields);
        let fingerprint = hasher.finish128().as_u128();

        let inner = SchemaRecordInner {
            name,
            fingerprint,
            aliases,
            fields
        };

        SchemaRecord {
            inner: Arc::new(inner)
        }
    }

    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        SchemaTypeIndex::Record.fingerprint_hash(hasher);
        self.name.hash(hasher);
        for field in &*self.fields {
            field.fingerprint_hash(hasher);
        }
    }

    pub fn fingerprint128(&self) -> u128 {
        self.fingerprint
    }

    pub fn fingerprint_uuid(&self) -> Uuid {
        Uuid::from_u128(self.fingerprint128())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

use super::SchemaRecord;
use super::SchemaInterface;
use std::hash::{Hash, Hasher};
use crate::schema::SchemaTypeIndex;

#[derive(Clone, Debug)]
pub enum SchemaRefConstraint {
    Concrete(SchemaRecord),
    Interface(SchemaInterface),
}

impl SchemaRefConstraint {
    // pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
    //     SchemaTypeIndex::RecordRef.fingerprint_hash(hasher);
    //     match self {
    //         SchemaRefConstraint::Concrete(inner) => {
    //             1.hash(hasher);
    //             inner.fingerprint_hash(hasher);
    //         }
    //         SchemaRefConstraint::Interface(inner) => {
    //             2.hash(hasher);
    //             inner.fingerprint_hash(hasher);
    //         }
    //     }
    // }
}
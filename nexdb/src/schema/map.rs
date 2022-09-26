use super::Schema;
use std::hash::{Hasher};
use crate::schema::SchemaTypeIndex;

#[derive(Clone, Debug)]
pub struct SchemaMap {
    //TODO: Check key_type is not an undesirable type (i.e. must be hashable)
    key_type: Box<Schema>,
    value_type: Box<Schema>,
}

impl SchemaMap {
    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        SchemaTypeIndex::Map.fingerprint_hash(hasher);
        self.key_type.fingerprint_hash(hasher);
        self.value_type.fingerprint_hash(hasher);
    }
}
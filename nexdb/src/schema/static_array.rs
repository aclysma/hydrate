use super::Schema;
use std::hash::{Hash, Hasher};
use crate::schema::SchemaTypeIndex;

//
// Anonymous Types
//
#[derive(Clone, Debug)]
pub struct SchemaStaticArray {
    pub(crate) item_type: Box<Schema>,
    pub(crate) length: usize
}

impl SchemaStaticArray {
    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        SchemaTypeIndex::StaticArray.fingerprint_hash(hasher);
        self.item_type.fingerprint_hash(hasher);
        self.length.hash(hasher);
    }
}
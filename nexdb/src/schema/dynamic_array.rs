use super::Schema;
use std::hash::{Hash, Hasher};
use crate::schema::SchemaTypeIndex;

#[derive(Clone, Debug)]
pub struct SchemaDynamicArray {
    item_type: Box<Schema>,
    //max_length: Option<usize>,
}

impl SchemaDynamicArray {
    pub(crate) fn new(
        item_type: Box<Schema>,
        //max_length: Option<usize>
    ) -> Self {
        SchemaDynamicArray {
            item_type,
            //max_length
        }
    }

    pub(crate) fn item_type(&self) -> &Schema {
        &*self.item_type
    }

    // pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
    //     SchemaTypeIndex::DynamicArray.fingerprint_hash(hasher);
    //     self.item_type.fingerprint_hash(hasher);
    //     //self.max_length.hash(hasher);
    // }
}
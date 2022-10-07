use super::Schema;
use std::hash::{Hash, Hasher};
use crate::schema::SchemaTypeIndex;

#[derive(Clone, Debug)]
pub struct SchemaStaticArray {
    pub(crate) item_type: Box<Schema>,
    pub(crate) length: usize
}

impl SchemaStaticArray {
    pub(crate) fn new(
        item_type: Box<Schema>,
        length: usize,
    ) -> Self {
        SchemaStaticArray {
            item_type,
            length
        }
    }

    pub fn item_type(&self) -> &Schema {
        &*self.item_type
    }
}
use super::Schema;
use std::hash::{Hash, Hasher};
use crate::schema::SchemaTypeIndex;

#[derive(Clone, Debug)]
pub struct SchemaDynamicArray {
    item_type: Box<Schema>,
}

impl SchemaDynamicArray {
    pub(crate) fn new(
        item_type: Box<Schema>,
    ) -> Self {
        SchemaDynamicArray {
            item_type,
        }
    }

    pub(crate) fn item_type(&self) -> &Schema {
        &*self.item_type
    }
}
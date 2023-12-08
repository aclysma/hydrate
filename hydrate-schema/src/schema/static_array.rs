use super::Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct SchemaStaticArray {
    pub(crate) item_type: Box<Schema>,
    pub(crate) length: usize,
}

impl SchemaStaticArray {
    pub(crate) fn new(
        item_type: Box<Schema>,
        length: usize,
    ) -> Self {
        SchemaStaticArray { item_type, length }
    }

    pub fn item_type(&self) -> &Schema {
        &*self.item_type
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

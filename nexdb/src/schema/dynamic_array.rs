use super::Schema;

#[derive(Clone, Debug)]
pub struct SchemaDynamicArray {
    item_type: Box<Schema>,
}

impl SchemaDynamicArray {
    pub(crate) fn new(item_type: Box<Schema>) -> Self {
        SchemaDynamicArray { item_type }
    }

    pub fn item_type(&self) -> &Schema {
        &*self.item_type
    }
}

use super::Schema;
use std::hash::{Hasher};
use crate::schema::SchemaTypeIndex;

#[derive(Clone, Debug)]
pub struct SchemaMap {
    key_type: Box<Schema>,
    value_type: Box<Schema>,
}

impl SchemaMap {
    pub fn new(key_type: Box<Schema>, value_type: Box<Schema>) -> Self {
        //TODO: Check key_type is not an undesirable type (i.e. must be hashable)
        SchemaMap {
            key_type,
            value_type
        }
    }

    pub fn key_type(&self) -> &Schema {
        &*self.key_type
    }

    pub fn value_type(&self) -> &Schema {
        &*self.value_type
    }
}
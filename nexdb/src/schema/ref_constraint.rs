use super::SchemaInterface;
use super::SchemaRecord;
use crate::schema::SchemaTypeIndex;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum SchemaRefConstraint {
    Concrete(SchemaRecord),
    Interface(SchemaInterface),
}

impl SchemaRefConstraint {}

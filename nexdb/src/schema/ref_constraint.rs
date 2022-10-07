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

}

#![allow(dead_code)]
// extern crate core;

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashSet<T> = std::collections::HashMap<T, ahash::RandomState>;

use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SchemaFingerprint(u128);
impl SchemaFingerprint {
    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjectId(u128);
impl ObjectId {
    pub fn null() -> Self {
        ObjectId(0)
    }
}

mod schema;
pub use schema::*;

mod value;
pub use value::*;

mod database;
pub use database::*;
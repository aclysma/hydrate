#![allow(dead_code)]
// extern crate core;

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BufferId(u128);
impl BufferId {
    pub fn null() -> Self {
        BufferId(0)
    }
}

mod schema;
pub use schema::*;

mod schema_def;
pub use schema_def::*;

pub use database::value::*;

mod database;
pub use database::*;

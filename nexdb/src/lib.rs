#![allow(dead_code)]
// extern crate core;

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashMapKeys<'a, K, V> = std::collections::hash_map::Keys<'a, K, V>;
pub type HashMapValues<'a, K, V> = std::collections::hash_map::Values<'a, K, V>;
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
pub type HashSetIter<'a, T> = std::collections::hash_set::Iter<'a, T>;

use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SchemaFingerprint(u128);
impl SchemaFingerprint {
    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct ObjectId(pub u128);
impl ObjectId {
    pub fn null() -> Self {
        ObjectId(0)
    }

    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
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

mod database;
pub use database::*;

mod schema_cache;
pub use schema_cache::*;

mod data_storage;
pub use data_storage::*;

mod editor;
pub use editor::*;

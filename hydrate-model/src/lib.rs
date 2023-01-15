#![allow(dead_code)]
// extern crate core;

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashMapKeys<'a, K, V> = std::collections::hash_map::Keys<'a, K, V>;
pub type HashMapValues<'a, K, V> = std::collections::hash_map::Values<'a, K, V>;
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
pub type HashSetIter<'a, T> = std::collections::hash_set::Iter<'a, T>;

use std::fmt::{Debug, Formatter};
use uuid::Uuid;

pub mod uuid_path;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SchemaFingerprint(u128);
impl SchemaFingerprint {
    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }
}

impl Debug for SchemaFingerprint {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("SchemaFingerprint")
            .field(&Uuid::from_u128(self.0))
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectId(pub u128);
impl ObjectId {
    pub fn null() -> Self {
        ObjectId(0)
    }

    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn is_null(&self) -> bool {
        return self.0 == 0;
    }
}

impl Debug for ObjectId {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("ObjectId")
            .field(&Uuid::from_u128(self.0))
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BufferId(u128);
impl BufferId {
    pub fn null() -> Self {
        BufferId(0)
    }
}

impl Debug for BufferId {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("BufferId")
            .field(&Uuid::from_u128(self.0))
            .finish()
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

// mod storage;
// pub use storage::*;

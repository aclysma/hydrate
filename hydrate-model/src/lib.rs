use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt::{Debug, Formatter};

pub mod uuid_path;

mod metadata;
pub use metadata::BuiltObjectMetadata;

pub use hydrate_schema::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ObjectId(pub u128);
impl ObjectId {
    pub const fn null() -> Self {
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
    pub const fn null() -> Self {
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

mod database;
pub use database::*;

mod data_storage;
pub use data_storage::*;

mod editor;
pub use editor::*;

pub mod wrappers;
pub use wrappers::*;

// mod storage;
// pub use storage::*;

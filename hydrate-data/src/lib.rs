
use uuid::Uuid;
//use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

pub use hydrate_schema::*;

pub mod value;
pub use value::Value;

mod data_set;
pub use data_set::BuildInfo;
pub use data_set::BuilderId;
pub use data_set::DataObjectInfo;
pub use data_set::DataSet;
pub use data_set::ImportInfo;
pub use data_set::ImporterId;
pub use data_set::ObjectLocation;
pub use data_set::ObjectName;
pub use data_set::ObjectSourceId;
pub use data_set::OverrideBehavior;
pub use data_set::DataSetError;
pub use data_set::DataSetResult;

mod data_set_view;
pub use data_set_view::DataContainer;
pub use data_set_view::DataContainerMut;
pub use data_set_view::DataSetView;
pub use data_set_view::DataSetViewMut;

mod single_object;
pub use single_object::SingleObject;

mod diff;
pub use diff::DataSetDiff;
pub use diff::DataSetDiffSet;

mod property_util_fn;
use property_util_fn::*;

mod traits;
pub use traits::*;

#[cfg(test)]
mod tests;

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum NullOverride {
    SetNull,
    SetNonNull,
}

pub use hydrate_base::ObjectId;

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
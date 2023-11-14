
use std::fmt::Debug;

pub use hydrate_schema::*;

pub mod value;
pub use value::Value;

mod data_set;
pub use data_set::BuildInfo;
pub use data_set::BuilderId;
pub use data_set::DataAssetInfo;
pub use data_set::DataSet;
pub use data_set::DataSetError;
pub use data_set::DataSetResult;
pub use data_set::ImportInfo;
pub use data_set::ImporterId;
pub use data_set::AssetLocation;
pub use data_set::AssetName;
pub use data_set::AssetSourceId;
pub use data_set::OverrideBehavior;

mod data_set_view;
pub use data_set_view::DataContainer;
pub use data_set_view::DataContainerMut;

mod single_object;
pub use single_object::SingleObject;

mod diff;
pub use diff::DataSetDiff;
pub use diff::DataSetDiffSet;

mod property_util_fn;
use property_util_fn::*;

mod field_wrappers;
pub use field_wrappers::*;

mod schema_set;
pub use schema_set::{SchemaSetBuilder, SchemaSet};

mod ordered_set;
pub use ordered_set::OrderedSet;

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

pub use hydrate_base::AssetId;

pub use hydrate_schema::*;
pub use hydrate_schema::{DataSetError, DataSetResult};

pub mod value;
pub use value::Value;

pub mod json_storage;

mod data_set;
pub use data_set::HashObjectMode;
pub use data_set::AssetLocation;
pub use data_set::AssetName;
pub use data_set::BuildInfo;
pub use data_set::BuilderId;
pub use data_set::DataSet;
pub use data_set::DataSetAssetInfo;
pub use data_set::ImportInfo;
pub use data_set::ImportableName;
pub use data_set::ImporterId;
pub use data_set::OverrideBehavior;
pub use data_set::PropertiesBundle;

mod data_set_view;
pub use data_set_view::DataContainer;
pub use data_set_view::DataContainerRef;
pub use data_set_view::DataContainerRefMut;

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
pub use schema_set::{SchemaSet, SchemaSetBuilder};

mod ordered_set;

mod path_reference;
pub use path_reference::PathReference;
pub use path_reference::PathReferenceHash;
pub use path_reference::CanonicalPathReference;
pub use path_reference::PathReferenceNamespaceResolver;

pub use ordered_set::OrderedSet;

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum NullOverride {
    Unset,
    SetNull,
    SetNonNull,
}

pub use hydrate_base::AssetId;

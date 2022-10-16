pub mod value;
pub use value::Value;

mod data_set;
pub use data_set::DataObjectInfo;
pub use data_set::DataSet;
pub use data_set::NullOverride;
pub use data_set::ObjectLocation;
pub use data_set::ObjectPath;
pub use data_set::ObjectSourceId;
pub use data_set::OverrideBehavior;

mod schema_set;
pub use schema_set::SchemaSet;

mod diff;
pub use diff::DataSetDiffSet;

#[cfg(test)]
mod tests;

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.

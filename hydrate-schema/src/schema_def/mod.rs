//! This package handles defining schema programmatically and by loading from json. A schema
//! may reference other schemas, even cyclically, so we have to load all the schemas and do a final
//! "linking" pass.

mod schema_def;
pub use schema_def::*;

mod schema_linker;
pub use schema_linker::*;

mod enum_type_builder;
mod fixed_type_builder;
mod record_type_builder;

mod json_schema;

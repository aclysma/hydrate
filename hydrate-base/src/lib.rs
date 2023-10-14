pub mod hashing;

pub mod uuid_path;

pub mod built_object_metadata;
pub use built_object_metadata::*;

//TODO: ObjectId/AssetId should be merged
mod asset_uuid;
pub use asset_uuid::AssetUuid;

mod object_id;
pub use object_id::ObjectId;

mod asset_type_id;
pub use asset_type_id::AssetTypeId;

mod asset_ref;
pub use asset_ref::AssetRef;

pub mod handle;
pub use handle::Handle;
pub use handle::LoadHandle;

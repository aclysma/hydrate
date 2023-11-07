use hydrate_base::Handle;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

//
// Engine-specific types start here
//

#[derive(Serialize, Deserialize, TypeUuid, Debug)]
#[uuid = "7132d33e-9bbc-4fb1-b857-17962afd44b8"]
pub struct TransformRef {
    pub transform: Handle<Transform>,
}

#[derive(Serialize, Deserialize, TypeUuid, Debug)]
#[uuid = "da334afa-7af9-4894-8b7e-29defe202e90"]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

// What if we had a way to "bind" raw rust structs to fields? Needs to know how to read and write,
// but we really just need to provide method of getting a ref and mutable ref to individual fields

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "df64f515-7e2f-47c2-b4d3-17ec7f2e63c7"]
pub struct AllFields {
    pub boolean: bool,
    pub int32: i32,
    pub int64: i64,
}

use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "281d21ca-9a70-473a-a531-e7d4734a0b53"]
pub struct GpuBufferBuiltData {
    pub resource_type: u32,
    pub alignment: u32,
    pub data: Vec<u8>,
}

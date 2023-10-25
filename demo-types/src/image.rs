use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "1a4dde10-5e60-483d-88fa-4f59752e4524"]
pub struct GpuImageBuiltData {
    pub image_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

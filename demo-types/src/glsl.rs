use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "e88ff29e-7af3-4c67-96a0-de53174454ed"]
pub struct GlslBuildTargetBuiltData {
    pub spv: Vec<u8>,
}

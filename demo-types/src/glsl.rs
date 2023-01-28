use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GlslBuildTargetBuiltData {
    pub spv: Vec<u8>,
}

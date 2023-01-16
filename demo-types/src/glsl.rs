use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GlslBuildTargetBuiltData {
    pub spv: Vec<u8>,
}

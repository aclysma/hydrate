use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ImageBuiltData {
    pub image_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

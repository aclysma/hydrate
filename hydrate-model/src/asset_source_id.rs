use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AssetSourceId(Uuid);

impl AssetSourceId {
    pub fn new() -> Self {
        AssetSourceId(Uuid::new_v4())
    }

    pub fn null() -> Self {
        AssetSourceId(Uuid::nil())
    }

    pub fn uuid(&self) -> &Uuid {
        &self.0
    }
}
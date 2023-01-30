use crate::AssetUuid;

/// A potentially unresolved reference to an asset
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetRef {
    Uuid(AssetUuid),
    //Path(std::path::PathBuf),
}
impl AssetRef {
    pub fn expect_uuid(&self) -> &AssetUuid {
        if let AssetRef::Uuid(uuid) = self {
            uuid
        } else {
            panic!("Expected AssetRef::Uuid, got {:?}", self)
        }
    }

    // pub fn is_path(&self) -> bool {
    //     matches!(self, AssetRef::Path(_))
    // }
    //
    // pub fn is_uuid(&self) -> bool {
    //     matches!(self, AssetRef::Uuid(_))
    // }
}

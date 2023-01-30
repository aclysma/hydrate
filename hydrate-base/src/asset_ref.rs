use crate::AssetUuid;
use serde::{Serialize, Deserialize};

/// A potentially unresolved reference to an asset
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AssetRef(pub AssetUuid);

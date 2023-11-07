use crate::AssetUuid;
use serde::{Deserialize, Serialize};

/// A potentially unresolved reference to an asset
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AssetRef(pub AssetUuid);

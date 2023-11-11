use crate::AssetId;
use serde::{Deserialize, Serialize};

/// A potentially unresolved reference to an asset
/// TODO: This might be more accurate to call it an ArtifactId, and maybe we could just get rid of this newtype entirely?
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AssetRef(pub AssetId);

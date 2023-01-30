use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "713e2e35-a394-4a80-b792-a1d95e5ad936"]
pub struct GltfBuiltMeshData {}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "d869d355-48a8-4a90-a757-e88810e80d5f"]
pub struct GltfBuiltMaterialData {}

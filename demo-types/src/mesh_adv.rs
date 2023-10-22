use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use hydrate_base::Handle;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshAdvShadowMethod {
    None,
    Opaque,
    //AlphaClip,
    //AlphaStochastic,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshAdvBlendMethod {
    Opaque,
    AlphaClip,
    //AlphaStochastic,
    AlphaBlend,
}

// This is non-texture data associated with the material. Must convert to
// MeshMaterialDataShaderParam to bind to a shader uniform
#[derive(Serialize, Deserialize, Clone, Debug, type_uuid::TypeUuid)]
#[repr(C)]
#[uuid = "90228283-3d7f-4ba8-9e20-6cc2871ad9ff"]
pub struct MeshAdvMaterialData {
    // Using f32 arrays for serde support
    pub base_color_factor: [f32; 4], // default: 1,1,1,1
    pub emissive_factor: [f32; 3],   // default: 0,0,0
    pub metallic_factor: f32,        //default: 1,
    pub roughness_factor: f32,       // default: 1,
    pub normal_texture_scale: f32,   // default: 1

    pub has_base_color_texture: bool,
    pub base_color_texture_has_alpha_channel: bool,
    pub has_metallic_roughness_texture: bool,
    pub has_normal_texture: bool,
    pub has_emissive_texture: bool,

    pub shadow_method: MeshAdvShadowMethod,
    pub blend_method: MeshAdvBlendMethod,
    pub alpha_threshold: f32,
    pub backface_culling: bool,
}

impl Default for MeshAdvMaterialData {
    fn default() -> Self {
        MeshAdvMaterialData {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            emissive_factor: [0.0, 0.0, 0.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            normal_texture_scale: 1.0,
            has_base_color_texture: false,
            base_color_texture_has_alpha_channel: false,
            has_metallic_roughness_texture: false,
            has_normal_texture: false,
            has_emissive_texture: false,
            shadow_method: MeshAdvShadowMethod::Opaque,
            blend_method: MeshAdvBlendMethod::Opaque,
            alpha_threshold: 0.5,
            backface_culling: true,
        }
    }
}





pub struct MeshAdvBufferAssetData {
    //pub resource_type: RafxResourceType,
    pub alignment: u32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshAdvPartAssetData {
    pub vertex_full_buffer_offset_in_bytes: u32,
    pub vertex_full_buffer_size_in_bytes: u32,
    pub vertex_position_buffer_offset_in_bytes: u32,
    pub vertex_position_buffer_size_in_bytes: u32,
    pub index_buffer_offset_in_bytes: u32,
    pub index_buffer_size_in_bytes: u32,
    //pub mesh_material: Handle<MeshMaterialAdvAsset>,
    //pub index_type: RafxIndexType,
}

#[derive(TypeUuid, Serialize, Deserialize, Clone)]
#[uuid = "4c888448-2650-4f56-82dc-71ba81f4295b"]
pub struct MeshAdvAssetData {
    pub mesh_parts: Vec<MeshAdvPartAssetData>,
    pub vertex_full_buffer: Handle<MeshAdvBufferAssetData>, // Vertex type is MeshVertexFull
    pub vertex_position_buffer: Handle<MeshAdvBufferAssetData>, // Vertex type is MeshVertexPosition
    pub index_buffer: Handle<MeshAdvBufferAssetData>,       // u16 indices
    //pub visible_bounds: VisibleBounds,
}







#[cfg(feature = "editor-types")]
use super::generated::{MeshAdvShadowMethodEnum, MeshAdvBlendMethodEnum};

#[cfg(feature = "editor-types")]
impl Into<MeshAdvBlendMethod> for MeshAdvBlendMethodEnum {
    fn into(self) -> MeshAdvBlendMethod {
        match self {
            MeshAdvBlendMethodEnum::Opaque => MeshAdvBlendMethod::Opaque,
            MeshAdvBlendMethodEnum::AlphaClip => MeshAdvBlendMethod::AlphaClip,
            MeshAdvBlendMethodEnum::AlphaBlend => MeshAdvBlendMethod::AlphaBlend,
        }
    }
}

#[cfg(feature = "editor-types")]
impl Into<MeshAdvShadowMethod> for MeshAdvShadowMethodEnum {
    fn into(self) -> MeshAdvShadowMethod {
        match self {
            MeshAdvShadowMethodEnum::None => MeshAdvShadowMethod::None,
            MeshAdvShadowMethodEnum::Opaque => MeshAdvShadowMethod::Opaque,
        }
    }
}

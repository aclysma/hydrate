use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
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

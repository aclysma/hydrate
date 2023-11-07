use hydrate_base::Handle;
use rafx_api::{
    RafxBlendState, RafxCullMode, RafxDepthState, RafxFillMode, RafxFrontFace, RafxIndexType,
    RafxRasterizerState, RafxResourceType, RafxSamplerDef,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(TypeUuid, Serialize, Deserialize, Debug, Clone, Hash, PartialEq)]
#[uuid = "7f30b29c-7fb9-4b31-a354-7cefbbade2f9"]
pub struct SamplerAssetData {
    pub sampler: RafxSamplerDef,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq)]
pub enum AlphaBlendingPreset {
    Disabled,
    Enabled,
    Custom,
}

impl Default for AlphaBlendingPreset {
    fn default() -> Self {
        AlphaBlendingPreset::Disabled
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq)]
pub enum DepthBufferPreset {
    Disabled,
    Enabled,
    ReadOnly,
    EnabledReverseZ,
    ReadOnlyReverseZ,
    WriteOnly,
    Custom,
}

impl Default for DepthBufferPreset {
    fn default() -> Self {
        DepthBufferPreset::Disabled
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq)]
pub struct FixedFunctionStateData {
    #[serde(default)]
    blend_state: RafxBlendState,
    #[serde(default)]
    depth_state: RafxDepthState,
    #[serde(default)]
    rasterizer_state: RafxRasterizerState,

    // These override the above states
    #[serde(default)]
    alpha_blending: AlphaBlendingPreset,
    #[serde(default)]
    depth_testing: DepthBufferPreset,
    #[serde(default)]
    cull_mode: Option<RafxCullMode>,
    #[serde(default)]
    front_face: Option<RafxFrontFace>,
    #[serde(default)]
    fill_mode: Option<RafxFillMode>,
    #[serde(default)]
    depth_bias: Option<i32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialShaderStage {
    Vertex,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
    Fragment,
    Compute,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GraphicsPipelineShaderStage {
    pub stage: MaterialShaderStage,
    //pub shader_module: Handle<ShaderAsset>,
    pub entry_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MaterialPassData {
    pub name: Option<String>,
    pub phase: Option<String>,
    pub fixed_function_state: FixedFunctionStateData,
    pub shaders: Vec<GraphicsPipelineShaderStage>,
}

#[derive(TypeUuid, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[uuid = "ad94bca2-1f02-4e5f-9117-1a7b03456a11"]
pub struct MaterialAssetData {
    pub passes: Vec<MaterialPassData>,
}

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

#[derive(TypeUuid, Serialize, Deserialize, Clone)]
#[uuid = "41ea076f-19d7-4deb-8af1-983148af5383"]
pub struct MeshAdvMaterialAssetData {
    //pub material_asset: Handle<MaterialAsset>,
    //pub material_data: MeshAdvMaterialData,
    //pub color_texture: Option<Handle<ImageAsset>>,
    //pub metallic_roughness_texture: Option<Handle<ImageAsset>>,
    //pub normal_texture: Option<Handle<ImageAsset>>,
    //pub emissive_texture: Option<Handle<ImageAsset>>,
}

#[derive(TypeUuid, Serialize, Deserialize, Clone)]
#[uuid = "4b53d85c-98e6-4d77-af8b-0914e67e10dc"]
pub struct MeshAdvBufferAssetData {
    pub resource_type: RafxResourceType,
    pub alignment: u32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeshAdvPartAssetData {
    pub vertex_full_buffer_offset_in_bytes: u32,
    pub vertex_full_buffer_size_in_bytes: u32,
    pub vertex_position_buffer_offset_in_bytes: u32,
    pub vertex_position_buffer_size_in_bytes: u32,
    pub index_buffer_offset_in_bytes: u32,
    pub index_buffer_size_in_bytes: u32,
    pub mesh_material: Handle<MeshAdvMaterialData>,
    pub index_type: RafxIndexType,
}

#[derive(TypeUuid, Serialize, Deserialize, Clone, Debug)]
#[uuid = "4c888448-2650-4f56-82dc-71ba81f4295b"]
pub struct MeshAdvMeshAssetData {
    pub mesh_parts: Vec<MeshAdvPartAssetData>,
    pub vertex_full_buffer: Option<Handle<MeshAdvBufferAssetData>>, // Vertex type is MeshVertexFull
    pub vertex_position_buffer: Option<Handle<MeshAdvBufferAssetData>>, // Vertex type is MeshVertexPosition
                                                                        // pub index_buffer: Handle<MeshAdvBufferAssetData>,       // u16 indices
                                                                        //pub visible_bounds: VisibleBounds,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
#[repr(C)]
pub struct MeshVertexFull {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub binormal: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
#[repr(C)]
pub struct MeshVertexPosition {
    pub position: [f32; 3],
}

// This file generated automatically by hydrate-codegen. Do not make manual edits. Use include!() to place these types in the intended location.
#[derive(Default)]
pub struct AllFieldsRecord(PropertyPath);

impl Field for AllFieldsRecord {
    fn new(property_path: PropertyPath) -> Self {
        AllFieldsRecord(property_path)
    }
}

impl Record for AllFieldsRecord {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl AllFieldsRecord {
    pub fn boolean(&self) -> BooleanField {
        BooleanField::new(self.0.push("boolean"))
    }

    pub fn dynamic_array_i32(&self) -> DynamicArrayField::<I32Field> {
        DynamicArrayField::<I32Field>::new(self.0.push("dynamic_array_i32"))
    }

    pub fn dynamic_array_vec3(&self) -> DynamicArrayField::<Vec3Record> {
        DynamicArrayField::<Vec3Record>::new(self.0.push("dynamic_array_vec3"))
    }

    pub fn f32(&self) -> F32Field {
        F32Field::new(self.0.push("f32"))
    }

    pub fn f64(&self) -> F64Field {
        F64Field::new(self.0.push("f64"))
    }

    pub fn i32(&self) -> I32Field {
        I32Field::new(self.0.push("i32"))
    }

    pub fn i64(&self) -> I64Field {
        I64Field::new(self.0.push("i64"))
    }

    pub fn nullable_bool(&self) -> NullableField::<BooleanField> {
        NullableField::<BooleanField>::new(self.0.push("nullable_bool"))
    }

    pub fn nullable_vec3(&self) -> NullableField::<Vec3Record> {
        NullableField::<Vec3Record>::new(self.0.push("nullable_vec3"))
    }

    pub fn reference(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("reference"))
    }

    pub fn string(&self) -> StringField {
        StringField::new(self.0.push("string"))
    }

    pub fn u32(&self) -> U32Field {
        U32Field::new(self.0.push("u32"))
    }

    pub fn u64(&self) -> U64Field {
        U64Field::new(self.0.push("u64"))
    }
}
#[derive(Default)]
pub struct GlslBuildTargetAssetRecord(PropertyPath);

impl Field for GlslBuildTargetAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GlslBuildTargetAssetRecord(property_path)
    }
}

impl Record for GlslBuildTargetAssetRecord {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl GlslBuildTargetAssetRecord {
    pub fn entry_point(&self) -> StringField {
        StringField::new(self.0.push("entry_point"))
    }

    pub fn source_file(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("source_file"))
    }
}
#[derive(Default)]
pub struct GlslSourceFileAssetRecord(PropertyPath);

impl Field for GlslSourceFileAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GlslSourceFileAssetRecord(property_path)
    }
}

impl Record for GlslSourceFileAssetRecord {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl GlslSourceFileAssetRecord {
}
#[derive(Default)]
pub struct GlslSourceFileImportedDataRecord(PropertyPath);

impl Field for GlslSourceFileImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        GlslSourceFileImportedDataRecord(property_path)
    }
}

impl Record for GlslSourceFileImportedDataRecord {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl GlslSourceFileImportedDataRecord {
    pub fn code(&self) -> StringField {
        StringField::new(self.0.push("code"))
    }
}
#[derive(Default)]
pub struct GpuBufferAssetRecord(PropertyPath);

impl Field for GpuBufferAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuBufferAssetRecord(property_path)
    }
}

impl Record for GpuBufferAssetRecord {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl GpuBufferAssetRecord {
}
#[derive(Default)]
pub struct GpuBufferImportedDataRecord(PropertyPath);

impl Field for GpuBufferImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuBufferImportedDataRecord(property_path)
    }
}

impl Record for GpuBufferImportedDataRecord {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl GpuBufferImportedDataRecord {
    pub fn alignment(&self) -> U32Field {
        U32Field::new(self.0.push("alignment"))
    }

    pub fn data(&self) -> BytesField {
        BytesField::new(self.0.push("data"))
    }

    pub fn resource_type(&self) -> U32Field {
        U32Field::new(self.0.push("resource_type"))
    }
}
#[derive(Default)]
pub struct GpuImageAssetRecord(PropertyPath);

impl Field for GpuImageAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuImageAssetRecord(property_path)
    }
}

impl Record for GpuImageAssetRecord {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl GpuImageAssetRecord {
    pub fn compress(&self) -> BooleanField {
        BooleanField::new(self.0.push("compress"))
    }
}
#[derive(Default)]
pub struct GpuImageImportedDataRecord(PropertyPath);

impl Field for GpuImageImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuImageImportedDataRecord(property_path)
    }
}

impl Record for GpuImageImportedDataRecord {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl GpuImageImportedDataRecord {
    pub fn height(&self) -> U32Field {
        U32Field::new(self.0.push("height"))
    }

    pub fn image_bytes(&self) -> BytesField {
        BytesField::new(self.0.push("image_bytes"))
    }

    pub fn width(&self) -> U32Field {
        U32Field::new(self.0.push("width"))
    }
}
#[derive(Copy, Clone)]
pub enum MeshAdvBlendMethodEnum {
    Opaque,
    AlphaClip,
    AlphaBlend,
}

impl Enum for MeshAdvBlendMethodEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            MeshAdvBlendMethodEnum::Opaque => "Opaque",
            MeshAdvBlendMethodEnum::AlphaClip => "AlphaClip",
            MeshAdvBlendMethodEnum::AlphaBlend => "AlphaBlend",
        }
    }

    fn from_symbol_name(str: &str) -> Option<MeshAdvBlendMethodEnum> {
        match str {
            "Opaque" => Some(MeshAdvBlendMethodEnum::Opaque),
            "OPAQUE" => Some(MeshAdvBlendMethodEnum::Opaque),
            "AlphaClip" => Some(MeshAdvBlendMethodEnum::AlphaClip),
            "ALPHA_CLIP" => Some(MeshAdvBlendMethodEnum::AlphaClip),
            "AlphaBlend" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            "ALPHA_BLEND" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            "BLEND" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            _ => None,
        }
    }
}

impl MeshAdvBlendMethodEnum {
    pub fn schema_name() -> &'static str {
        "MeshAdvBlendMethod"
    }
}
#[derive(Copy, Clone)]
pub enum MeshAdvIndexTypeEnum {
    Uint16,
    Uint32,
}

impl Enum for MeshAdvIndexTypeEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            MeshAdvIndexTypeEnum::Uint16 => "Uint16",
            MeshAdvIndexTypeEnum::Uint32 => "Uint32",
        }
    }

    fn from_symbol_name(str: &str) -> Option<MeshAdvIndexTypeEnum> {
        match str {
            "Uint16" => Some(MeshAdvIndexTypeEnum::Uint16),
            "Uint32" => Some(MeshAdvIndexTypeEnum::Uint32),
            _ => None,
        }
    }
}

impl MeshAdvIndexTypeEnum {
    pub fn schema_name() -> &'static str {
        "MeshAdvIndexType"
    }
}
#[derive(Default)]
pub struct MeshAdvMaterialAssetRecord(PropertyPath);

impl Field for MeshAdvMaterialAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMaterialAssetRecord(property_path)
    }
}

impl Record for MeshAdvMaterialAssetRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl MeshAdvMaterialAssetRecord {
    pub fn alpha_threshold(&self) -> F32Field {
        F32Field::new(self.0.push("alpha_threshold"))
    }

    pub fn backface_culling(&self) -> BooleanField {
        BooleanField::new(self.0.push("backface_culling"))
    }

    pub fn base_color_factor(&self) -> Vec4Record {
        Vec4Record::new(self.0.push("base_color_factor"))
    }

    pub fn blend_method(&self) -> EnumField::<MeshAdvBlendMethodEnum> {
        EnumField::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"))
    }

    pub fn color_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("color_texture"))
    }

    pub fn color_texture_has_alpha_channel(&self) -> BooleanField {
        BooleanField::new(self.0.push("color_texture_has_alpha_channel"))
    }

    pub fn emissive_factor(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("emissive_factor"))
    }

    pub fn emissive_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("emissive_texture"))
    }

    pub fn metallic_factor(&self) -> F32Field {
        F32Field::new(self.0.push("metallic_factor"))
    }

    pub fn metallic_roughness_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("metallic_roughness_texture"))
    }

    pub fn normal_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("normal_texture"))
    }

    pub fn normal_texture_scale(&self) -> F32Field {
        F32Field::new(self.0.push("normal_texture_scale"))
    }

    pub fn roughness_factor(&self) -> F32Field {
        F32Field::new(self.0.push("roughness_factor"))
    }

    pub fn shadow_method(&self) -> EnumField::<MeshAdvShadowMethodEnum> {
        EnumField::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"))
    }
}
#[derive(Default)]
pub struct MeshAdvMaterialImportedDataRecord(PropertyPath);

impl Field for MeshAdvMaterialImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMaterialImportedDataRecord(property_path)
    }
}

impl Record for MeshAdvMaterialImportedDataRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl MeshAdvMaterialImportedDataRecord {
}
#[derive(Default)]
pub struct MeshAdvMeshAssetRecord(PropertyPath);

impl Field for MeshAdvMeshAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshAssetRecord(property_path)
    }
}

impl Record for MeshAdvMeshAssetRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl MeshAdvMeshAssetRecord {
    pub fn material_slots(&self) -> DynamicArrayField::<AssetRefField> {
        DynamicArrayField::<AssetRefField>::new(self.0.push("material_slots"))
    }
}
#[derive(Default)]
pub struct MeshAdvMeshImportedDataRecord(PropertyPath);

impl Field for MeshAdvMeshImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshImportedDataRecord(property_path)
    }
}

impl Record for MeshAdvMeshImportedDataRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl MeshAdvMeshImportedDataRecord {
    pub fn mesh_parts(&self) -> DynamicArrayField::<MeshAdvMeshImportedDataMeshPartRecord> {
        DynamicArrayField::<MeshAdvMeshImportedDataMeshPartRecord>::new(self.0.push("mesh_parts"))
    }
}
#[derive(Default)]
pub struct MeshAdvMeshImportedDataMeshPartRecord(PropertyPath);

impl Field for MeshAdvMeshImportedDataMeshPartRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshImportedDataMeshPartRecord(property_path)
    }
}

impl Record for MeshAdvMeshImportedDataMeshPartRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl MeshAdvMeshImportedDataMeshPartRecord {
    pub fn indices(&self) -> BytesField {
        BytesField::new(self.0.push("indices"))
    }

    pub fn material_index(&self) -> U32Field {
        U32Field::new(self.0.push("material_index"))
    }

    pub fn normals(&self) -> BytesField {
        BytesField::new(self.0.push("normals"))
    }

    pub fn positions(&self) -> BytesField {
        BytesField::new(self.0.push("positions"))
    }

    pub fn texture_coordinates(&self) -> BytesField {
        BytesField::new(self.0.push("texture_coordinates"))
    }
}
#[derive(Copy, Clone)]
pub enum MeshAdvShadowMethodEnum {
    None,
    Opaque,
}

impl Enum for MeshAdvShadowMethodEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            MeshAdvShadowMethodEnum::None => "None",
            MeshAdvShadowMethodEnum::Opaque => "Opaque",
        }
    }

    fn from_symbol_name(str: &str) -> Option<MeshAdvShadowMethodEnum> {
        match str {
            "None" => Some(MeshAdvShadowMethodEnum::None),
            "NONE" => Some(MeshAdvShadowMethodEnum::None),
            "Opaque" => Some(MeshAdvShadowMethodEnum::Opaque),
            "OPAQUE" => Some(MeshAdvShadowMethodEnum::Opaque),
            _ => None,
        }
    }
}

impl MeshAdvShadowMethodEnum {
    pub fn schema_name() -> &'static str {
        "MeshAdvShadowMethod"
    }
}
#[derive(Default)]
pub struct TransformRecord(PropertyPath);

impl Field for TransformRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRecord(property_path)
    }
}

impl Record for TransformRecord {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl TransformRecord {
    pub fn all_fields(&self) -> AllFieldsRecord {
        AllFieldsRecord::new(self.0.push("all_fields"))
    }

    pub fn position(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("position"))
    }

    pub fn rotation(&self) -> Vec4Record {
        Vec4Record::new(self.0.push("rotation"))
    }

    pub fn scale(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("scale"))
    }
}
#[derive(Default)]
pub struct TransformRefRecord(PropertyPath);

impl Field for TransformRefRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRefRecord(property_path)
    }
}

impl Record for TransformRefRecord {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl TransformRefRecord {
    pub fn transform(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("transform"))
    }
}
#[derive(Default)]
pub struct Vec3Record(PropertyPath);

impl Field for Vec3Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec3Record(property_path)
    }
}

impl Record for Vec3Record {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl Vec3Record {
    pub fn x(&self) -> F32Field {
        F32Field::new(self.0.push("x"))
    }

    pub fn y(&self) -> F32Field {
        F32Field::new(self.0.push("y"))
    }

    pub fn z(&self) -> F32Field {
        F32Field::new(self.0.push("z"))
    }
}
#[derive(Default)]
pub struct Vec4Record(PropertyPath);

impl Field for Vec4Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec4Record(property_path)
    }
}

impl Record for Vec4Record {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl Vec4Record {
    pub fn w(&self) -> F32Field {
        F32Field::new(self.0.push("w"))
    }

    pub fn x(&self) -> F32Field {
        F32Field::new(self.0.push("x"))
    }

    pub fn y(&self) -> F32Field {
        F32Field::new(self.0.push("y"))
    }

    pub fn z(&self) -> F32Field {
        F32Field::new(self.0.push("z"))
    }
}

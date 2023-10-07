pub use super::*;
use std::path::{Path, PathBuf};

use demo_types::blender_material::*;
use hydrate_model::value::ValueEnum;
use hydrate_model::{BooleanField, BuiltObjectMetadata, DataContainer, DataContainerMut, DataSet, DataSetError, DataSetResult, DataSetView, DataSetViewMut, EditorModel, Enum, EnumField, F32Field, Field, HashMap, ObjectId, ObjectLocation, ObjectName, PropertyPath, SchemaDefType, SchemaLinker, SchemaSet, SingleObject, StringField, Value};
use hydrate_pipeline::{AssetPlugin, Builder, BuilderRegistry, BuiltAsset, ImportedImportable, Importer, ImporterRegistry, ScannedImportable};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use demo_types::simple_data_gen_from_schema::{Vec3Record, Vec4Record};
//use gltf::Mesh;

// Import this data to be "Default" values?
// - Import overwrites any unchanged values?
// - Maybe UI to force "full" re-import?
// - Add concept of importable that populates an asset instead of an import, and can be dis-associated?
// - keep it simple: importer can overwrite asset data
// - imported asset can be converted to an in-engine edited asset?

#[derive(Serialize, Deserialize)]
struct MaterialJsonFileFormat {
    pub base_color_factor: [f32; 4], // default: 1,1,1,1
    pub emissive_factor: [f32; 3],   // default: 0,0,0
    pub metallic_factor: f32,        // default: 1,
    pub roughness_factor: f32,       // default: 1,
    pub normal_texture_scale: f32,   // default: 1

    #[serde(default)]
    pub color_texture: Option<String>,
    #[serde(default)]
    pub metallic_roughness_texture: Option<String>,
    #[serde(default)]
    pub normal_texture: Option<String>,
    #[serde(default)]
    pub emissive_texture: Option<String>,

    #[serde(default)]
    pub shadow_method: Option<String>,
    #[serde(default)]
    pub blend_method: Option<String>,
    #[serde(default)]
    pub alpha_threshold: Option<f32>,
    #[serde(default)]
    pub backface_culling: Option<bool>,
    #[serde(default)]
    pub color_texture_has_alpha_channel: bool,
}

pub struct BlenderMaterialAsset {}

impl BlenderMaterialAsset {
    pub fn schema_name() -> &'static str {
        "BlenderMaterialAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {})
            .unwrap();
    }
}


impl Into<MeshAdvBlendMethod> for MeshAdvBlendMethodEnum {
    fn into(self) -> MeshAdvBlendMethod {
        match self {
            MeshAdvBlendMethodEnum::Opaque => MeshAdvBlendMethod::Opaque,
            MeshAdvBlendMethodEnum::AlphaClip => MeshAdvBlendMethod::AlphaClip,
            MeshAdvBlendMethodEnum::AlphaBlend => MeshAdvBlendMethod::AlphaBlend,
        }
    }
}

impl Into<MeshAdvShadowMethod> for MeshAdvShadowMethodEnum {
    fn into(self) -> MeshAdvShadowMethod {
        match self {
            MeshAdvShadowMethodEnum::None => MeshAdvShadowMethod::None,
            MeshAdvShadowMethodEnum::Opaque => MeshAdvShadowMethod::Opaque,
        }
    }
}

/*
pub enum MeshAdvShadowMethodEnum {
    None,
    Opaque
}

impl Into<MeshAdvShadowMethod> for MeshAdvShadowMethodEnum {
    fn into(self) -> MeshAdvShadowMethod {
        match self {
            MeshAdvShadowMethodEnum::None => MeshAdvShadowMethod::None,
            MeshAdvShadowMethodEnum::Opaque => MeshAdvShadowMethod::Opaque,
        }
    }
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
            "Opaque" => Some(MeshAdvShadowMethodEnum::Opaque),
            _ => None,
        }
    }
}

pub enum MeshAdvBlendMethodEnum {
    Opaque,
    AlphaClip,
    AlphaBlend,
}

impl Into<MeshAdvBlendMethod> for MeshAdvBlendMethodEnum {
    fn into(self) -> MeshAdvBlendMethod {
        match self {
            MeshAdvBlendMethodEnum::Opaque => MeshAdvBlendMethod::Opaque,
            MeshAdvBlendMethodEnum::AlphaClip => MeshAdvBlendMethod::AlphaClip,
            MeshAdvBlendMethodEnum::AlphaBlend => MeshAdvBlendMethod::AlphaBlend,
        }
    }
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
            "AlphaClip" => Some(MeshAdvBlendMethodEnum::AlphaClip),
            "AlphaBlend" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            _ => None,
        }
    }
}

// pub struct MeshAdvShadowMethodEnum(PropertyPath);
//
// impl Field for MeshAdvShadowMethodEnum {
//     fn new(property_path: PropertyPath) -> Self {
//         MeshAdvShadowMethodEnum(property_path)
//     }
// }
//
// impl MeshAdvShadowMethodEnum {
//     pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<MeshAdvShadowMethod> {
//         //data_set_view.schema_set().find_named_type("x").unwrap().as_enum().unwrap().value_from_string()
//         //data_set_view.schema_set().find_named_type("x").unwrap().as_enum().unwrap().value_from_string()
//
//         let e = data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_enum().unwrap();
//         MeshAdvShadowMethod::from_str(e.symbol_name()).ok_or(DataSetError::UnexpectedEnumSymbol)
//
//     }
//
//     pub fn set(&self, data_set_view: &mut DataSetViewMut, value: MeshAdvShadowMethod) -> DataSetResult<()> {
//         data_set_view.set_property_override(self.0.path(), Value::Enum(ValueEnum::new(value.str().to_string())))
//     }
// }


#[derive(Default)]
pub struct BlenderMaterialImportedDataRecord(PropertyPath);

impl BlenderMaterialImportedDataRecord {
    pub fn base_color_factor(&self) -> Vec4Record { Vec4Record::new(self.0.push("base_color_factor")) }
    pub fn emissive_factor(&self) -> Vec3Record { Vec3Record::new(self.0.push("emissive_factor")) }
    pub fn metallic_factor(&self) -> F32Field { F32Field::new(self.0.push("metallic_factor")) }
    pub fn roughness_factor(&self) -> F32Field { F32Field::new(self.0.push("roughness_factor")) }
    pub fn normal_texture_scale(&self) -> F32Field { F32Field::new(self.0.push("normal_texture_scale")) }

    pub fn color_texture(&self) -> StringField { StringField::new(self.0.push("color_texture")) }
    pub fn metallic_roughness_texture(&self) -> StringField { StringField::new(self.0.push("metallic_roughness_texture")) }
    pub fn normal_texture(&self) -> StringField { StringField::new(self.0.push("normal_texture")) }
    pub fn emissive_texture(&self) -> StringField { StringField::new(self.0.push("emissive_texture")) }

    pub fn shadow_method(&self) -> EnumField<MeshAdvShadowMethodEnum> { EnumField::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method")) }
    pub fn blend_method(&self) -> EnumField<MeshAdvBlendMethodEnum> { EnumField::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method")) }
    pub fn alpha_threshold(&self) -> F32Field { F32Field::new(self.0.push("alpha_threshold")) }
    pub fn backface_culling(&self) -> BooleanField { BooleanField::new(self.0.push("backface_culling")) }
    pub fn color_texture_has_alpha_channel(&self) -> BooleanField { BooleanField::new(self.0.push("color_texture_has_alpha_channel")) }
}
*/







#[derive(Default)]
struct BlenderMaterialImportedDataRecord(PropertyPath);

impl Field for BlenderMaterialImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        BlenderMaterialImportedDataRecord(property_path)
    }
}

impl BlenderMaterialImportedDataRecord {
    fn base_color_factor(&self) -> Vec4Record {
        Vec4Record::new(self.0.push("base_color_factor"))
    }

    fn emissive_factor(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("emissive_factor"))
    }

    fn metallic_factor(&self) -> F32Field {
        F32Field::new(self.0.push("metallic_factor"))
    }

    fn roughness_factor(&self) -> F32Field {
        F32Field::new(self.0.push("roughness_factor"))
    }

    fn normal_texture_scale(&self) -> F32Field {
        F32Field::new(self.0.push("normal_texture_scale"))
    }

    fn color_texture(&self) -> StringField {
        StringField::new(self.0.push("color_texture"))
    }

    fn metallic_roughness_texture(&self) -> StringField {
        StringField::new(self.0.push("metallic_roughness_texture"))
    }

    fn normal_texture(&self) -> StringField {
        StringField::new(self.0.push("normal_texture"))
    }

    fn emissive_texture(&self) -> StringField {
        StringField::new(self.0.push("emissive_texture"))
    }

    fn shadow_method(&self) -> EnumField::<MeshAdvShadowMethodEnum> {
        EnumField::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"))
    }

    fn blend_method(&self) -> EnumField::<MeshAdvBlendMethodEnum> {
        EnumField::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"))
    }

    fn alpha_threshold(&self) -> F32Field {
        F32Field::new(self.0.push("alpha_threshold"))
    }

    fn backface_culling(&self) -> BooleanField {
        BooleanField::new(self.0.push("backface_culling"))
    }

    fn color_texture_has_alpha_channel(&self) -> BooleanField {
        BooleanField::new(self.0.push("color_texture_has_alpha_channel"))
    }
}




enum MeshAdvBlendMethodEnum {
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
            "AlphaClip" => Some(MeshAdvBlendMethodEnum::AlphaClip),
            "AlphaBlend" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            _ => None,
        }
    }
}
enum MeshAdvShadowMethodEnum {
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
            "Opaque" => Some(MeshAdvShadowMethodEnum::Opaque),
            _ => None,
        }
    }
}













pub struct BlenderMaterialImportedData {}

impl BlenderMaterialImportedData {
    pub fn schema_name() -> &'static str {
        "BlenderMaterialImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        // linker
        //     .register_enum_type("MeshAdvShadowMethod", |x| {
        //         x.add_symbol("None", 0);
        //         x.add_symbol("Opaque", 1);
        //     })
        //     .unwrap();
        //
        // linker
        //     .register_enum_type("MeshAdvBlendMethod", |x| {
        //         x.add_symbol("Opaque", 0);
        //         x.add_symbol("AlphaClip", 1);
        //         x.add_symbol("AlphaBlend", 2);
        //     })
        //     .unwrap();
        //
        // linker
        //     .register_record_type(Self::schema_name(), |x| {
        //         x.add_struct("base_color_factor", "Vec4");
        //         x.add_struct("emissive_factor", "Vec3");
        //         x.add_f32("metallic_factor");
        //         x.add_f32("roughness_factor");
        //         x.add_f32("normal_texture_scale");
        //
        //         x.add_string("color_texture");
        //         x.add_string("metallic_roughness_texture");
        //         x.add_string("normal_texture");
        //         x.add_string("emissive_texture");
        //
        //         x.add_enum("shadow_method", "MeshAdvShadowMethod");
        //         x.add_enum("blend_method", "MeshAdvBlendMethod");
        //         x.add_f32("alpha_threshold");
        //         x.add_boolean("backface_culling");
        //         x.add_boolean("color_texture_has_alpha_channel");
        //     })
        //     .unwrap();
    }
}

// #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub enum MeshAdvShadowMethod {
//     None,
//     Opaque,
//     //AlphaClip,
//     //AlphaStochastic,
// }
//
// #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub enum MeshAdvBlendMethod {
//     Opaque,
//     AlphaClip,
//     //AlphaStochastic,
//     AlphaBlend,
// }
//
// // This is non-texture data associated with the material. Must convert to
// // MeshMaterialDataShaderParam to bind to a shader uniform
// #[derive(Serialize, Deserialize, Clone, Debug)]
// #[repr(C)]
// pub struct MeshAdvMaterialData {
//     // Using f32 arrays for serde support
//     pub base_color_factor: [f32; 4], // default: 1,1,1,1
//     pub emissive_factor: [f32; 3],   // default: 0,0,0
//     pub metallic_factor: f32,        //default: 1,
//     pub roughness_factor: f32,       // default: 1,
//     pub normal_texture_scale: f32,   // default: 1
//
//     pub has_base_color_texture: bool,
//     pub base_color_texture_has_alpha_channel: bool,
//     pub has_metallic_roughness_texture: bool,
//     pub has_normal_texture: bool,
//     pub has_emissive_texture: bool,
//
//     pub shadow_method: MeshAdvShadowMethod,
//     pub blend_method: MeshAdvBlendMethod,
//     pub alpha_threshold: f32,
//     pub backface_culling: bool,
// }
//
// impl Default for MeshAdvMaterialData {
//     fn default() -> Self {
//         MeshAdvMaterialData {
//             base_color_factor: [1.0, 1.0, 1.0, 1.0],
//             emissive_factor: [0.0, 0.0, 0.0],
//             metallic_factor: 1.0,
//             roughness_factor: 1.0,
//             normal_texture_scale: 1.0,
//             has_base_color_texture: false,
//             base_color_texture_has_alpha_channel: false,
//             has_metallic_roughness_texture: false,
//             has_normal_texture: false,
//             has_emissive_texture: false,
//             shadow_method: MeshAdvShadowMethod::Opaque,
//             blend_method: MeshAdvBlendMethod::Opaque,
//             alpha_threshold: 0.5,
//             backface_culling: true,
//         }
//     }
// }

// #[derive(Serialize, Deserialize)]
// struct BlenderMaterialBuiltData {
//     //image_bytes: Vec<u8>,
//     //width: u32,
//     //height: u32,
// }

pub struct BlenderMaterialAssetPlugin;

impl AssetPlugin for BlenderMaterialAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistry,
        builder_registry: &mut BuilderRegistry,
    ) {
        BlenderMaterialAsset::register_schema(schema_linker);
        BlenderMaterialImportedData::register_schema(schema_linker);

        importer_registry.register_handler::<BlenderMaterialImporter>(schema_linker);
        builder_registry.register_handler::<BlenderMaterialBuilder>(schema_linker);
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "e76bab79-654a-476f-93b1-88cd5fee7d1f"]
pub struct BlenderMaterialImporter;

impl Importer for BlenderMaterialImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["blender_material"]
    }

    fn scan_file(
        &self,
        path: &Path,
        schema_set: &SchemaSet,
    ) -> Vec<ScannedImportable> {
        let asset_type = schema_set
            .find_named_type(BlenderMaterialAsset::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();
        vec![ScannedImportable {
            name: None,
            asset_type,
            file_references: Default::default(),
        }]
    }

    //fn create_default_asset(&self, editor_model: &mut EditorModel, object_name: ObjectName, object_location: ObjectLocation) -> ObjectId {
    //    let schema_record = editor_model.root_edit_context_mut().schema_set().find_named_type(BlenderMaterialAsset::schema_name()).unwrap().as_record().unwrap().clone();
    //    editor_model.root_edit_context_mut().new_object(&object_name, &object_location, &schema_record)
    //}

    fn import_file(
        &self,
        path: &Path,
        object_ids: &HashMap<Option<String>, ObjectId>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let json_str = std::fs::read_to_string(path).unwrap();
        let json_data: MaterialJsonFileFormat = serde_json::from_str(&json_str).unwrap();

        let shadow_method_enum_type = schema
            .find_named_type("MeshAdvShadowMethod")
            .unwrap()
            .as_enum()
            .unwrap();
        let blend_method_enum_type = schema
            .find_named_type("MeshAdvBlendMethod")
            .unwrap()
            .as_enum()
            .unwrap();

        //let shadow_method_str = json_data.shadow_method;
        let shadow_method = if let Some(shadow_method_str) = &json_data.shadow_method.as_ref() {
            Value::enum_value_from_string(shadow_method_enum_type, shadow_method_str)
        } else {
            Value::enum_value_from_string(shadow_method_enum_type, "Opaque")
        }
        .unwrap();

        //let blend_method_str = json_data.blend_method;
        let blend_method = if let Some(blend_method_str) = &json_data.blend_method.as_ref() {
            Value::enum_value_from_string(blend_method_enum_type, blend_method_str)
        } else {
            Value::enum_value_from_string(blend_method_enum_type, "Opaque")
        }
        .unwrap();

        //
        // Store in an object
        //
        let image_imported_data_schema = schema
            .find_named_type(BlenderMaterialImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let mut import_object = SingleObject::new(image_imported_data_schema);

        let x = BlenderMaterialImportedDataRecord::default();
        {
            let mut data_container = DataContainerMut::new_single_object(&mut import_object, schema);
            x.base_color_factor().x().set(&mut data_container, json_data.base_color_factor[0]).unwrap();
            x.base_color_factor().y().set(&mut data_container, json_data.base_color_factor[1]).unwrap();
            x.base_color_factor().z().set(&mut data_container, json_data.base_color_factor[2]).unwrap();
        }

        //let data_set_view = DataSetView::new()
        //x.base_color_factor().x().get(data_set_view);


        import_object.set_property_override(
            schema,
            "base_color_factor.x",
            Value::F32(json_data.base_color_factor[0]),
        );
        import_object.set_property_override(
            schema,
            "base_color_factor.y",
            Value::F32(json_data.base_color_factor[1]),
        );
        import_object.set_property_override(
            schema,
            "base_color_factor.z",
            Value::F32(json_data.base_color_factor[2]),
        );
        import_object.set_property_override(
            schema,
            "base_color_factor.w",
            Value::F32(json_data.base_color_factor[3]),
        );

        import_object.set_property_override(
            schema,
            "emissive_factor.x",
            Value::F32(json_data.emissive_factor[0]),
        );
        import_object.set_property_override(
            schema,
            "emissive_factor.y",
            Value::F32(json_data.emissive_factor[1]),
        );
        import_object.set_property_override(
            schema,
            "emissive_factor.z",
            Value::F32(json_data.emissive_factor[2]),
        );

        import_object.set_property_override(
            schema,
            "metallic_factor",
            Value::F32(json_data.metallic_factor),
        );
        import_object.set_property_override(
            schema,
            "roughness_factor",
            Value::F32(json_data.roughness_factor),
        );
        import_object.set_property_override(
            schema,
            "normal_texture_scale",
            Value::F32(json_data.normal_texture_scale),
        );

        import_object.set_property_override(
            schema,
            "color_texture",
            Value::String(json_data.color_texture.unwrap_or_default()),
        );
        import_object.set_property_override(
            schema,
            "metallic_roughness_texture",
            Value::String(json_data.metallic_roughness_texture.unwrap_or_default()),
        );
        import_object.set_property_override(
            schema,
            "normal_texture",
            Value::String(json_data.normal_texture.unwrap_or_default()),
        );
        import_object.set_property_override(
            schema,
            "emissive_texture",
            Value::String(json_data.emissive_texture.unwrap_or_default()),
        );

        import_object.set_property_override(schema, "shadow_method", shadow_method);
        import_object.set_property_override(schema, "blend_method", blend_method);
        import_object.set_property_override(
            schema,
            "alpha_threshold",
            Value::F32(json_data.alpha_threshold.unwrap_or(0.5)),
        );
        import_object.set_property_override(
            schema,
            "backface_culling",
            Value::Boolean(json_data.backface_culling.unwrap_or(true)),
        );
        import_object.set_property_override(
            schema,
            "color_texture_has_alpha_channel",
            Value::Boolean(json_data.color_texture_has_alpha_channel),
        );

        //
        // x.add_string("blend_method");
        // x.add_f32("alpha_threshold");
        // x.add_nullable("backface_culling");
        // x.add_boolean("color_texture_has_alpha_channel");

        //import_object.set_property_override(schema, "image_bytes", Value::Bytes(image_bytes));

        //
        // Return the created objects
        //
        let mut imported_objects = HashMap::default();
        imported_objects.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                data: import_object,
            },
        );
        imported_objects
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "02f17f4e-8df2-4b79-95cf-d2ee62e92a01"]
pub struct BlenderMaterialBuilder {}

impl Builder for BlenderMaterialBuilder {
    fn asset_type(&self) -> &'static str {
        BlenderMaterialAsset::schema_name()
    }

    fn build_dependencies(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Vec<ObjectId> {
        vec![asset_id]
    }

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
    ) -> BuiltAsset {
        //
        // Read asset properties
        //
        // let compressed = data_set
        //     .resolve_property(schema, asset_id, "compress")
        //     .unwrap()
        //     .as_boolean()
        //     .unwrap();

        //
        // Read imported data
        //
        let imported_data = &dependency_data[&asset_id];
        // let image_bytes = imported_data
        //     .resolve_property(schema, "image_bytes")
        //     .unwrap()
        //     .as_bytes()
        //     .unwrap()
        //     .clone();

        let base_color_factor_x = imported_data
            .resolve_property(schema, "base_color_factor.x")
            .unwrap()
            .as_f32()
            .unwrap();
        let base_color_factor_y = imported_data
            .resolve_property(schema, "base_color_factor.y")
            .unwrap()
            .as_f32()
            .unwrap();
        let base_color_factor_z = imported_data
            .resolve_property(schema, "base_color_factor.z")
            .unwrap()
            .as_f32()
            .unwrap();
        let base_color_factor_w = imported_data
            .resolve_property(schema, "base_color_factor.w")
            .unwrap()
            .as_f32()
            .unwrap();

        let emissive_factor_x = imported_data
            .resolve_property(schema, "emissive_factor.x")
            .unwrap()
            .as_f32()
            .unwrap();
        let emissive_factor_y = imported_data
            .resolve_property(schema, "emissive_factor.y")
            .unwrap()
            .as_f32()
            .unwrap();
        let emissive_factor_z = imported_data
            .resolve_property(schema, "emissive_factor.z")
            .unwrap()
            .as_f32()
            .unwrap();

        let metallic_factor = imported_data
            .resolve_property(schema, "metallic_factor")
            .unwrap()
            .as_f32()
            .unwrap();
        let roughness_factor = imported_data
            .resolve_property(schema, "roughness_factor")
            .unwrap()
            .as_f32()
            .unwrap();
        let normal_texture_scale = imported_data
            .resolve_property(schema, "normal_texture_scale")
            .unwrap()
            .as_f32()
            .unwrap();

        let color_texture = imported_data
            .resolve_property(schema, "color_texture")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();
        let metallic_roughness_texture = imported_data
            .resolve_property(schema, "metallic_roughness_texture")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();
        let normal_texture = imported_data
            .resolve_property(schema, "normal_texture")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();
        let emissive_texture = imported_data
            .resolve_property(schema, "emissive_texture")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();

        let shadow_method = imported_data
            .resolve_property(schema, "shadow_method")
            .unwrap()
            .as_enum()
            .unwrap()
            .clone();
        let blend_method = imported_data
            .resolve_property(schema, "blend_method")
            .unwrap()
            .as_enum()
            .unwrap()
            .clone();
        let alpha_threshold = imported_data
            .resolve_property(schema, "alpha_threshold")
            .unwrap()
            .as_f32()
            .unwrap()
            .clone();
        let backface_culling = imported_data
            .resolve_property(schema, "backface_culling")
            .unwrap()
            .as_boolean()
            .unwrap();
        let color_texture_has_alpha_channel = imported_data
            .resolve_property(schema, "color_texture_has_alpha_channel")
            .unwrap()
            .as_boolean()
            .unwrap();



        //let data_set_view = DataSetView::new()
        let data_set_view = DataContainer::new_single_object(imported_data, schema);
        let value = BlenderMaterialImportedDataRecord::default();
        let alpha_threshold = value.alpha_threshold().get(&data_set_view).unwrap();
        let shadow_method = value.shadow_method().get(&data_set_view).unwrap();
        let blend_method = value.blend_method().get(&data_set_view).unwrap();


        //
        // Do some processing
        //

        //
        // Store the result
        //
        // let shadow_method = match shadow_method.symbol_name() {
        //     "None" => MeshAdvShadowMethod::None,
        //     "Opaque" => MeshAdvShadowMethod::Opaque,
        //     v @ _ => panic!("unknown shadow method {}", v),
        // };

        // let blend_method = match blend_method.symbol_name() {
        //     "Opaque" => MeshAdvBlendMethod::Opaque,
        //     "AlphaClip" => MeshAdvBlendMethod::AlphaClip,
        //     "AlphaBlend" => MeshAdvBlendMethod::AlphaBlend,
        //     v @ _ => panic!("unknown blend method {}", v),
        // };

        let processed_data = MeshAdvMaterialData {
            base_color_factor: [
                base_color_factor_x,
                base_color_factor_y,
                base_color_factor_z,
                base_color_factor_w,
            ],
            emissive_factor: [emissive_factor_x, emissive_factor_y, emissive_factor_z],
            metallic_factor,
            roughness_factor,
            normal_texture_scale,
            has_base_color_texture: !color_texture.is_empty(),
            base_color_texture_has_alpha_channel: color_texture_has_alpha_channel,
            has_metallic_roughness_texture: !metallic_roughness_texture.is_empty(),
            has_normal_texture: !normal_texture.is_empty(),
            has_emissive_texture: !emissive_texture.is_empty(),
            shadow_method: shadow_method.into(),
            blend_method: blend_method.into(),
            alpha_threshold,
            backface_culling,
        };

        let serialized = bincode::serialize(&processed_data).unwrap();
        BuiltAsset {
            metadata: BuiltObjectMetadata {
                dependencies: vec![],
                subresource_count: 0,
                asset_type: uuid::Uuid::from_bytes(processed_data.uuid())
            },
            data: serialized
        }
    }
}

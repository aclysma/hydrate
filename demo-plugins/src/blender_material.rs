pub use super::*;
use std::path::{Path, PathBuf};

use demo_types::blender_material::*;
use hydrate_model::value::ValueEnum;
use hydrate_model::{
    DataSet, EditorModel, HashMap, ObjectId, ObjectLocation, ObjectName, SchemaDefType,
    SchemaLinker, SchemaSet, SingleObject, Value,
};
use hydrate_pipeline::{
    AssetPlugin, Builder, BuilderRegistry, ImportedImportable, Importer, ImporterRegistry,
    ScannedImportable,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

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

pub struct BlenderMaterialImportedData {}

impl BlenderMaterialImportedData {
    pub fn schema_name() -> &'static str {
        "BlenderMaterialImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_enum_type("MeshAdvShadowMethod", |x| {
                x.add_symbol("None", 0);
                x.add_symbol("Opaque", 1);
            })
            .unwrap();

        linker
            .register_enum_type("MeshAdvBlendMethod", |x| {
                x.add_symbol("Opaque", 0);
                x.add_symbol("AlphaClip", 1);
                x.add_symbol("AlphaBlend", 2);
            })
            .unwrap();

        linker
            .register_record_type(Self::schema_name(), |x| {
                x.add_struct("base_color_factor", "Vec4");
                x.add_struct("emissive_factor", "Vec3");
                x.add_f32("metallic_factor");
                x.add_f32("roughness_factor");
                x.add_f32("normal_texture_scale");

                x.add_string("color_texture");
                x.add_string("metallic_roughness_texture");
                x.add_string("normal_texture");
                x.add_string("emissive_texture");

                x.add_enum("shadow_method", "MeshAdvShadowMethod");
                x.add_enum("blend_method", "MeshAdvBlendMethod");
                x.add_f32("alpha_threshold");
                x.add_boolean("backface_culling");
                x.add_boolean("color_texture_has_alpha_channel");
            })
            .unwrap();
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
            shadow_method_enum_type.value_from_string(shadow_method_str)
        } else {
            shadow_method_enum_type.value_from_string("Opaque")
        }
        .unwrap();

        //let blend_method_str = json_data.blend_method;
        let blend_method = if let Some(blend_method_str) = &json_data.shadow_method.as_ref() {
            blend_method_enum_type.value_from_string(blend_method_str)
        } else {
            blend_method_enum_type.value_from_string("Opaque")
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
    ) -> Vec<u8> {
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

        //
        // Do some processing
        //

        //
        // Store the result
        //
        let shadow_method = match shadow_method.symbol_name() {
            "None" => MeshAdvShadowMethod::None,
            "Opaque" => MeshAdvShadowMethod::Opaque,
            v @ _ => panic!("unknown shadow method {}", v),
        };

        let blend_method = match blend_method.symbol_name() {
            "Opaque" => MeshAdvBlendMethod::Opaque,
            "AlphaClip" => MeshAdvBlendMethod::AlphaClip,
            "AlphaBlend" => MeshAdvBlendMethod::AlphaBlend,
            v @ _ => panic!("unknown blend method {}", v),
        };

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
            shadow_method,
            blend_method,
            alpha_threshold,
            backface_culling,
        };

        let serialized = bincode::serialize(&processed_data).unwrap();
        serialized
    }
}

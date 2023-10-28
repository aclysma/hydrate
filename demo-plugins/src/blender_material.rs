pub use super::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use demo_types::mesh_adv::*;
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::{BuilderRegistryBuilder, DataContainer, DataContainerMut, DataSet, Enum, HashMap, ImportableObject, ImporterId, ImporterRegistryBuilder, JobProcessorRegistryBuilder, ObjectId, ObjectRefField, Record, ReferencedSourceFile, SchemaLinker, SchemaSet, SingleObject};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuiltAsset};
use hydrate_model::pipeline::{ImportedImportable, ScannedImportable, Importer};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use uuid::Uuid;

use super::generated::{MeshAdvMaterialImportedDataRecord, MeshAdvMaterialAssetRecord, MeshAdvBlendMethodEnum, MeshAdvShadowMethodEnum};

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
            .find_named_type(MeshAdvMaterialAssetRecord::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let json_str = std::fs::read_to_string(path).unwrap();
        let json_data: MaterialJsonFileFormat = serde_json::from_str(&json_str).unwrap();

        let mut file_references: Vec<ReferencedSourceFile> = Default::default();

        fn try_add_file_reference<T: TypeUuid>(file_references: &mut Vec<ReferencedSourceFile>, path_as_string: &Option<String>) {
            let importer_image_id = ImporterId(Uuid::from_bytes(T::UUID));
            if let Some(path_as_string) = path_as_string {
                file_references.push(ReferencedSourceFile {
                    importer_id: importer_image_id,
                    path: PathBuf::from_str(path_as_string).unwrap(),
                })
            }
        }

        try_add_file_reference::<GpuImageImporter>(&mut file_references, &json_data.color_texture);
        try_add_file_reference::<GpuImageImporter>(&mut file_references, &json_data.metallic_roughness_texture);
        try_add_file_reference::<GpuImageImporter>(&mut file_references, &json_data.normal_texture);
        try_add_file_reference::<GpuImageImporter>(&mut file_references, &json_data.emissive_texture);

        vec![ScannedImportable {
            name: None,
            asset_type,
            file_references,
        }]
    }

    fn import_file(
        &self,
        path: &Path,
        importable_objects: &HashMap<Option<String>, ImportableObject>,
        schema_set: &SchemaSet,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let json_str = std::fs::read_to_string(path).unwrap();
        let json_data: MaterialJsonFileFormat = serde_json::from_str(&json_str).unwrap();

        //
        // Parse strings to enums or provide default value if they weren't specified
        //
        let shadow_method = if let Some(shadow_method_string) = &json_data.shadow_method {
            //TODO: This relies on input json and code matching perfectly, ideally we would search schema type for aliases
            //println!("find MeshAdvShadowMethodEnum {:?}", shadow_method_string);
            MeshAdvShadowMethodEnum::from_symbol_name(shadow_method_string.as_str()).unwrap()
        } else {
            MeshAdvShadowMethodEnum::None
        };

        let blend_method = if let Some(blend_method_string) = &json_data.blend_method {
            //TODO: This relies on input json and code matching perfectly, ideally we would search schema type for alias
            //println!("find MeshAdvBlendMethodEnum {:?}", blend_method_string);
            MeshAdvBlendMethodEnum::from_symbol_name(blend_method_string.as_str()).unwrap()
        } else {
            MeshAdvBlendMethodEnum::Opaque
        };

        //
        // Create the default asset
        //
        let default_asset = {
            let mut default_asset_object = MeshAdvMaterialAssetRecord::new_single_object(schema_set).unwrap();
            let mut default_asset_data_container = DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
            let x = MeshAdvMaterialAssetRecord::default();
            x.base_color_factor().set_vec4(&mut default_asset_data_container, json_data.base_color_factor).unwrap();
            x.emissive_factor().set_vec3(&mut default_asset_data_container, json_data.emissive_factor).unwrap();
            x.metallic_factor().set(&mut default_asset_data_container, json_data.metallic_factor).unwrap();
            x.roughness_factor().set(&mut default_asset_data_container, json_data.roughness_factor).unwrap();
            x.normal_texture_scale().set(&mut default_asset_data_container, json_data.normal_texture_scale).unwrap();

            fn try_find_file_reference(
                importable_objects: &HashMap<Option<String>, ImportableObject>,
                data_container: &mut DataContainerMut,
                ref_field: ObjectRefField,
                path_as_string: &Option<String>
            ) {
                if let Some(path_as_string) = path_as_string {
                    if let Some(referenced_object_id) = importable_objects.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(path_as_string).unwrap()) {
                        ref_field.set(data_container, *referenced_object_id).unwrap();
                    }
                }
            }

            try_find_file_reference(&importable_objects, &mut default_asset_data_container, x.color_texture(), &json_data.color_texture);
            try_find_file_reference(&importable_objects, &mut default_asset_data_container, x.metallic_roughness_texture(), &json_data.metallic_roughness_texture);
            try_find_file_reference(&importable_objects, &mut default_asset_data_container, x.normal_texture(), &json_data.normal_texture);
            try_find_file_reference(&importable_objects, &mut default_asset_data_container, x.emissive_texture(), &json_data.emissive_texture);

            x.shadow_method().set(&mut default_asset_data_container, shadow_method).unwrap();
            x.blend_method().set(&mut default_asset_data_container, blend_method).unwrap();
            x.alpha_threshold().set(&mut default_asset_data_container, json_data.alpha_threshold.unwrap_or(0.5)).unwrap();
            x.backface_culling().set(&mut default_asset_data_container, json_data.backface_culling.unwrap_or(true)).unwrap();
            //TODO: Does this incorrectly write older enum string names when code is older than schema file?
            x.color_texture_has_alpha_channel().set(&mut default_asset_data_container, json_data.color_texture_has_alpha_channel).unwrap();
            default_asset_object
        };

        //
        // Return the created objects
        //
        let mut imported_objects = HashMap::default();
        imported_objects.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                import_data: None,
                default_asset: Some(default_asset)
            },
        );
        imported_objects
    }
}

pub struct BlenderMaterialAssetPlugin;

impl AssetPlugin for BlenderMaterialAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<BlenderMaterialImporter>(schema_linker);
    }
}





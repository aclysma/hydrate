pub use super::*;
use std::path::PathBuf;

use hydrate_data::{AssetRefFieldOwned, DataSetError, RecordOwned};
use hydrate_model::pipeline::{AssetPlugin, ImportContext, ScanContext};
use hydrate_model::pipeline::{ImportedImportable, Importer, ScannedImportable};
use hydrate_pipeline::{
    BuilderRegistryBuilder, Enum, HashMap, ImportableAsset, ImporterId, ImporterRegistryBuilder,
    JobProcessorRegistryBuilder, PipelineResult, RecordAccessor, ReferencedSourceFile,
    SchemaLinker,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use uuid::Uuid;

use super::generated::{
    MeshAdvBlendMethodEnum, MeshAdvMaterialAssetAccessor, MeshAdvMaterialAssetOwned,
    MeshAdvShadowMethodEnum,
};

#[derive(Serialize, Deserialize)]
struct MaterialJsonFileFormat {
    pub base_color_factor: [f32; 4], // default: 1,1,1,1
    pub emissive_factor: [f32; 3],   // default: 0,0,0
    pub metallic_factor: f32,        // default: 1,
    pub roughness_factor: f32,       // default: 1,
    pub normal_texture_scale: f32,   // default: 1

    #[serde(default)]
    pub color_texture: Option<PathBuf>,
    #[serde(default)]
    pub metallic_roughness_texture: Option<PathBuf>,
    #[serde(default)]
    pub normal_texture: Option<PathBuf>,
    #[serde(default)]
    pub emissive_texture: Option<PathBuf>,

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
        context: ScanContext,
    ) -> PipelineResult<Vec<ScannedImportable>> {
        let asset_type = context
            .schema_set
            .find_named_type(MeshAdvMaterialAssetAccessor::schema_name())?
            .as_record()?
            .clone();

        let json_str = std::fs::read_to_string(context.path)?;
        let json_data: MaterialJsonFileFormat = {
            profiling::scope!("serde_json::from_str");
            serde_json::from_str(&json_str)?
        };

        let mut file_references: Vec<ReferencedSourceFile> = Default::default();

        fn try_add_file_reference<T: TypeUuid>(
            file_references: &mut Vec<ReferencedSourceFile>,
            path_as_string: &Option<PathBuf>,
        ) {
            let importer_image_id = ImporterId(Uuid::from_bytes(T::UUID));
            if let Some(path_as_string) = path_as_string {
                file_references.push(ReferencedSourceFile {
                    importer_id: importer_image_id,
                    path: path_as_string.clone(),
                })
            }
        }

        try_add_file_reference::<GpuImageImporter>(&mut file_references, &json_data.color_texture);
        try_add_file_reference::<GpuImageImporter>(
            &mut file_references,
            &json_data.metallic_roughness_texture,
        );
        try_add_file_reference::<GpuImageImporter>(&mut file_references, &json_data.normal_texture);
        try_add_file_reference::<GpuImageImporter>(
            &mut file_references,
            &json_data.emissive_texture,
        );

        Ok(vec![ScannedImportable {
            name: None,
            asset_type,
            file_references,
        }])
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<HashMap<Option<String>, ImportedImportable>> {
        //
        // Read the file
        //
        let json_str = std::fs::read_to_string(context.path)?;
        let json_data: MaterialJsonFileFormat = {
            profiling::scope!("serde_json::from_str");
            serde_json::from_str(&json_str)?
        };

        //
        // Parse strings to enums or provide default value if they weren't specified
        //
        let shadow_method = if let Some(shadow_method_string) = &json_data.shadow_method {
            //TODO: This relies on input json and code matching perfectly, ideally we would search schema type for aliases
            //println!("find MeshAdvShadowMethodEnum {:?}", shadow_method_string);
            MeshAdvShadowMethodEnum::from_symbol_name(shadow_method_string.as_str())
                .ok_or(DataSetError::UnexpectedEnumSymbol)?
        } else {
            MeshAdvShadowMethodEnum::None
        };

        let blend_method = if let Some(blend_method_string) = &json_data.blend_method {
            //TODO: This relies on input json and code matching perfectly, ideally we would search schema type for alias
            //println!("find MeshAdvBlendMethodEnum {:?}", blend_method_string);
            MeshAdvBlendMethodEnum::from_symbol_name(blend_method_string.as_str())
                .ok_or(DataSetError::UnexpectedEnumSymbol)?
        } else {
            MeshAdvBlendMethodEnum::Opaque
        };

        //
        // Create the default asset
        //
        let default_asset = MeshAdvMaterialAssetOwned::new_builder(context.schema_set);

        default_asset
            .base_color_factor()
            .set_vec4(json_data.base_color_factor)?;
        default_asset
            .emissive_factor()
            .set_vec3(json_data.emissive_factor)?;
        default_asset
            .metallic_factor()
            .set(json_data.metallic_factor)?;
        default_asset
            .roughness_factor()
            .set(json_data.roughness_factor)?;
        default_asset
            .normal_texture_scale()
            .set(json_data.normal_texture_scale)?;

        fn try_find_file_reference(
            importable_assets: &HashMap<Option<String>, ImportableAsset>,
            ref_field: AssetRefFieldOwned,
            path_as_string: &Option<PathBuf>,
        ) -> PipelineResult<()> {
            if let Some(path_as_string) = path_as_string {
                if let Some(referenced_asset_id) = importable_assets
                    .get(&None)
                    .ok_or("Path not found in import data")?
                    .referenced_paths
                    .get(path_as_string)
                {
                    ref_field.set(*referenced_asset_id)?;
                }
            }

            Ok(())
        }

        try_find_file_reference(
            &context.importable_assets,
            default_asset.color_texture(),
            &json_data.color_texture,
        )?;
        try_find_file_reference(
            &context.importable_assets,
            default_asset.metallic_roughness_texture(),
            &json_data.metallic_roughness_texture,
        )?;
        try_find_file_reference(
            &context.importable_assets,
            default_asset.normal_texture(),
            &json_data.normal_texture,
        )?;
        try_find_file_reference(
            &context.importable_assets,
            default_asset.emissive_texture(),
            &json_data.emissive_texture,
        )?;

        default_asset.shadow_method().set(shadow_method)?;
        default_asset.blend_method().set(blend_method)?;
        default_asset
            .alpha_threshold()
            .set(json_data.alpha_threshold.unwrap_or(0.5))?;
        default_asset
            .backface_culling()
            .set(json_data.backface_culling.unwrap_or(true))?;
        //TODO: Does this incorrectly write older enum string names when code is older than schema file?
        default_asset
            .color_texture_has_alpha_channel()
            .set(json_data.color_texture_has_alpha_channel)?;

        //
        // Return the created assets
        //
        let mut imported_assets = HashMap::default();
        imported_assets.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                import_data: None,
                default_asset: Some(default_asset.into_inner()?),
            },
        );
        Ok(imported_assets)
    }
}

pub struct BlenderMaterialAssetPlugin;

impl AssetPlugin for BlenderMaterialAssetPlugin {
    fn setup(
        _schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        _builder_registry: &mut BuilderRegistryBuilder,
        _job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<BlenderMaterialImporter>();
    }
}

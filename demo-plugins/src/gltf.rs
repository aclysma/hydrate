use super::generated::{
    GpuImageAssetRecord, GpuImageImportedDataRecord, MeshAdvMaterialAssetRecord,
    MeshAdvMeshAssetRecord, MeshAdvMeshImportedDataRecord,
};
use hydrate_data::{ImportableName, Record};
use hydrate_model::pipeline::Importer;
use hydrate_model::pipeline::{AssetPlugin, ImportContext, ScanContext};
use hydrate_pipeline::{AssetPluginSetupContext, HashMap, PipelineResult};
use type_uuid::TypeUuid;

fn name_or_index(
    prefix: &str,
    name: Option<&str>,
    index: usize,
) -> ImportableName {
    if let Some(name) = name {
        ImportableName::new(format!("{}_{}", prefix, name))
    } else {
        ImportableName::new(format!("{}_{}", prefix, index))
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "01d71c49-867c-4d96-ad16-7c08b6cbfaf9"]
pub struct GltfImporter;

impl Importer for GltfImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["gltf", "glb"]
    }

    fn scan_file(
        &self,
        context: ScanContext,
    ) -> PipelineResult<()> {
        let (doc, _buffers, _images) =
            ::gltf::import(context.path).map_err(|e| format!("gltf_import() failed: {}", e))?;

        for (i, image) in doc.images().enumerate() {
            let name = name_or_index("image", image.name(), i);
            context.add_importable::<GpuImageAssetRecord>(name)?;
        }

        for (i, mesh) in doc.meshes().enumerate() {
            let name = name_or_index("mesh", mesh.name(), i);
            context.add_importable::<MeshAdvMeshAssetRecord>(name)?;
        }

        for (i, material) in doc.materials().enumerate() {
            let name = name_or_index("material", material.name(), i);
            context.add_importable::<MeshAdvMaterialAssetRecord>(name)?;
        }

        Ok(())
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<()> {
        //
        // Read the file
        //
        let (doc, _buffers, _images) =
            ::gltf::import(context.path).map_err(|e| format!("gltf_import() failed: {}", e))?;

        let mut image_index_to_object_id = HashMap::default();

        for (i, image) in doc.images().enumerate() {
            let name = name_or_index("image", image.name(), i);
            if let Some(importable_object) = context.asset_id_for_importable(&name) {
                //
                // Create import data
                //
                let import_data = GpuImageImportedDataRecord::new_builder(context.schema_set);
                // omitted for brevity

                //
                // Create the default asset
                //
                let asset_data = GpuImageAssetRecord::new_builder(context.schema_set);
                //omitted for brevity

                image_index_to_object_id.insert(image.index(), importable_object);
                context.add_importable(
                    name,
                    asset_data.into_inner()?,
                    Some(import_data.into_inner()?),
                );
            }
        }

        for (i, mesh) in doc.meshes().enumerate() {
            let name = name_or_index("mesh", mesh.name(), i);
            if context.should_import(&name) {
                //
                // Create import data
                //
                let import_data = MeshAdvMeshImportedDataRecord::new_builder(context.schema_set);

                //
                // Create the default asset
                //
                let asset_data = MeshAdvMeshAssetRecord::new_builder(context.schema_set);

                //
                // Return the created assets
                //
                context.add_importable(
                    name,
                    asset_data.into_inner()?,
                    Some(import_data.into_inner()?),
                );
            }
        }

        for (i, material) in doc.materials().enumerate() {
            let name = name_or_index("material", material.name(), i);
            if context.should_import(&name) {
                //
                // Create the default asset
                //
                let default_asset = MeshAdvMaterialAssetRecord::new_builder(context.schema_set);
                default_asset
                    .base_color_factor()
                    .set_vec4(material.pbr_metallic_roughness().base_color_factor())?;
                default_asset
                    .emissive_factor()
                    .set_vec3(material.emissive_factor())?;
                default_asset
                    .metallic_factor()
                    .set(material.pbr_metallic_roughness().metallic_factor())?;
                default_asset
                    .roughness_factor()
                    .set(material.pbr_metallic_roughness().roughness_factor())?;
                default_asset
                    .normal_texture_scale()
                    .set(material.normal_texture().map_or(1.0, |x| x.scale()))?;

                if let Some(texture) = material.pbr_metallic_roughness().base_color_texture() {
                    let texture_index = texture.texture().index();
                    let texture_object_id = image_index_to_object_id[&texture_index];
                    default_asset.color_texture().set(texture_object_id)?;
                }

                if let Some(texture) = material
                    .pbr_metallic_roughness()
                    .metallic_roughness_texture()
                {
                    let texture_index = texture.texture().index();
                    let texture_object_id = image_index_to_object_id[&texture_index];
                    default_asset
                        .metallic_roughness_texture()
                        .set(texture_object_id)?;
                }

                if let Some(texture) = material.normal_texture() {
                    let texture_index = texture.texture().index();
                    let texture_object_id = image_index_to_object_id[&texture_index];
                    default_asset.normal_texture().set(texture_object_id)?;
                }

                if let Some(texture) = material.emissive_texture() {
                    let texture_index = texture.texture().index();
                    let texture_object_id = image_index_to_object_id[&texture_index];
                    default_asset.emissive_texture().set(texture_object_id)?;
                }

                //x.shadow_method().set(&mut default_asset_data_container, shadow_method)?;
                //x.blend_method().set(&mut default_asset_data_container, blend_method)?;
                default_asset
                    .alpha_threshold()
                    .set(material.alpha_cutoff().unwrap_or(0.5))?;
                default_asset.backface_culling().set(false)?;
                //TODO: Does this incorrectly write older enum string names when code is older than schema file?
                default_asset.color_texture_has_alpha_channel().set(false)?;

                //
                // Return the created assets
                //
                context.add_importable(name, default_asset.into_inner()?, None);
            }
        }

        Ok(())
    }
}

pub struct GltfAssetPlugin;

impl AssetPlugin for GltfAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        context.importer_registry.register_handler::<GltfImporter>();
    }
}

pub use super::*;
use std::path::Path;

use super::generated::{
    MeshAdvMaterialAssetAccessor, MeshAdvMeshAssetAccessor, MeshAdvMeshImportedDataAccessor,
};
use hydrate_model::pipeline::{AssetPlugin, ImportContext, ImporterRegistry, ScanContext};
use hydrate_model::pipeline::{ImportedImportable, Importer, ScannedImportable};
use hydrate_pipeline::{
    BuilderRegistryBuilder, DataContainerRefMut, HashMap, ImportableAsset, ImporterRegistryBuilder,
    JobProcessorRegistryBuilder, RecordAccessor, SchemaLinker, SchemaSet,
};
use type_uuid::TypeUuid;

fn name_or_index(
    prefix: &str,
    name: Option<&str>,
    index: usize,
) -> String {
    if let Some(name) = name {
        format!("{}_{}", prefix, name)
    } else {
        format!("{}_{}", prefix, index)
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
    ) -> Vec<ScannedImportable> {
        let mesh_asset_type = context
            .schema_set
            .find_named_type(MeshAdvMeshAssetAccessor::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let material_asset_type = context
            .schema_set
            .find_named_type(MeshAdvMaterialAssetAccessor::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let (doc, _buffers, _images) = ::gltf::import(context.path).unwrap();

        let mut importables = Vec::default();

        for (i, mesh) in doc.meshes().enumerate() {
            let name = name_or_index("mesh", mesh.name(), i);

            importables.push(ScannedImportable {
                name: Some(name),
                asset_type: mesh_asset_type.clone(),
                file_references: Default::default(),
            });
        }

        for (i, material) in doc.materials().enumerate() {
            let name = name_or_index("material", material.name(), i);

            importables.push(ScannedImportable {
                name: Some(name),
                asset_type: material_asset_type.clone(),
                file_references: Default::default(),
            });
        }

        importables
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let (doc, _buffers, _images) = ::gltf::import(context.path).unwrap();

        let mut imported_assets = HashMap::default();

        for (i, mesh) in doc.meshes().enumerate() {
            let name = Some(name_or_index("mesh", mesh.name(), i));
            if context.importable_assets.contains_key(&name) {
                //
                // Create import data
                //
                let import_data = {
                    let import_object =
                        MeshAdvMeshImportedDataAccessor::new_single_object(context.schema_set)
                            .unwrap();
                    // let mut import_data_container =
                    //     DataContainerMut::new_single_object(&mut import_object, schema_set);
                    // let x = MeshAdvMeshImportedDataAccessor::default();
                    import_object
                };

                //
                // Create the default asset
                //

                let default_asset = {
                    let default_asset_object =
                        MeshAdvMeshAssetAccessor::new_single_object(context.schema_set).unwrap();
                    // let mut default_asset_data_container =
                    //     DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
                    // let x = MeshAdvMeshAssetAccessor::default();
                    default_asset_object
                };

                //
                // Return the created assets
                //
                imported_assets.insert(
                    name,
                    ImportedImportable {
                        file_references: Default::default(),
                        import_data: Some(import_data),
                        default_asset: Some(default_asset),
                    },
                );
            }
        }

        for (i, material) in doc.materials().enumerate() {
            let name = Some(name_or_index("material", material.name(), i));
            if context.importable_assets.contains_key(&name) {
                //
                // Create the default asset
                //

                let default_asset = {
                    let mut default_asset_object =
                        MeshAdvMaterialAssetAccessor::new_single_object(context.schema_set).unwrap();
                    let mut default_asset_data_container = DataContainerRefMut::from_single_object(
                        &mut default_asset_object,
                        context.schema_set,
                    );
                    let x = MeshAdvMaterialAssetAccessor::default();
                    x.base_color_factor()
                        .set_vec4(
                            &mut default_asset_data_container,
                            material.pbr_metallic_roughness().base_color_factor(),
                        )
                        .unwrap();
                    x.emissive_factor()
                        .set_vec3(
                            &mut default_asset_data_container,
                            material.emissive_factor(),
                        )
                        .unwrap();
                    x.metallic_factor()
                        .set(
                            &mut default_asset_data_container,
                            material.pbr_metallic_roughness().metallic_factor(),
                        )
                        .unwrap();
                    x.roughness_factor()
                        .set(
                            &mut default_asset_data_container,
                            material.pbr_metallic_roughness().roughness_factor(),
                        )
                        .unwrap();
                    x.normal_texture_scale()
                        .set(
                            &mut default_asset_data_container,
                            material.normal_texture().map_or(1.0, |x| x.scale()),
                        )
                        .unwrap();

                    //TODO: This needs to be updated to handle images in the GLTF or referenced externally

                    // x.color_texture().set(&mut default_asset_data_container, material.color_texture().unwrap_or_default()).unwrap();
                    // x.metallic_roughness_texture().set(&mut default_asset_data_container, material.metallic_roughness_texture().unwrap_or_default()).unwrap();
                    // x.normal_texture().set(&mut default_asset_data_container, material.normal_texture().unwrap_or_default()).unwrap();
                    // x.emissive_texture().set(&mut default_asset_data_container, material.emissive_texture().unwrap_or_default()).unwrap();

                    // if let Some(color_texture) = material.pbr_metallic_roughness().base_color_texture() {
                    //     if let Some(referenced_asset_id) = importable_assets.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&color_texture.).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_asset_id).unwrap();
                    //     }
                    // }
                    //
                    // if let Some(metallic_roughness_texture) = json_data.metallic_roughness_texture {
                    //     if let Some(referenced_asset_id) = importable_assets.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&metallic_roughness_texture).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_asset_id).unwrap();
                    //     }
                    // }
                    //
                    // if let Some(normal_texture) = json_data.normal_texture {
                    //     if let Some(referenced_asset_id) = importable_assets.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&normal_texture).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_asset_id).unwrap();
                    //     }
                    // }
                    //
                    // if let Some(emissive_texture) = json_data.emissive_texture {
                    //     if let Some(referenced_asset_id) = importable_assets.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&emissive_texture).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_asset_id).unwrap();
                    //     }
                    // }

                    //x.shadow_method().set(&mut default_asset_data_container, shadow_method).unwrap();
                    //x.blend_method().set(&mut default_asset_data_container, blend_method).unwrap();
                    x.alpha_threshold()
                        .set(
                            &mut default_asset_data_container,
                            material.alpha_cutoff().unwrap_or(0.5),
                        )
                        .unwrap();
                    x.backface_culling()
                        .set(&mut default_asset_data_container, false)
                        .unwrap();
                    //TODO: Does this incorrectly write older enum string names when code is older than schema file?
                    x.color_texture_has_alpha_channel()
                        .set(&mut default_asset_data_container, false)
                        .unwrap();
                    default_asset_object
                };

                //
                // Return the created assets
                //
                imported_assets.insert(
                    name,
                    ImportedImportable {
                        file_references: Default::default(),
                        import_data: None,
                        default_asset: Some(default_asset),
                    },
                );
            }
        }

        imported_assets
    }
}

pub struct GltfAssetPlugin;

impl AssetPlugin for GltfAssetPlugin {
    fn setup(
        _schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        _builder_registry: &mut BuilderRegistryBuilder,
        _job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<GltfImporter>();
    }
}

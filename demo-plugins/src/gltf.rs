pub use super::*;
use std::path::{Path, PathBuf};

use super::generated::{
    MeshAdvMaterialAssetRecord, MeshAdvMaterialImportedDataRecord, MeshAdvMeshAssetRecord,
    MeshAdvMeshImportedDataRecord,
};
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::pipeline::{
    AssetPlugin, Builder, BuilderRegistry, BuiltAsset, ImporterRegistry,
};
use hydrate_model::pipeline::{ImportedImportable, Importer, ScannedImportable};
use hydrate_model::{
    BuilderRegistryBuilder, DataContainerMut, DataSet, EditorModel, HashMap, ImportableObject,
    ImporterRegistryBuilder, JobProcessorRegistryBuilder, ObjectId, ObjectLocation, ObjectName,
    Record, SchemaLinker, SchemaSet, SingleObject, Value,
};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};

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
        path: &Path,
        schema_set: &SchemaSet,
        importer_registry: &ImporterRegistry,
    ) -> Vec<ScannedImportable> {
        let mesh_asset_type = schema_set
            .find_named_type(MeshAdvMeshAssetRecord::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let material_asset_type = schema_set
            .find_named_type(MeshAdvMaterialAssetRecord::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let (doc, buffers, images) = ::gltf::import(path).unwrap();

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
        path: &Path,
        importable_objects: &HashMap<Option<String>, ImportableObject>,
        schema_set: &SchemaSet,
        //import_info: &ImportInfo,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let (doc, buffers, images) = ::gltf::import(path).unwrap();

        let mut imported_objects = HashMap::default();

        for (i, mesh) in doc.meshes().enumerate() {
            let name = Some(name_or_index("mesh", mesh.name(), i));
            if importable_objects.contains_key(&name) {
                //
                // Create import data
                //
                let import_data = {
                    let mut import_object =
                        MeshAdvMeshImportedDataRecord::new_single_object(schema_set).unwrap();
                    let mut import_data_container =
                        DataContainerMut::new_single_object(&mut import_object, schema_set);
                    let x = MeshAdvMeshImportedDataRecord::default();
                    import_object
                };

                //
                // Create the default asset
                //

                let default_asset = {
                    let mut default_asset_object =
                        MeshAdvMeshAssetRecord::new_single_object(schema_set).unwrap();
                    let mut default_asset_data_container =
                        DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
                    let x = MeshAdvMeshAssetRecord::default();
                    default_asset_object
                };

                //
                // Return the created objects
                //
                imported_objects.insert(
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
            if importable_objects.contains_key(&name) {
                //
                // Create the default asset
                //

                let default_asset = {
                    let mut default_asset_object =
                        MeshAdvMaterialAssetRecord::new_single_object(schema_set).unwrap();
                    let mut default_asset_data_container =
                        DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
                    let x = MeshAdvMaterialAssetRecord::default();
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
                    //     if let Some(referenced_object_id) = importable_objects.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&color_texture.).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_object_id).unwrap();
                    //     }
                    // }
                    //
                    // if let Some(metallic_roughness_texture) = json_data.metallic_roughness_texture {
                    //     if let Some(referenced_object_id) = importable_objects.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&metallic_roughness_texture).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_object_id).unwrap();
                    //     }
                    // }
                    //
                    // if let Some(normal_texture) = json_data.normal_texture {
                    //     if let Some(referenced_object_id) = importable_objects.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&normal_texture).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_object_id).unwrap();
                    //     }
                    // }
                    //
                    // if let Some(emissive_texture) = json_data.emissive_texture {
                    //     if let Some(referenced_object_id) = importable_objects.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&emissive_texture).unwrap()) {
                    //         x.color_texture().set(&mut default_asset_data_container, *referenced_object_id).unwrap();
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
                // Return the created objects
                //
                imported_objects.insert(
                    name,
                    ImportedImportable {
                        file_references: Default::default(),
                        import_data: None,
                        default_asset: Some(default_asset),
                    },
                );
            }
        }

        imported_objects
    }
}

pub struct GltfAssetPlugin;

impl AssetPlugin for GltfAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<GltfImporter>(schema_linker);
    }
}

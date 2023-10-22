pub use super::*;
use std::path::{Path, PathBuf};

use demo_types::gltf::*;
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::{BuilderRegistryBuilder, DataContainerMut, DataSet, EditorModel, HashMap, ImporterRegistryBuilder, ObjectId, ObjectLocation, ObjectName, Record, SchemaLinker, SchemaSet, SingleObject, Value};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuilderRegistry, BuiltAsset, ImporterRegistry};
use hydrate_model::pipeline::{ImportedImportable, ScannedImportable, Importer};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use super::generated::{MeshAdvMaterialAssetRecord, MeshAdvMaterialImportedDataRecord, MeshAdvMeshAssetRecord, MeshAdvMeshImportedDataRecord};

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
        object_ids: &HashMap<Option<String>, ObjectId>,
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
            if object_ids.contains_key(&name) {
                //
                // Create import data
                //
                let import_data = {
                    let mut import_object = MeshAdvMeshImportedDataRecord::new_single_object(schema_set).unwrap();
                    let mut import_data_container = DataContainerMut::new_single_object(&mut import_object, schema_set);
                    let x = MeshAdvMeshImportedDataRecord::default();
                    import_object
                };

                //
                // Create the default asset
                //

                let default_asset = {
                    let mut default_asset_object = MeshAdvMeshAssetRecord::new_single_object(schema_set).unwrap();
                    let mut default_asset_data_container = DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
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
                        import_data,
                        default_asset
                    },
                );
            }
        }

        for (i, material) in doc.materials().enumerate() {
            let name = Some(name_or_index("material", material.name(), i));
            if object_ids.contains_key(&name) {
                //
                // Create import data
                //
                let import_data = {
                    let mut import_object = MeshAdvMaterialImportedDataRecord::new_single_object(schema_set).unwrap();
                    let mut import_data_container = DataContainerMut::new_single_object(&mut import_object, schema_set);
                    let x = MeshAdvMaterialImportedDataRecord::default();
                    import_object
                };

                //
                // Create the default asset
                //

                let default_asset = {
                    let mut default_asset_object = MeshAdvMaterialAssetRecord::new_single_object(schema_set).unwrap();
                    let mut default_asset_data_container = DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
                    let x = MeshAdvMaterialAssetRecord::default();
                    default_asset_object
                };

                //
                // Return the created objects
                //
                imported_objects.insert(
                    name,
                    ImportedImportable {
                        file_references: Default::default(),
                        import_data,
                        default_asset
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
    ) {
        importer_registry.register_handler::<GltfImporter>(schema_linker);
    }
}

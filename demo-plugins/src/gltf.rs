pub use super::*;
use std::path::{Path, PathBuf};

use demo_types::gltf::*;
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::{BuilderRegistryBuilder, DataSet, EditorModel, HashMap, ImporterRegistryBuilder, ObjectId, ObjectLocation, ObjectName, SchemaLinker, SchemaSet, SingleObject, Value};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuilderRegistry, BuiltAsset, ImporterRegistry};
use hydrate_model::pipeline::{ImportedImportable, ScannedImportable, Importer};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};

pub struct GltfMeshAsset {}

impl GltfMeshAsset {
    pub fn schema_name() -> &'static str {
        "GltfMeshAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("imported_data", GltfImportedData::schema_name());
                //x.add_string("path");
                //x.add_boolean("compress");
            })
            .unwrap();
    }
}

pub struct GltfMeshImportedData {}

impl GltfMeshImportedData {
    pub fn schema_name() -> &'static str {
        "GltfMeshImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("asset", GltfImportedData::schema_name());
                //x.add_bytes("image_bytes"); // TODO: this would be a buffer
                //x.add_u32("width");
                //x.add_u32("height");
            })
            .unwrap();
    }
}

pub struct GltfMaterialAsset {}

impl GltfMaterialAsset {
    pub fn schema_name() -> &'static str {
        "GltfMaterialAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("imported_data", GltfImportedData::schema_name());
                //x.add_string("path");
                //x.add_boolean("compress");
            })
            .unwrap();
    }
}

pub struct GltfMaterialImportedData {}

impl GltfMaterialImportedData {
    pub fn schema_name() -> &'static str {
        "GltfMaterialImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("asset", GltfImportedData::schema_name());
                //x.add_bytes("image_bytes"); // TODO: this would be a buffer
                //x.add_u32("width");
                //x.add_u32("height");
            })
            .unwrap();
    }
}

pub struct GltfAssetPlugin;

impl AssetPlugin for GltfAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
    ) {
        GltfMeshAsset::register_schema(schema_linker);
        GltfMeshImportedData::register_schema(schema_linker);

        GltfMaterialAsset::register_schema(schema_linker);
        GltfMaterialImportedData::register_schema(schema_linker);

        importer_registry.register_handler::<GltfImporter>(schema_linker);
        builder_registry.register_handler::<GltfMeshBuilder>(schema_linker);
        builder_registry.register_handler::<GltfMaterialBuilder>(schema_linker);
    }
}

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
            .find_named_type(GltfMeshAsset::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let material_asset_type = schema_set
            .find_named_type(GltfMaterialAsset::schema_name())
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

    //fn create_default_asset(&self, editor_model: &mut EditorModel, object_name: ObjectName, object_location: ObjectLocation) -> ObjectId {
    //    let schema_record = editor_model.root_edit_context_mut().schema_set().find_named_type(GltfAsset::schema_name()).unwrap().as_record().unwrap().clone();
    //    editor_model.root_edit_context_mut().new_object(&object_name, &object_location, &schema_record)
    //}

    fn import_file(
        &self,
        path: &Path,
        object_ids: &HashMap<Option<String>, ObjectId>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
    ) -> HashMap<Option<String>, ImportedImportable> {
        // TODO: Replace with a shim so we can track what files are being read
        // - We trigger the importer for them by specifying the file path and kind of file (i.e. an image, specific type of JSON file, etc.)
        // - We may need to let the "import" dialog try to perform the import to get error messages and discover what will end up being imported
        //let bytes = std::fs::read(path).unwrap();

        // let decoded_image =
        //     ::image::load_from_memory_with_format(&bytes, ::image::GltfFormat::Png).unwrap();

        let gltf_mesh_imported_data_schema = schema
            .find_named_type(GltfMeshImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let gltf_material_imported_data_schema = schema
            .find_named_type(GltfMaterialImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let (doc, buffers, images) = ::gltf::import(path).unwrap();

        //let mut import_object = SingleObject::new(gltf_imported_data_schema);
        // import_object.set_property_override(schema, "image_bytes", Value::Bytes(image_bytes));
        // import_object.set_property_override(schema, "width", Value::U32(width));
        // import_object.set_property_override(schema, "height", Value::U32(height));

        let mut imported_objects = HashMap::default();

        for (i, mesh) in doc.meshes().enumerate() {
            let name = Some(name_or_index("mesh", mesh.name(), i));
            if object_ids.contains_key(&name) {
                let mut import_object = SingleObject::new(gltf_mesh_imported_data_schema);

                imported_objects.insert(
                    name,
                    ImportedImportable {
                        file_references: Default::default(),
                        data: import_object,
                    },
                );
            }
        }

        for (i, material) in doc.materials().enumerate() {
            let name = Some(name_or_index("material", material.name(), i));
            if object_ids.contains_key(&name) {
                let mut import_object = SingleObject::new(gltf_material_imported_data_schema);

                imported_objects.insert(
                    name,
                    ImportedImportable {
                        file_references: Default::default(),
                        data: import_object,
                    },
                );
            }
        }

        imported_objects
    }
}

#[derive(Default)]
pub struct GltfMeshBuilder {}

impl Builder for GltfMeshBuilder {
    fn asset_type(&self) -> &'static str {
        GltfMeshAsset::schema_name()
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

        //
        // Compress the image, or just return the raw image bytes
        //

        let processed_data = GltfBuiltMeshData {};

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

#[derive(Default)]
pub struct GltfMaterialBuilder {}

impl Builder for GltfMaterialBuilder {
    fn asset_type(&self) -> &'static str {
        GltfMaterialAsset::schema_name()
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

        //
        // Compress the image, or just return the raw image bytes
        //

        let processed_data = GltfBuiltMaterialData {};

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

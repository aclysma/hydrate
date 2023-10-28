pub use super::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use glam::Vec3;

use demo_types::mesh_adv::*;
use hydrate_base::BuiltObjectMetadata;
use hydrate_model::{BuilderRegistryBuilder, DataContainer, DataContainerMut, DataSet, Enum, HashMap, ImportableObject, ImporterId, ImporterRegistryBuilder, JobProcessorRegistryBuilder, ObjectId, ObjectRefField, Record, ReferencedSourceFile, SchemaLinker, SchemaSet, SingleObject};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuiltAsset};
use hydrate_model::pipeline::{ImportedImportable, ScannedImportable, Importer};
use serde::{Deserialize, Serialize};
use type_uuid::{TypeUuid, TypeUuidDynamic};
use uuid::Uuid;
use crate::b3f::B3FReader;
use crate::generated::{GpuBufferAssetRecord, MeshAdvIndexTypeEnum, MeshAdvMeshAssetRecord, MeshAdvMeshImportedDataRecord};
use crate::push_buffer::PushBuffer;

use super::generated::{MeshAdvMaterialImportedDataRecord, MeshAdvMaterialAssetRecord, MeshAdvBlendMethodEnum, MeshAdvShadowMethodEnum};

#[derive(Serialize, Deserialize, Debug)]
enum MeshPartJsonIndexType {
    U16,
    U32,
}

#[derive(Serialize, Deserialize, Debug)]
struct MeshPartJson {
    #[serde(default)]
    pub position: Option<u32>,
    #[serde(default)]
    pub normal: Option<u32>,
    #[serde(default)]
    pub tangent: Option<u32>,
    #[serde(default)]
    pub uv: Vec<u32>,
    pub indices: u32,
    pub index_type: MeshPartJsonIndexType,
    // path to .blender_material
    pub material: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MeshJson {
    pub mesh_parts: Vec<MeshPartJson>,
}

fn try_cast_u8_slice<T: Copy + 'static>(data: &[u8]) -> Option<&[T]> {
    if data.len() % std::mem::size_of::<T>() != 0 {
        return None;
    }

    let ptr = data.as_ptr() as *const T;
    if ptr as usize % std::mem::align_of::<T>() != 0 {
        return None;
    }

    let casted: &[T] =
        unsafe { std::slice::from_raw_parts(ptr, data.len() / std::mem::size_of::<T>()) };

    Some(casted)
}

#[derive(TypeUuid, Default)]
#[uuid = "5f2be1a1-b025-4d72-960b-24cb03ff19de"]
pub struct BlenderMeshImporter;

impl Importer for BlenderMeshImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["blender_mesh"]
    }

    fn scan_file(
        &self,
        path: &Path,
        schema_set: &SchemaSet,
    ) -> Vec<ScannedImportable> {
        let mesh_adv_asset_type = schema_set
            .find_named_type(MeshAdvMeshAssetRecord::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let gpu_buffer_asset_type = schema_set
            .find_named_type(GpuBufferAssetRecord::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let bytes = std::fs::read(path).unwrap();

        let b3f_reader = B3FReader::new(&bytes)
            .ok_or("Blender Mesh Import error, mesh file format not recognized").unwrap();
        let mesh_as_json: MeshJson =
            serde_json::from_slice(b3f_reader.get_block(0)).map_err(|e| e.to_string()).unwrap();

        fn try_add_file_reference<T: TypeUuid>(file_references: &mut Vec<ReferencedSourceFile>, path_as_string: &String) {
            let importer_image_id = ImporterId(Uuid::from_bytes(T::UUID));
            file_references.push(ReferencedSourceFile {
                importer_id: importer_image_id,
                path: PathBuf::from_str(path_as_string).unwrap(),
            })
        }

        let mut mesh_file_references = Vec::default();
        for mesh_part in &mesh_as_json.mesh_parts {
            try_add_file_reference::<BlenderMaterialImporter>(&mut mesh_file_references, &mesh_part.material);
        }

        let mut scanned_importables = Vec::default();
        scanned_importables.push(ScannedImportable {
            name: None,
            asset_type: mesh_adv_asset_type,
            file_references: mesh_file_references,
        });

        // scanned_importables.push(ScannedImportable {
        //     name: Some("vertex_full_buffer".to_string()),
        //     asset_type: gpu_buffer_asset_type.clone(),
        //     file_references: vec![],
        // });
        //
        // scanned_importables.push(ScannedImportable {
        //     name: Some("vertex_position_buffer_id".to_string()),
        //     asset_type: gpu_buffer_asset_type.clone(),
        //     file_references: vec![],
        // });
        //
        // scanned_importables.push(ScannedImportable {
        //     name: Some("index_buffer_id".to_string()),
        //     asset_type: gpu_buffer_asset_type,
        //     file_references: vec![],
        // });

        scanned_importables
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
        let bytes = std::fs::read(path).unwrap();

        let b3f_reader = B3FReader::new(&bytes)
            .ok_or("Blender Mesh Import error, mesh file format not recognized").unwrap();
        let mesh_as_json: MeshJson =
            serde_json::from_slice(b3f_reader.get_block(0)).map_err(|e| e.to_string()).unwrap();


        let mut all_positions = Vec::<glam::Vec3>::with_capacity(1024);
        let mut all_position_indices = Vec::<u32>::with_capacity(8192);

        let mut all_vertices_full = PushBuffer::new(16384);
        let mut all_vertices_position = PushBuffer::new(16384);
        let mut all_indices = PushBuffer::new(16384);

        let mut mesh_parts: Vec<MeshAdvPartAssetData> =
            Vec::with_capacity(mesh_as_json.mesh_parts.len());

        for mesh_part in &mesh_as_json.mesh_parts {
            //
            // Get byte slices of all input data for this mesh part
            //
            let positions_bytes =
                b3f_reader.get_block(mesh_part.position.ok_or("No position data").unwrap() as usize);
            let normals_bytes =
                b3f_reader.get_block(mesh_part.normal.ok_or("No normal data").unwrap() as usize);
            let tex_coords_bytes = b3f_reader
                .get_block(*mesh_part.uv.get(0).ok_or("No texture coordinate data").unwrap() as usize);
            let part_indices_bytes = b3f_reader.get_block(mesh_part.indices as usize);

            //
            // Get strongly typed slices of all input data for this mesh part
            //
            let positions = try_cast_u8_slice::<[f32; 3]>(positions_bytes)
                .ok_or("Could not cast due to alignment").unwrap();
            let normals = try_cast_u8_slice::<[f32; 3]>(normals_bytes)
                .ok_or("Could not cast due to alignment").unwrap();
            let tex_coords = try_cast_u8_slice::<[f32; 2]>(tex_coords_bytes)
                .ok_or("Could not cast due to alignment").unwrap();

            // Indices may be encoded as u16 or u32, either way copy them out to a Vec<u32>
            let mut part_indices = Vec::<u32>::default();
            match mesh_part.index_type {
                MeshPartJsonIndexType::U16 => {
                    let part_indices_u16 = try_cast_u8_slice::<u16>(part_indices_bytes)
                        .ok_or("Could not cast due to alignment").unwrap();
                    part_indices.reserve(part_indices_u16.len());
                    for &part_index in part_indices_u16 {
                        part_indices.push(part_index as u32);
                    }
                }
                MeshPartJsonIndexType::U32 => {
                    let part_indices_u32 = try_cast_u8_slice::<u32>(part_indices_bytes)
                        .ok_or("Could not cast due to alignment").unwrap();
                    part_indices.reserve(part_indices_u32.len());
                    for &part_index in part_indices_u32 {
                        part_indices.push(part_index);
                    }
                }
            };

            let part_data = super::mesh_util::process_mesh_part(
                &part_indices,
                &positions,
                &normals,
                &tex_coords,
                &mut all_vertices_full,
                &mut all_vertices_position,
                &mut all_indices,
            );

            //
            // Positions and indices for the visibility system
            //
            for index in part_indices {
                all_position_indices.push(index as u32);
            }

            for i in 0..positions.len() {
                all_positions.push(Vec3::new(positions[i][0], positions[i][1], positions[i][2]));
            }

            // if let Some(referenced_object_id) = importable_objects.get(&None).unwrap().referenced_paths.get(&PathBuf::from_str(&mesh_part.material).unwrap()) {
            //     ref_field.set(data_container, *referenced_object_id).unwrap();
            // }

            mesh_parts.push(MeshAdvPartAssetData {
                //mesh_material,
                vertex_full_buffer_offset_in_bytes: part_data.vertex_full_buffer_offset_in_bytes,
                vertex_full_buffer_size_in_bytes: part_data.vertex_full_buffer_size_in_bytes,
                vertex_position_buffer_offset_in_bytes: part_data
                    .vertex_position_buffer_offset_in_bytes,
                vertex_position_buffer_size_in_bytes: part_data
                    .vertex_position_buffer_size_in_bytes,
                index_buffer_offset_in_bytes: part_data.index_buffer_offset_in_bytes,
                index_buffer_size_in_bytes: part_data.index_buffer_size_in_bytes,
                index_type: part_data.index_type,
            })
        }

        //TODO: Build the mesh
        // Figure out how to move this work to the build step
        // We want to produce the buffers as separate built assets without processing the file more than once

        //
        // Read imported data
        //
        let import_data = {
            let mut import_data_object = MeshAdvMeshImportedDataRecord::new_single_object(schema_set).unwrap();
            let mut import_data_container = DataContainerMut::new_single_object(&mut import_data_object, schema_set);
            let x = MeshAdvMeshImportedDataRecord::default();
            let entry_uuid = x.mesh_parts().add_entry(&mut import_data_container);
            let entry = x.mesh_parts().entry(entry_uuid);
            entry.vertex_full_buffer_offset_in_bytes().set(&mut import_data_container, 1).unwrap();
            entry.vertex_full_buffer_size_in_bytes().set(&mut import_data_container, 1).unwrap();
            entry.vertex_position_buffer_offset_in_bytes().set(&mut import_data_container, 1).unwrap();
            entry.vertex_position_buffer_size_in_bytes().set(&mut import_data_container, 1).unwrap();
            entry.index_buffer_offset_in_bytes().set(&mut import_data_container, 1).unwrap();
            entry.index_buffer_size_in_bytes().set(&mut import_data_container, 1).unwrap();
            //entry.mesh_material().set(&mut import_data_container, )
            entry.index_type().set(&mut import_data_container, MeshAdvIndexTypeEnum::Uint16).unwrap();

            import_data_object
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
        let mut imported_objects = HashMap::default();
        imported_objects.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                import_data: Some(import_data),
                default_asset: Some(default_asset)
            },
        );
        imported_objects
    }
}

pub struct BlenderMeshAssetPlugin;

impl AssetPlugin for BlenderMeshAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<BlenderMeshImporter>(schema_linker);
    }
}



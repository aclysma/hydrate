pub use super::*;
use std::path::{Path, PathBuf};

use crate::generated::{
    MeshAdvMeshAssetAccessor, MeshAdvMeshAssetOwned, MeshAdvMeshImportedDataAccessor,
    MeshAdvMeshImportedDataOwned,
};
use crate::push_buffer::PushBuffer;
use hydrate_base::b3f::B3FReader;
use hydrate_data::{RecordBuilder, RecordOwned};
use hydrate_model::pipeline::{AssetPlugin, ImportContext, ScanContext};
use hydrate_model::pipeline::{ImportedImportable, Importer, ScannedImportable};
use hydrate_pipeline::{
    BuilderRegistryBuilder, DataContainerRefMut, HashMap, ImportableAsset, ImporterId,
    ImporterRegistry, ImporterRegistryBuilder, JobProcessorRegistryBuilder, RecordAccessor,
    ReferencedSourceFile, SchemaLinker, SchemaSet,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use uuid::Uuid;

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
    pub material: PathBuf,
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
        context: ScanContext,
    ) -> Vec<ScannedImportable> {
        let mesh_adv_asset_type = context
            .schema_set
            .find_named_type(MeshAdvMeshAssetAccessor::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let bytes = std::fs::read(context.path).unwrap();

        let b3f_reader = B3FReader::new(&bytes)
            .ok_or("Blender Mesh Import error, mesh file format not recognized")
            .unwrap();
        let mesh_as_json: MeshJson = {
            profiling::scope!("serde_json::from_slice");
            serde_json::from_slice(b3f_reader.get_block(0))
                .map_err(|e| e.to_string())
                .unwrap()
        };

        fn try_add_file_reference<T: TypeUuid>(
            file_references: &mut Vec<ReferencedSourceFile>,
            path: PathBuf,
        ) {
            let importer_image_id = ImporterId(Uuid::from_bytes(T::UUID));
            file_references.push(ReferencedSourceFile {
                importer_id: importer_image_id,
                path,
            })
        }

        let mut mesh_file_references = Vec::default();
        for mesh_part in &mesh_as_json.mesh_parts {
            try_add_file_reference::<BlenderMaterialImporter>(
                &mut mesh_file_references,
                mesh_part.material.clone(),
            );
        }

        let mut scanned_importables = Vec::default();
        scanned_importables.push(ScannedImportable {
            name: None,
            asset_type: mesh_adv_asset_type,
            file_references: mesh_file_references,
        });

        scanned_importables
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let bytes = std::fs::read(context.path).unwrap();

        let b3f_reader = B3FReader::new(&bytes)
            .ok_or("Blender Mesh Import error, mesh file format not recognized")
            .unwrap();
        let mesh_as_json: MeshJson = {
            profiling::scope!("serde_json::from_slice");
            serde_json::from_slice(b3f_reader.get_block(0))
                .map_err(|e| e.to_string())
                .unwrap()
        };

        let import_data = MeshAdvMeshImportedDataOwned::new_builder(context.schema_set);

        //
        // Find the materials and assign them unique slot indexes
        //
        let mut material_slots = Vec::default();
        let mut material_slots_lookup = HashMap::default();
        for mesh_part in &mesh_as_json.mesh_parts {
            if !material_slots_lookup.contains_key(&mesh_part.material) {
                let slot_index = material_slots.len() as u32;
                material_slots.push(mesh_part.material.clone());
                material_slots_lookup.insert(mesh_part.material.clone(), slot_index);
            }
        }

        for mesh_part in &mesh_as_json.mesh_parts {
            //
            // Get byte slices of all input data for this mesh part
            //
            let positions_bytes = b3f_reader
                .get_block(mesh_part.position.ok_or("No position data").unwrap() as usize);
            let normals_bytes =
                b3f_reader.get_block(mesh_part.normal.ok_or("No normal data").unwrap() as usize);
            let tex_coords_bytes = b3f_reader.get_block(
                *mesh_part
                    .uv
                    .get(0)
                    .ok_or("No texture coordinate data")
                    .unwrap() as usize,
            );
            let part_indices_bytes = b3f_reader.get_block(mesh_part.indices as usize);

            //
            // Get strongly typed slices of all input data for this mesh part
            //

            // Indices may be encoded as u16 or u32, either way copy them out to a Vec<u32>
            let mut part_indices_u32 = Vec::<u32>::default();
            match mesh_part.index_type {
                MeshPartJsonIndexType::U16 => {
                    let part_indices_u16_ref = try_cast_u8_slice::<u16>(part_indices_bytes)
                        .ok_or("Could not cast due to alignment")
                        .unwrap();
                    part_indices_u32.reserve(part_indices_u16_ref.len());
                    for &part_index in part_indices_u16_ref {
                        part_indices_u32.push(part_index as u32);
                    }
                }
                MeshPartJsonIndexType::U32 => {
                    let part_indices_u32_ref = try_cast_u8_slice::<u32>(part_indices_bytes)
                        .ok_or("Could not cast due to alignment")
                        .unwrap();
                    part_indices_u32.reserve(part_indices_u32_ref.len());
                    for &part_index in part_indices_u32_ref {
                        part_indices_u32.push(part_index);
                    }
                }
            };

            let part_indices = PushBuffer::from_vec(&part_indices_u32).into_data();

            let material_index = *material_slots_lookup.get(&mesh_part.material).unwrap();

            let entry_uuid = import_data.mesh_parts().add_entry().unwrap();
            let entry = import_data.mesh_parts().entry(entry_uuid);
            entry.positions().set(positions_bytes.to_vec()).unwrap();
            entry.normals().set(normals_bytes.to_vec()).unwrap();
            entry
                .texture_coordinates()
                .set(tex_coords_bytes.to_vec())
                .unwrap();
            entry.indices().set(part_indices).unwrap();
            entry.material_index().set(material_index).unwrap();
        }

        //
        // Create the default asset
        //
        let default_asset = MeshAdvMeshAssetOwned::new_builder(context.schema_set);

        //
        // Set up the material slots
        //
        for material_slot in material_slots {
            let asset_id = context
                .importable_assets
                .get(&None)
                .unwrap()
                .referenced_paths
                .get(&material_slot)
                .unwrap();
            let entry = default_asset.material_slots().add_entry().unwrap();
            default_asset
                .material_slots()
                .entry(entry)
                .set(*asset_id)
                .unwrap();
        }

        //
        // Return the created assets
        //
        let mut imported_assets = HashMap::default();
        imported_assets.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                import_data: Some(import_data.into_inner().unwrap()),
                default_asset: Some(default_asset.into_inner().unwrap()),
            },
        );
        imported_assets
    }
}

pub struct BlenderMeshAssetPlugin;

impl AssetPlugin for BlenderMeshAssetPlugin {
    fn setup(
        _schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        _builder_registry: &mut BuilderRegistryBuilder,
        _job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<BlenderMeshImporter>();
    }
}

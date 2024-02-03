pub use super::*;
use std::io::BufReader;
use std::path::PathBuf;

use crate::generated::{MeshAdvMeshAssetRecord, MeshAdvMeshImportedDataRecord};
use crate::push_buffer::PushBuffer;
use hydrate_base::b3f::B3FReader;
use hydrate_data::{ImportableName, Record};
use hydrate_model::pipeline::Importer;
use hydrate_model::pipeline::{AssetPlugin, ImportContext, ScanContext};
use hydrate_pipeline::{
    AssetPluginSetupContext, BuilderRegistryBuilder, HashMap, ImporterRegistryBuilder,
    JobProcessorRegistryBuilder, PipelineResult,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

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
    ) -> PipelineResult<()> {
        let file = std::fs::File::open(context.path)?;
        let mut buf_reader = BufReader::new(file);
        let b3f_reader = B3FReader::new(&mut buf_reader)?
            .ok_or("Blender Mesh Import error, mesh file format not recognized")?;
        let json_block = b3f_reader.read_block(&mut buf_reader, 0)?;
        let mesh_as_json: MeshJson = {
            profiling::scope!("serde_json::from_slice");
            serde_json::from_slice(&json_block).map_err(|e| e.to_string())?
        };

        context.add_default_importable::<MeshAdvMeshAssetRecord>()?;

        for mesh_part in &mesh_as_json.mesh_parts {
            context.add_path_reference_with_importer::<BlenderMaterialImporter, _>(
                ImportableName::default(),
                &mesh_part.material,
            )?;
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
        let file = std::fs::File::open(context.path)?;
        let mut buf_reader = BufReader::new(file);
        let b3f_reader = B3FReader::new(&mut buf_reader)?
            .ok_or("Blender Mesh Import error, mesh file format not recognized")?;
        let json_block = b3f_reader.read_block(&mut buf_reader, 0)?;
        let mesh_as_json: MeshJson = {
            profiling::scope!("serde_json::from_slice");
            serde_json::from_slice(&json_block).map_err(|e| e.to_string())?
        };

        let import_data = MeshAdvMeshImportedDataRecord::new_builder(context.schema_set);

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
            let positions_bytes = b3f_reader.read_block(
                &mut buf_reader,
                mesh_part.position.ok_or("No position data")? as usize,
            )?;
            let normals_bytes = b3f_reader.read_block(
                &mut buf_reader,
                mesh_part.normal.ok_or("No normal data")? as usize,
            )?;
            let tex_coords_bytes = b3f_reader.read_block(
                &mut buf_reader,
                *mesh_part.uv.get(0).ok_or("No texture coordinate data")? as usize,
            )?;
            let part_indices_bytes =
                b3f_reader.read_block(&mut buf_reader, mesh_part.indices as usize)?;

            //
            // Get strongly typed slices of all input data for this mesh part
            //

            // Indices may be encoded as u16 or u32, either way copy them out to a Vec<u32>
            let mut part_indices_u32 = Vec::<u32>::default();
            match mesh_part.index_type {
                MeshPartJsonIndexType::U16 => {
                    let part_indices_u16_ref = try_cast_u8_slice::<u16>(&part_indices_bytes)
                        .ok_or("Could not cast due to alignment")?;
                    part_indices_u32.reserve(part_indices_u16_ref.len());
                    for &part_index in part_indices_u16_ref {
                        part_indices_u32.push(part_index as u32);
                    }
                }
                MeshPartJsonIndexType::U32 => {
                    let part_indices_u32_ref = try_cast_u8_slice::<u32>(&part_indices_bytes)
                        .ok_or("Could not cast due to alignment")?;
                    part_indices_u32.reserve(part_indices_u32_ref.len());
                    for &part_index in part_indices_u32_ref {
                        part_indices_u32.push(part_index);
                    }
                }
            };

            let part_indices = PushBuffer::from_vec(&part_indices_u32).into_data();

            let material_index = *material_slots_lookup
                .get(&mesh_part.material)
                .ok_or("Could not find material reference by path")?;

            let entry_uuid = import_data.mesh_parts().add_entry()?;
            let entry = import_data.mesh_parts().entry(entry_uuid);
            entry.positions().set(positions_bytes.to_vec())?;
            entry.normals().set(normals_bytes.to_vec())?;
            entry.texture_coordinates().set(tex_coords_bytes.to_vec())?;
            entry.indices().set(part_indices)?;
            entry.material_index().set(material_index)?;
        }

        //
        // Create the default asset
        //
        let default_asset = MeshAdvMeshAssetRecord::new_builder(context.schema_set);

        //
        // Set up the material slots
        //
        for material_slot in material_slots {
            let asset_id = context.asset_id_for_referenced_file_path(
                ImportableName::default(),
                &material_slot.into(),
            )?;
            let entry = default_asset.material_slots().add_entry()?;
            default_asset.material_slots().entry(entry).set(asset_id)?;
        }

        //
        // Return the created assets
        //
        context
            .add_default_importable(default_asset.into_inner()?, Some(import_data.into_inner()?));
        Ok(())
    }
}

pub struct BlenderMeshAssetPlugin;

impl AssetPlugin for BlenderMeshAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        context
            .importer_registry
            .register_handler::<BlenderMeshImporter>();
    }
}

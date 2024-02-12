pub use super::*;
use glam::Vec3;
use rafx_api::RafxResourceType;

use crate::generated::{
    MeshAdvMaterialAssetRecord, MeshAdvMeshAssetRecord, MeshAdvMeshImportedDataRecord,
};
use crate::push_buffer::PushBuffer;
use demo_types::mesh_adv::*;
use hydrate_data::Record;
use hydrate_model::pipeline::{AssetPlugin, Builder};
use hydrate_pipeline::{
    AssetId, AssetPluginSetupContext, BuilderContext,
    JobInput, JobOutput, JobProcessor,
    PipelineResult, RunContext,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Hash, Serialize, Deserialize)]
pub struct MeshAdvMaterialJobInput {
    pub asset_id: AssetId,
}
impl JobInput for MeshAdvMaterialJobInput {}

#[derive(Serialize, Deserialize)]
pub struct MeshAdvMaterialJobOutput {}
impl JobOutput for MeshAdvMaterialJobOutput {}

#[derive(Default, TypeUuid)]
#[uuid = "d28004fa-6eb7-4110-8a17-10d42d92a956"]
pub struct MeshAdvMaterialJobProcessor;

impl JobProcessor for MeshAdvMaterialJobProcessor {
    type InputT = MeshAdvMaterialJobInput;
    type OutputT = MeshAdvMaterialJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn run<'a>(
        &self,
        context: &'a RunContext<'a, Self::InputT>,
    ) -> PipelineResult<MeshAdvMaterialJobOutput> {
        //
        // Read asset data
        //
        let asset_data = context.asset::<MeshAdvMaterialAssetRecord>(context.input.asset_id)?;

        let base_color_factor = asset_data.base_color_factor().get_vec4()?;
        let emissive_factor = asset_data.emissive_factor().get_vec3()?;

        let metallic_factor = asset_data.metallic_factor().get()?;
        let roughness_factor = asset_data.roughness_factor().get()?;
        let normal_texture_scale = asset_data.normal_texture_scale().get()?;

        let color_texture = asset_data.color_texture().get()?;
        let metallic_roughness_texture = asset_data.metallic_roughness_texture().get()?;
        let normal_texture = asset_data.normal_texture().get()?;
        let emissive_texture = asset_data.emissive_texture().get()?;
        let shadow_method = asset_data.shadow_method().get()?;
        let blend_method = asset_data.blend_method().get()?;

        let alpha_threshold = asset_data.alpha_threshold().get()?;
        let backface_culling = asset_data.backface_culling().get()?;
        let color_texture_has_alpha_channel =
            asset_data.color_texture_has_alpha_channel().get().unwrap();

        //
        // Create the processed data
        //
        let processed_data = MeshAdvMaterialData {
            base_color_factor,
            emissive_factor,
            metallic_factor,
            roughness_factor,
            normal_texture_scale,
            has_base_color_texture: !color_texture.is_null(),
            base_color_texture_has_alpha_channel: color_texture_has_alpha_channel,
            has_metallic_roughness_texture: !metallic_roughness_texture.is_null(),
            has_normal_texture: !normal_texture.is_null(),
            has_emissive_texture: !emissive_texture.is_null(),
            shadow_method: shadow_method.into(),
            blend_method: blend_method.into(),
            alpha_threshold,
            backface_culling,
        };

        //
        // Serialize and return
        //
        context.produce_default_artifact(context.input.asset_id, processed_data)?;

        Ok(MeshAdvMaterialJobOutput {})
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "02f17f4e-8df2-4b79-95cf-d2ee62e92a01"]
pub struct MeshAdvMaterialBuilder {}

impl Builder for MeshAdvMaterialBuilder {
    fn asset_type(&self) -> &'static str {
        MeshAdvMaterialAssetRecord::schema_name()
    }

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) -> PipelineResult<()> {
        //Future: Might produce jobs per-platform
        context.enqueue_job::<MeshAdvMaterialJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            MeshAdvMaterialJobInput {
                asset_id: context.asset_id,
            },
        )?;
        Ok(())
    }
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

#[derive(Hash, Serialize, Deserialize)]
pub struct MeshAdvMeshPreprocessJobInput {
    pub asset_id: AssetId,
}
impl JobInput for MeshAdvMeshPreprocessJobInput {}

#[derive(Serialize, Deserialize)]
pub struct MeshAdvMeshPreprocessJobOutput {}
impl JobOutput for MeshAdvMeshPreprocessJobOutput {}

#[derive(Default, TypeUuid)]
#[uuid = "d1a87176-09b5-4722-802e-60012653966d"]
pub struct MeshAdvMeshPreprocessJobProcessor;

impl JobProcessor for MeshAdvMeshPreprocessJobProcessor {
    type InputT = MeshAdvMeshPreprocessJobInput;
    type OutputT = MeshAdvMeshPreprocessJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn run<'a>(
        &self,
        context: &'a RunContext<'a, Self::InputT>,
    ) -> PipelineResult<MeshAdvMeshPreprocessJobOutput> {
        //
        // Read asset data
        //
        let asset_data = context.asset::<MeshAdvMeshAssetRecord>(context.input.asset_id)?;

        let mut materials = Vec::default();
        for entry in asset_data
            .material_slots()
            .resolve_entries()
            .unwrap()
            .into_iter()
        {
            let entry = asset_data.material_slots().entry(*entry).get().unwrap();
            materials.push(entry);
        }

        //
        // Read import data
        //
        let imported_data = context
            .imported_data::<MeshAdvMeshImportedDataRecord>(context.input.asset_id)
            .unwrap();

        let mut all_positions = Vec::<glam::Vec3>::with_capacity(1024);
        let mut all_position_indices = Vec::<u32>::with_capacity(8192);

        let mut all_vertices_full = PushBuffer::new(16384);
        let mut all_vertices_position = PushBuffer::new(16384);
        let mut all_indices = PushBuffer::new(16384);

        let mut mesh_part_data = Vec::default();
        for entry in imported_data
            .mesh_parts()
            .resolve_entries()
            .unwrap()
            .into_iter()
        {
            let entry = imported_data.mesh_parts().entry(*entry);

            //
            // Get strongly typed slices of all input data for this mesh part
            //
            let positions_field_reader = entry.positions();
            let positions_bytes = positions_field_reader.get()?;
            let positions = try_cast_u8_slice::<[f32; 3]>(positions_bytes)
                .ok_or("Could not cast due to alignment")?;

            let normal_field_reader = entry.normals();
            let normals_bytes = normal_field_reader.get()?;
            let normals = try_cast_u8_slice::<[f32; 3]>(normals_bytes)
                .ok_or("Could not cast due to alignment")?;

            let tex_coords_field_reader = entry.texture_coordinates();
            let tex_coords_bytes = tex_coords_field_reader.get()?;
            let tex_coords = try_cast_u8_slice::<[f32; 2]>(tex_coords_bytes)
                .ok_or("Could not cast due to alignment")?;

            let indices_field_reader = entry.indices();
            let indices_bytes = indices_field_reader.get()?;
            let part_indices =
                try_cast_u8_slice::<u32>(indices_bytes).ok_or("Could not cast due to alignment")?;

            //
            // Part data which mostly contains offsets in the buffers for this part
            //
            let part_data = mesh_util::process_mesh_part(
                part_indices,
                positions,
                normals,
                tex_coords,
                &mut all_vertices_full,
                &mut all_vertices_position,
                &mut all_indices,
            );

            mesh_part_data.push(part_data);

            //
            // Positions and indices for the visibility system
            //
            for index in part_indices {
                all_position_indices.push(*index as u32);
            }

            for i in 0..positions.len() {
                all_positions.push(Vec3::new(positions[i][0], positions[i][1], positions[i][2]));
            }
        }

        //
        // Vertex Full Buffer
        //
        let vertex_buffer_full_artifact_id = if !all_vertices_full.is_empty() {
            Some(context.produce_artifact(
                context.input.asset_id,
                Some("full"),
                MeshAdvBufferAssetData {
                    resource_type: RafxResourceType::VERTEX_BUFFER,
                    alignment: std::mem::size_of::<MeshVertexFull>() as u32,
                    data: all_vertices_full.into_data(),
                },
            ))
        } else {
            None
        };

        //
        // Vertex Position Buffer
        //
        let vertex_buffer_position_artifact_id = if !all_vertices_position.is_empty() {
            Some(context.produce_artifact(
                context.input.asset_id,
                Some("position"),
                MeshAdvBufferAssetData {
                    resource_type: RafxResourceType::VERTEX_BUFFER,
                    alignment: std::mem::size_of::<MeshVertexPosition>() as u32,
                    data: all_vertices_position.into_data(),
                },
            ))
        } else {
            None
        };

        //
        // Mesh asset
        //
        context.produce_default_artifact_with_handles(
            context.input.asset_id,
            |handle_factory| {
                let mut mesh_parts = Vec::default();
                for (entry, part_data) in imported_data
                    .mesh_parts()
                    .resolve_entries()
                    .unwrap()
                    .into_iter()
                    .zip(mesh_part_data)
                {
                    let entry = imported_data.mesh_parts().entry(*entry);

                    let material_slot_index = entry.material_index().get().unwrap();
                    let material_asset_id = materials[material_slot_index as usize];
                    let material_handle =
                        handle_factory.make_handle_to_default_artifact(material_asset_id);

                    mesh_parts.push(MeshAdvPartAssetData {
                        vertex_full_buffer_offset_in_bytes: part_data
                            .vertex_full_buffer_offset_in_bytes,
                        vertex_full_buffer_size_in_bytes: part_data
                            .vertex_full_buffer_size_in_bytes,
                        vertex_position_buffer_offset_in_bytes: part_data
                            .vertex_position_buffer_offset_in_bytes,
                        vertex_position_buffer_size_in_bytes: part_data
                            .vertex_position_buffer_size_in_bytes,
                        index_buffer_offset_in_bytes: part_data.index_buffer_offset_in_bytes,
                        index_buffer_size_in_bytes: part_data.index_buffer_size_in_bytes,
                        mesh_material: material_handle,
                        index_type: part_data.index_type,
                    })
                }

                let vertex_full_buffer = if let Some(vertex_buffer_full_artifact_id) =
                    vertex_buffer_full_artifact_id
                {
                    Some(handle_factory.make_handle_to_artifact(vertex_buffer_full_artifact_id?))
                } else {
                    None
                };

                let vertex_position_buffer = if let Some(vertex_buffer_position_artifact_id) =
                    vertex_buffer_position_artifact_id
                {
                    Some(
                        handle_factory.make_handle_to_artifact(vertex_buffer_position_artifact_id?),
                    )
                } else {
                    None
                };

                Ok(MeshAdvMeshAssetData {
                    mesh_parts,
                    vertex_full_buffer,
                    vertex_position_buffer,
                })
            },
        )?;

        Ok(MeshAdvMeshPreprocessJobOutput {})
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "658b712f-e498-4c64-a26d-d83d775affb6"]
pub struct MeshAdvMeshBuilder {}

impl Builder for MeshAdvMeshBuilder {
    fn asset_type(&self) -> &'static str {
        MeshAdvMeshAssetRecord::schema_name()
    }

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) -> PipelineResult<()> {
        // Produce an intermediate with all data
        // Produce buffers for various vertex types
        // Some day I might want to look at the materials to decide what vertex buffers should exist

        context.enqueue_job::<MeshAdvMeshPreprocessJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            MeshAdvMeshPreprocessJobInput {
                asset_id: context.asset_id,
            },
        )?;
        Ok(())
    }
}

pub struct MeshAdvAssetPlugin;

impl AssetPlugin for MeshAdvAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        context
            .builder_registry
            .register_handler::<MeshAdvMaterialBuilder>();
        context
            .job_processor_registry
            .register_job_processor::<MeshAdvMaterialJobProcessor>();

        context
            .builder_registry
            .register_handler::<MeshAdvMeshBuilder>();
        context
            .job_processor_registry
            .register_job_processor::<MeshAdvMeshPreprocessJobProcessor>();
    }
}

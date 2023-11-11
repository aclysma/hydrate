pub use super::*;
use glam::Vec3;
use rafx_api::RafxResourceType;

use crate::generated::MeshAdvMeshAssetRecord;
use crate::generated_wrapper::MeshAdvMeshImportedDataRecord;
use crate::push_buffer::PushBuffer;
use demo_types::mesh_adv::*;
use hydrate_model::pipeline::{AssetPlugin, Builder};
use hydrate_model::{
    job_system, BuilderRegistryBuilder, DataContainer, DataSet, HashMap,
    ImporterRegistryBuilder, JobApi, JobEnumeratedDependencies, JobInput, JobOutput, JobProcessor,
    JobProcessorRegistryBuilder, AssetId, Record, SchemaLinker, SchemaSet, SingleObject,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use super::generated::{
    MeshAdvMaterialAssetRecord
};

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

    fn enumerate_dependencies(
        &self,
        _input: &MeshAdvMaterialJobInput,
        _data_set: &DataSet,
        _schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies::default()
    }

    fn run(
        &self,
        input: &MeshAdvMaterialJobInput,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        _dependency_data: &HashMap<AssetId, SingleObject>,
        job_api: &dyn JobApi,
    ) -> MeshAdvMaterialJobOutput {
        //
        // Read asset data
        //
        let data_container = DataContainer::from_dataset(data_set, schema_set, input.asset_id);
        let x = MeshAdvMaterialAssetRecord::default();

        let base_color_factor = x.base_color_factor().get_vec4(&data_container).unwrap();
        let emissive_factor = x.emissive_factor().get_vec3(&data_container).unwrap();

        let metallic_factor = x.metallic_factor().get(&data_container).unwrap();
        let roughness_factor = x.roughness_factor().get(&data_container).unwrap();
        let normal_texture_scale = x.normal_texture_scale().get(&data_container).unwrap();

        let color_texture = x.color_texture().get(&data_container).unwrap();
        let metallic_roughness_texture =
            x.metallic_roughness_texture().get(&data_container).unwrap();
        let normal_texture = x.normal_texture().get(&data_container).unwrap();
        let emissive_texture = x.emissive_texture().get(&data_container).unwrap();
        let shadow_method = x.shadow_method().get(&data_container).unwrap();
        let blend_method = x.blend_method().get(&data_container).unwrap();

        let alpha_threshold = x.alpha_threshold().get(&data_container).unwrap();
        let backface_culling = x.backface_culling().get(&data_container).unwrap();
        let color_texture_has_alpha_channel = x
            .color_texture_has_alpha_channel()
            .get(&data_container)
            .unwrap();

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
        job_system::produce_asset(job_api, input.asset_id, processed_data);

        MeshAdvMaterialJobOutput {}
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
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
    ) {
        //Future: Might produce jobs per-platform
        job_system::enqueue_job::<MeshAdvMaterialJobProcessor>(
            data_set,
            schema_set,
            job_api,
            MeshAdvMaterialJobInput { asset_id },
        );
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

    fn enumerate_dependencies(
        &self,
        input: &MeshAdvMeshPreprocessJobInput,
        _data_set: &DataSet,
        _schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies {
            import_data: vec![input.asset_id],
            upstream_jobs: Vec::default(),
        }
    }

    fn run(
        &self,
        input: &MeshAdvMeshPreprocessJobInput,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<AssetId, SingleObject>,
        job_api: &dyn JobApi,
    ) -> MeshAdvMeshPreprocessJobOutput {
        //
        // Read asset data
        //
        let data_container = DataContainer::from_dataset(data_set, schema_set, input.asset_id);
        let x = MeshAdvMeshAssetRecord::default();
        let mut materials = Vec::default();
        for entry in x
            .material_slots()
            .resolve_entries(&data_container)
            .into_iter()
        {
            let entry = x
                .material_slots()
                .entry(*entry)
                .get(&data_container)
                .unwrap();
            materials.push(entry);
        }

        //
        // Read import data
        //
        let imported_data = &dependency_data[&input.asset_id];
        let data_container = DataContainer::from_single_object(imported_data, schema_set);
        let x = MeshAdvMeshImportedDataRecord::default();

        let mut all_positions = Vec::<glam::Vec3>::with_capacity(1024);
        let mut all_position_indices = Vec::<u32>::with_capacity(8192);

        let mut all_vertices_full = PushBuffer::new(16384);
        let mut all_vertices_position = PushBuffer::new(16384);
        let mut all_indices = PushBuffer::new(16384);

        let mut mesh_part_data = Vec::default();
        for entry in x.mesh_parts().resolve_entries(&data_container).into_iter() {
            let entry = x.mesh_parts().entry(*entry);

            //
            // Get byte slices of all input data for this mesh part
            //
            let positions_bytes = entry.positions().get(&data_container).unwrap();
            let normals_bytes = entry.normals().get(&data_container).unwrap();
            let tex_coords_bytes = entry.texture_coordinates().get(&data_container).unwrap();
            let indices_bytes = entry.indices().get(&data_container).unwrap();

            // let mut tex_coords_pb = PushBuffer::new(tex_coords_bytes.len());
            // let tex_coords_pb_result = tex_coords_pb.push_bytes(&tex_coords_bytes, std::mem::align_of::<[f32; 2]>());
            // let tex_coords_data = tex_coords_pb.into_data();
            // let tex_coords_slice = unsafe {
            //     std::slice::from_raw_parts(tex_coords_data.as_ptr().add(tex_coords_pb_result.offset()), tex_coords_pb_result.size())
            // };

            //
            // Get strongly typed slices of all input data for this mesh part
            //
            let positions = try_cast_u8_slice::<[f32; 3]>(positions_bytes)
                .ok_or("Could not cast due to alignment")
                .unwrap();
            let normals = try_cast_u8_slice::<[f32; 3]>(normals_bytes)
                .ok_or("Could not cast due to alignment")
                .unwrap();
            let tex_coords = try_cast_u8_slice::<[f32; 2]>(tex_coords_bytes)
                .ok_or("Could not cast due to alignment")
                .unwrap();
            let part_indices = try_cast_u8_slice::<u32>(indices_bytes)
                .ok_or("Could not cast due to alignment")
                .unwrap();

            //
            // Part data which mostly contains offsets in the buffers for this part
            //
            let part_data = super::mesh_util::process_mesh_part(
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
        println!("asset id {:?}", input.asset_id);
        let vertex_buffer_full_artifact_id = if !all_vertices_full.is_empty() {
            Some(job_system::produce_artifact(
                job_api,
                input.asset_id,
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
            Some(job_system::produce_artifact(
                job_api,
                input.asset_id,
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
        job_system::produce_asset_with_handles(job_api, input.asset_id, || {
            let mut mesh_parts = Vec::default();
            for (entry, part_data) in x
                .mesh_parts()
                .resolve_entries(&data_container)
                .into_iter()
                .zip(mesh_part_data)
            {
                let entry = x.mesh_parts().entry(*entry);

                let material_slot_index = entry.material_index().get(&data_container).unwrap();
                let material_asset_id = materials[material_slot_index as usize];
                let material_handle =
                    job_system::make_handle_to_default_artifact(job_api, material_asset_id);

                mesh_parts.push(MeshAdvPartAssetData {
                    vertex_full_buffer_offset_in_bytes: part_data
                        .vertex_full_buffer_offset_in_bytes,
                    vertex_full_buffer_size_in_bytes: part_data.vertex_full_buffer_size_in_bytes,
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

            let vertex_full_buffer = vertex_buffer_full_artifact_id
                .map(|x| job_system::make_handle_to_artifact(job_api, x));
            let vertex_position_buffer = vertex_buffer_position_artifact_id
                .map(|x| job_system::make_handle_to_artifact(job_api, x));

            MeshAdvMeshAssetData {
                mesh_parts,
                vertex_full_buffer,
                vertex_position_buffer,
            }
        });

        MeshAdvMeshPreprocessJobOutput {}
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
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
    ) {
        // Produce an intermediate with all data
        // Produce buffers for various vertex types
        // Some day I might want to look at the materials to decide what vertex buffers should exist

        let _preprocess_job_id = job_system::enqueue_job::<MeshAdvMeshPreprocessJobProcessor>(
            data_set,
            schema_set,
            job_api,
            MeshAdvMeshPreprocessJobInput { asset_id },
        );
    }
}

pub struct MeshAdvAssetPlugin;

impl AssetPlugin for MeshAdvAssetPlugin {
    fn setup(
        _schema_linker: &mut SchemaLinker,
        _importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        builder_registry.register_handler::<MeshAdvMaterialBuilder>();
        job_processor_registry.register_job_processor::<MeshAdvMaterialJobProcessor>();

        builder_registry.register_handler::<MeshAdvMeshBuilder>();
        job_processor_registry.register_job_processor::<MeshAdvMeshPreprocessJobProcessor>();
    }
}

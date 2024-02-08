use demo_types::gpu_buffer::GpuBufferBuiltData;
use demo_types::image::GpuImageAssetData;
use demo_types::mesh_adv::{
    MeshAdvBufferAssetData, MeshAdvMaterialAssetData, MeshAdvMaterialData, MeshAdvMeshAssetData,
};
use demo_types::simple_data::{Transform, TransformRef};
use hydrate::loader::Handle;
use std::path::PathBuf;

mod example_image_asset;
use example_image_asset::*;

pub fn build_data_source_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/build_data"
    ))
}

fn main() {
    let _client = profiling::tracy_client::Client::start();
    profiling::register_thread!("main");

    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    //
    // Set up storage for loaded assets
    //
    let mut loader = hydrate::loader::AssetManager::new(build_data_source_path()).unwrap();
    loader.add_storage_with_loader::<GpuImageAssetData, GpuImageAsset, GpuImageLoader>(Box::new(
        GpuImageLoader,
    ));
    loader.add_storage::<GpuBufferBuiltData>();
    loader.add_storage::<Transform>();
    loader.add_storage::<TransformRef>();
    loader.add_storage::<MeshAdvMeshAssetData>();
    loader.add_storage::<MeshAdvBufferAssetData>();
    loader.add_storage::<MeshAdvMaterialAssetData>();
    loader.add_storage::<MeshAdvMaterialData>();

    //
    // Request a few assets (including an image, which will take time to load)
    //
    let mut load_handle_transform_ref: Option<Handle<TransformRef>> =
        Some(loader.load_asset_symbol_name("assets://test_transform_ref"));
    let load_handle_mesh: Handle<MeshAdvMeshAssetData> =
        loader.load_asset_symbol_name("assets://sphere.glb.mesh_Sphere");
    let load_handle_image: Handle<GpuImageAsset> =
        loader.load_asset_symbol_name("assets://large_test/rocks/materials/Stones_Big1_a.tif");

    //
    // Game Loop
    //
    let mut loop_count = 0;
    loop {
        //
        // This represents the rest of the game thread and the required update() on the loader
        //
        std::thread::sleep(std::time::Duration::from_millis(200));
        loader.update();

        //
        // After we do 50 iterations, drop this handle to demonstrate unloading
        //
        loop_count += 1;
        println!("-------- frame {} ----------", loop_count);
        // if loop_count > 20 {
        //     println!("UNLOAD THE TRANSFORM REF");
        //     load_handle_transform_ref = None;
        // }

        //
        // print info about the transform ref object and the asset it references
        //
        if let Some(load_handle_transform_ref) = &load_handle_transform_ref {
            let data = load_handle_transform_ref.artifact(loader.storage());
            if let Some(data) = data {
                let data_inner = data.transform.artifact(loader.storage());
                println!("load_handle_transform_ref loaded {:?}", data);
                println!("load_handle_transform_ref inner loaded {:?}", data_inner);
            } else {
                println!("load_handle_transform_ref not loaded");
            }
        } else {
            println!("load_handle_transform_ref unloaded");
        }

        //
        // print info about the mesh object (and some of the assets indirectly loaded by it)
        //
        let data = load_handle_mesh.artifact(loader.storage());
        if let Some(data) = data {
            let data_full_vb = data
                .vertex_position_buffer
                .as_ref()
                .map(|x| x.artifact(loader.storage()).unwrap());
            let data_position_vb = data
                .vertex_position_buffer
                .as_ref()
                .map(|x| x.artifact(loader.storage()).unwrap());
            println!("load_handle_mesh loaded {:?}", data.mesh_parts);
            if let Some(data_full_vb) = data_full_vb {
                println!("full vb {:?}", data_full_vb.data.len());
            }

            if let Some(data_position_vb) = data_position_vb {
                println!("position vb {:?}", data_position_vb.data.len());
            }
        } else {
            println!("load_handle_mesh not loaded");
        }

        //
        // print info about the image object
        //
        let data = load_handle_image.artifact(loader.storage());
        if let Some(data) = data {
            println!("load_handle_image loaded {:?}", data.image_bytes.len());
        } else {
            println!("load_handle_image not loaded");
        }

        profiling::finish_frame!();
    }
}

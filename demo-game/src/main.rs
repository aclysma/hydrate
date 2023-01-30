use demo_types::image::ImageBuiltData;
use hydrate::loader::Handle;
use hydrate::model::ObjectId;
use std::path::PathBuf;
use demo_types::gltf::{GltfBuiltMaterialData, GltfBuiltMeshData};
use demo_types::simple_data::Transform;

pub fn build_data_source_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/build_data"
    ))
}

fn main() {
    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut loader = hydrate::loader::AssetManager::new(build_data_source_path()).unwrap();
    loader.add_storage::<ImageBuiltData>();
    loader.add_storage::<GltfBuiltMaterialData>();
    loader.add_storage::<GltfBuiltMeshData>();
    loader.add_storage::<Transform>();

    let load_handle_image: Handle<ImageBuiltData> = loader.load_asset(ObjectId(
        uuid::Uuid::parse_str("df737bdbfc014fc5929a5e7a0d0f1281")
            .unwrap()
            .as_u128(),
    ));


    let load_handle_mesh: Handle<GltfBuiltMeshData> = loader.load_asset(ObjectId(
        uuid::Uuid::parse_str("ced7b55b693240b281feed577fcc4cba")
            .unwrap()
            .as_u128(),
    ));


    let load_handle_material: Handle<GltfBuiltMaterialData> = loader.load_asset(ObjectId(
        uuid::Uuid::parse_str("ccd1f453d6224b2fab9bc8021a6c7dde")
            .unwrap()
            .as_u128(),
    ));



    let load_handle_material2: Handle<GltfBuiltMaterialData> = loader.load_asset(ObjectId(
        uuid::Uuid::parse_str("ccd1f453d6224b2fab9bc8021a6c7dde")
            .unwrap()
            .as_u128(),
    ));


    let load_handle_transform: Handle<Transform> = loader.load_asset(ObjectId(
        uuid::Uuid::parse_str("dece7fdfc3fc4691b93101c0b25cb822")
            .unwrap()
            .as_u128(),
    ));



    loop {
        std::thread::sleep(std::time::Duration::from_millis(15));
        loader.update();

        let data = load_handle_image.asset(loader.storage());
        if let Some(data) = data {
            //println!("{} {}", data.width, data.height);
        } else {
            println!("not loaded");
        }

        let data = load_handle_mesh.asset(loader.storage());
        if let Some(data) = data {
            //println!("mesh loaded");
        } else {
            println!("mesh not loaded");
        }

        let data = load_handle_material.asset(loader.storage());
        if let Some(data) = data {
            //println!("material loaded");
        } else {
            println!("material not loaded");
        }

        let data = load_handle_material2.asset(loader.storage());
        if let Some(data) = data {
            //println!("material loaded");
        } else {
            println!("material not loaded");
        }

        let data = load_handle_transform.asset(loader.storage());
        if let Some(data) = data {
            //println!("transform loaded {:?}", data);
        } else {
            println!("material not loaded");
        }
    }
}

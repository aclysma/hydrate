use demo_types::image::ImageBuiltData;
use hydrate::loader::Handle;
use hydrate::model::ObjectId;
use std::path::PathBuf;

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

    let load_handle: Handle<ImageBuiltData> = loader.load_asset(ObjectId(
        uuid::Uuid::parse_str("df737bdbfc014fc5929a5e7a0d0f1281")
            .unwrap()
            .as_u128(),
    ));
    loop {
        std::thread::sleep(std::time::Duration::from_millis(15));
        loader.update();

        let data = load_handle.asset(loader.storage());
        if let Some(data) = data {
            println!("{} {}", data.width, data.height);
        } else {
            println!("not loaded");
        }
    }
}

use demo_plugins::{
    BlenderMaterialAssetPlugin, GlslAssetPlugin, GltfAssetPlugin, ImageAssetPlugin,
    SimpleDataAssetPlugin,
};
use std::path::PathBuf;

fn schema_def_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/schema"))
}

fn schema_cache_file_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/schema_cache_file.json"
    ))
}

fn asset_id_based_data_source_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/assets_id_based"))
}

fn asset_path_based_data_source_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/assets_path_based"))
}

pub fn import_data_source_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/import_data"))
}

pub fn build_data_source_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/build_data"))
}

fn main() {
    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut linker = hydrate::model::SchemaLinker::default();

    let mut asset_engine_builder = hydrate::pipeline::AssetEngineBuilder::new()
        .register_plugin::<ImageAssetPlugin>(&mut linker)
        .register_plugin::<BlenderMaterialAssetPlugin>(&mut linker)
        .register_plugin::<GlslAssetPlugin>(&mut linker)
        .register_plugin::<GltfAssetPlugin>(&mut linker)
        .register_plugin::<SimpleDataAssetPlugin>(&mut linker);

    //TODO: Take a config file
    //TODO: Support N sources using path nodes
    let db_state = hydrate::editor::DbState::load_or_init_empty(
        linker,
        &asset_id_based_data_source_path(),
        &asset_path_based_data_source_path(),
        &schema_def_path(),
        &schema_cache_file_path(),
    );
    let asset_engine = asset_engine_builder.build(
        &db_state.editor_model,
        import_data_source_path(),
        build_data_source_path(),
    );

    //Headless
    //asset_engine.update(&db_state.editor_model);

    hydrate::editor::run(db_state, asset_engine);
}

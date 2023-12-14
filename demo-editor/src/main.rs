mod inspectors;

use demo_plugins::{
    BlenderMaterialAssetPlugin, BlenderMeshAssetPlugin, GlslAssetPlugin, GltfAssetPlugin,
    GpuBufferAssetPlugin, GpuImageAssetPlugin, MeshAdvAssetPlugin, SimpleDataAssetPlugin,
};
use hydrate::pipeline::{HydrateProjectConfiguration};
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    profiling::tracy_client::Client::start();
    profiling::register_thread!("main");

    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_module("wgpu_core", log::LevelFilter::Info)
        .filter_level(log::LevelFilter::Debug)
        .init();

    //
    // Load initial configuration
    //
    let project_configuration = HydrateProjectConfiguration::locate_project_file(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap();

    let asset_plugin_registry =
        hydrate::pipeline::AssetPluginRegistryBuilders::new()
            .register_plugin::<GpuBufferAssetPlugin>()
            .register_plugin::<GpuImageAssetPlugin>()
            .register_plugin::<BlenderMaterialAssetPlugin>()
            .register_plugin::<BlenderMeshAssetPlugin>()
            .register_plugin::<MeshAdvAssetPlugin>()
            .register_plugin::<GlslAssetPlugin>()
            .register_plugin::<GltfAssetPlugin>()
            .register_plugin::<SimpleDataAssetPlugin>();


    let mut editor = hydrate::editor::Editor::new(project_configuration, asset_plugin_registry);

    let schema_set = editor.schema_set().clone();
    inspectors::register_inspectors(&schema_set, editor.inspector_registry_mut());

    editor.run()
}

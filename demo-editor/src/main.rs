mod inspectors;

use demo_plugins::generated::Vec3Record;
use demo_plugins::{
    BlenderMaterialAssetPlugin, BlenderMeshAssetPlugin, GlslAssetPlugin, GltfAssetPlugin,
    GpuBufferAssetPlugin, GpuImageAssetPlugin, MeshAdvAssetPlugin, SimpleDataAssetPlugin,
};
use egui::Ui;
use hydrate::editor::inspector_system;
use hydrate::editor::inspector_system::InspectorContext;
use hydrate::model::{
    AssetPathCache, EditorModelWithCache, Record, Schema, SchemaDefRecordFieldMarkup, SchemaRecord,
};
use hydrate::pipeline::AssetEngine;
use std::path::PathBuf;

fn schema_def_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/schema"
    ))
}

fn schema_cache_file_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/schema_cache_file.json"
    ))
}

fn asset_id_based_asset_source_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/assets_id_based"
    ))
}

fn asset_path_based_data_source_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/assets_path_based"
    ))
}

pub fn import_data_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/import_data"
    ))
}

pub fn build_data_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/build_data"
    ))
}

pub fn job_data_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/job_data"
    ))
}

fn main() -> eframe::Result<()> {
    profiling::tracy_client::Client::start();
    profiling::register_thread!("main");

    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_module("wgpu_core", log::LevelFilter::Info)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let (db_state, asset_engine) = {
        profiling::scope!("Hydrate Initialization");
        let mut linker = hydrate::model::SchemaLinker::default();

        let asset_plugin_registration_helper =
            hydrate::pipeline::AssetPluginRegistrationHelper::new()
                .register_plugin::<GpuBufferAssetPlugin>(&mut linker)
                .register_plugin::<GpuImageAssetPlugin>(&mut linker)
                .register_plugin::<BlenderMaterialAssetPlugin>(&mut linker)
                .register_plugin::<BlenderMeshAssetPlugin>(&mut linker)
                .register_plugin::<MeshAdvAssetPlugin>(&mut linker)
                .register_plugin::<GlslAssetPlugin>(&mut linker)
                .register_plugin::<GltfAssetPlugin>(&mut linker)
                .register_plugin::<SimpleDataAssetPlugin>(&mut linker);

        //TODO: Take a config file
        //TODO: Support N sources using path nodes
        let schema_set = {
            profiling::scope!("Load Schema");
            hydrate::editor::DbState::load_schema(
                linker,
                &[&schema_def_path()],
                &schema_cache_file_path(),
            )
        };

        let (importer_registry, builder_registry, job_processor_registry) =
            asset_plugin_registration_helper.finish(&schema_set);

        let mut imports_to_queue = Vec::default();
        let mut db_state = hydrate::editor::DbState::load(
            &schema_set,
            &importer_registry,
            &asset_id_based_asset_source_path(),
            &asset_path_based_data_source_path(),
            &schema_cache_file_path(),
            &mut imports_to_queue,
        );

        let asset_path_cache = AssetPathCache::build(&db_state.editor_model);
        let mut editor_model_with_cache = EditorModelWithCache {
            editor_model: &mut db_state.editor_model,
            asset_path_cache: &asset_path_cache,
        };

        let mut asset_engine = {
            profiling::scope!("Create Asset Engine");
            AssetEngine::new(
                &schema_set,
                importer_registry,
                builder_registry,
                job_processor_registry,
                &mut editor_model_with_cache,
                import_data_path(),
                job_data_path(),
                build_data_path(),
            )
        };

        {
            profiling::scope!("Queue import operations");
            for import_to_queue in imports_to_queue {
                //println!("Queueing import operation {:?}", import_to_queue);
                asset_engine.queue_import_operation(
                    import_to_queue.requested_importables,
                    import_to_queue.importer_id,
                    import_to_queue.source_file_path,
                    import_to_queue.import_type,
                );
            }
        }

        //Headless
        profiling::scope!("First asset engine update");
        //TODO: This update should be done after app is open and UI should show
        //TODO: Ideally the scan for assets/path locations would also be done after the application has been opened and UI can show
        //asset_engine.update(&mut editor_model_with_cache).unwrap();
        (db_state, asset_engine)
    };

    let inspector_registry = crate::inspectors::create_registry(db_state.editor_model.schema_set());
    hydrate::editor::run(db_state, asset_engine, inspector_registry)
}

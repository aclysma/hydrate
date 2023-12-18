mod app;
mod ui;

mod db_state;
pub use db_state::DbState;

mod persistent_app_state;

pub mod action_queue;

mod egui_debug_ui;
mod fonts;
mod modal_action;
mod ui_state;
mod image_loader;

pub use egui;
pub use egui_extras;
use hydrate_model::{AssetPathCache, EditorModelWithCache, SchemaSet};

use crate::app::HydrateEditorApp;
pub use crate::ui::components::inspector_system;
use hydrate_model::pipeline::{AssetEngine, AssetPluginRegistryBuilders, HydrateProjectConfiguration, ImportJobToQueue};
use crate::inspector_system::InspectorRegistry;

pub struct Editor {
    db_state: DbState,
    asset_engine: AssetEngine,
    inspector_registry: InspectorRegistry,
}

impl Editor {
    pub fn inspector_registry_mut(&mut self) -> &mut InspectorRegistry {
        &mut self.inspector_registry
    }

    pub fn schema_set(&self) -> &SchemaSet {
        self.db_state.editor_model.schema_set()
    }

    pub fn new(
        project_configuration: HydrateProjectConfiguration,
        asset_plugin_registry: AssetPluginRegistryBuilders
    ) -> Self {
        profiling::scope!("Hydrate Initialization");

        let schema_set = {
            profiling::scope!("Load Schema");
            DbState::load_schema(
                &project_configuration,
            )
        };

        let registries =
            asset_plugin_registry.finish(&schema_set);

        let mut import_job_to_queue = ImportJobToQueue::default();
        let mut db_state = DbState::load(
            &schema_set,
            &registries.importer_registry,
            &project_configuration,
            &mut import_job_to_queue,
        );

        let asset_path_cache = AssetPathCache::build(&db_state.editor_model).unwrap();
        let mut editor_model_with_cache = EditorModelWithCache {
            editor_model: &mut db_state.editor_model,
            asset_path_cache: &asset_path_cache,
        };

        let mut asset_engine = {
            profiling::scope!("Create Asset Engine");
            AssetEngine::new(
                &schema_set,
                registries,
                &mut editor_model_with_cache,
                &project_configuration,
            )
        };

        {
            profiling::scope!("Queue import operations");
            if !import_job_to_queue.is_empty() {
                asset_engine.queue_import_operation(import_job_to_queue);
            }

        }
        let inspector_registry = InspectorRegistry::default();

        Self {
            db_state,
            asset_engine,
            inspector_registry
        }
    }

    pub fn run(self) -> eframe::Result<()> {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([450.0, 300.0]),
            //.with_min_inner_size([900.0, 650.0]),
            follow_system_theme: false,
            default_theme: eframe::Theme::Dark,
            centered: true,
            window_builder: Some(Box::new(|mut builder| {
                builder.position = Some(egui::pos2(1000.0, 0.0));
                builder.inner_size = Some(egui::vec2(700.0, 500.0));
                builder
            })),
            ..Default::default()
        };

        eframe::run_native(
            "Hydrate Editor",
            native_options,
            Box::new(|cc| {
                Box::new(HydrateEditorApp::new(
                    cc,
                    self.db_state,
                    self.asset_engine,
                    self.inspector_registry,
                ))
            }),
        )
    }
}

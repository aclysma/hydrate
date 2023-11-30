mod ui;
mod app;

mod db_state;
pub use db_state::DbState;

mod persistent_app_state;

mod action_queue;

mod ui_state;
mod modal_action;
mod egui_debug_ui;

use hydrate_model::pipeline::AssetEngine;
use crate::app::HydrateEditorApp;

pub fn run(db_state: DbState, asset_engine: AssetEngine) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0]),
            //.with_min_inner_size([900.0, 650.0]),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "Hydrate Editor",
        native_options,
        Box::new(|cc| Box::new(HydrateEditorApp::new(cc, db_state, asset_engine))),
    )
}
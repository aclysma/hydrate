mod ui;
mod app;

mod db_state;
pub use db_state::DbState;

mod persistent_app_state;

mod action_queue;

mod ui_state;
mod modal_action;

use hydrate_model::pipeline::AssetEngine;
use crate::app::HydrateEditorApp;

pub fn run(db_state: DbState, asset_engine: AssetEngine) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(HydrateEditorApp::new(cc, db_state, asset_engine))),
    )
}
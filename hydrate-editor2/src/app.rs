use egui::{FontDefinitions, Frame};
use egui::epaint::text::FontsImpl;
use hydrate_model::{AssetId, HashSet};
use hydrate_model::pipeline::AssetEngine;
use crate::action_queue::UIActionQueueReceiver;
use crate::db_state::DbState;
use crate::modal_action::{ModalAction, ModalActionControlFlow};
use crate::ui_state::EditorModelUiState;
use crate::persistent_app_state::PersistentAppState;
use crate::ui::components::AssetGalleryUiState;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct HydrateEditorApp {
    db_state: DbState,
    asset_engine: AssetEngine,
    persistent_state: PersistentAppState,
    asset_gallery_ui_state: AssetGalleryUiState,
    editor_model_ui_state: EditorModelUiState,
    action_queue: UIActionQueueReceiver,
    modal_action: Option<Box<dyn ModalAction>>,
}

impl HydrateEditorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, db_state: DbState, asset_engine: AssetEngine) -> Self {
        let persistent_state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            PersistentAppState::default()
        };

        cc.egui_ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        });

        HydrateEditorApp {
            db_state,
            asset_engine,
            persistent_state,
            asset_gallery_ui_state: Default::default(),
            editor_model_ui_state: EditorModelUiState::default(),
            action_queue: UIActionQueueReceiver::default(),
            modal_action: None,
        }
    }
}

impl eframe::App for HydrateEditorApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.persistent_state);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        // Generate some profiling info
        profiling::scope!("Main Thread");

        let action_queue_sender = self.action_queue.sender();

        self.editor_model_ui_state.update(&self.db_state.editor_model);

        let clear_modal_action = if let Some(modal_action) = &mut self.modal_action {
            let control_flow = modal_action.draw(ctx, &self.editor_model_ui_state, &mut self.asset_engine, &action_queue_sender);
            match control_flow {
                ModalActionControlFlow::Continue => false,
                ModalActionControlFlow::End => true,
            }
        } else {
            false
        };

        if clear_modal_action {
            self.modal_action = None;
        }

        let default_font = ctx.style().text_styles.get(&egui::TextStyle::Body).unwrap().clone();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.set_enabled(self.modal_action.is_none());

            // The top panel is often a good place for a menu bar:
            crate::ui::components::draw_main_menu_bar(ctx, ui);

        });

        egui::SidePanel::right("right_panel").resizable(true).show(ctx, |ui| {
            ui.set_enabled(self.modal_action.is_none());
            ui.separator();
            ui.label("detail panel");
            if !self.asset_gallery_ui_state.selected_assets.is_empty() {
                for selected in &self.asset_gallery_ui_state.selected_assets {
                    //TODO: Temp hack
                    crate::ui::components::draw_inspector(ui, &self.editor_model_ui_state, *selected, self.db_state.editor_model.schema_set());
                    break;
                }
            }

        });

        egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            ui.set_enabled(self.modal_action.is_none());
            ui.separator();
            ui.label("Asset tree");
        });

        //let mut frame = Frame::central_panel(&*ctx.style());
        egui::CentralPanel::default()/*.frame(frame)*/.show(ctx, |ui| {
            ui.set_enabled(self.modal_action.is_none());
            let mut fonts = FontsImpl::new(1.0, 1024, FontDefinitions::default());
            crate::ui::components::draw_asset_gallery(
                ui,
                &mut fonts,
                &default_font,
                &self.editor_model_ui_state,
                &mut self.asset_gallery_ui_state,
                &action_queue_sender
            );
        });

        self.action_queue.process(&mut self.db_state.editor_model, &mut self.asset_engine, &self.editor_model_ui_state, &mut self.modal_action);

        // Finish the frame.
        profiling::finish_frame!();
    }
}

use std::time::Duration;
use crate::action_queue::{UIAction, UIActionQueueReceiver};
use crate::db_state::DbState;
use crate::egui_debug_ui::EguiDebugUiState;
use crate::modal_action::{ModalAction, ModalActionControlFlow, ModalContext};
use crate::persistent_app_state::PersistentAppState;
use crate::ui::components::inspector_system::{InspectorRegistry};
use crate::ui::components::{AssetGalleryUiState, AssetTreeUiState, InspectorUiState};
use crate::ui::modals::ImportFilesModal;
use crate::ui_state::EditorModelUiState;
use egui::epaint::text::FontsImpl;
use egui::{FontDefinitions, ViewportCommand};
use hydrate_model::pipeline::{AssetEngine, AssetEngineState, HydrateProjectConfiguration};
use hydrate_model::EditorModelWithCache;

#[derive(Default)]
pub struct UiState {
    pub asset_tree_ui_state: AssetTreeUiState,
    pub asset_gallery_ui_state: AssetGalleryUiState,
    pub inspector_ui_state: InspectorUiState,
    pub editor_model_ui_state: EditorModelUiState,
    pub egui_debug_ui_state: EguiDebugUiState,
    pub user_confirmed_should_quit: bool,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct HydrateEditorApp {
    db_state: DbState,
    asset_engine: AssetEngine,
    persistent_state: PersistentAppState,
    ui_state: UiState,
    action_queue: UIActionQueueReceiver,
    modal_action: Option<Box<dyn ModalAction>>,
    inspector_registry: InspectorRegistry,
}

impl HydrateEditorApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        db_state: DbState,
        asset_engine: AssetEngine,
        inspector_registry: InspectorRegistry,
    ) -> Self {
        let persistent_state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            PersistentAppState::default()
        };

        cc.egui_ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            // style.text_styles.insert("CustomStyle", egui::FontId {
            //
            // })
        });

        let fonts = crate::fonts::load_custom_fonts();
        cc.egui_ctx.set_fonts(fonts);

        HydrateEditorApp {
            db_state,
            asset_engine,
            persistent_state,
            ui_state: Default::default(),
            action_queue: UIActionQueueReceiver::default(),
            modal_action: None,
            inspector_registry,
        }
    }
}

impl eframe::App for HydrateEditorApp {
    /// Called by the frame work to save state before shutdown.
    fn save(
        &mut self,
        storage: &mut dyn eframe::Storage,
    ) {
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

        self.ui_state
            .editor_model_ui_state
            .update(&self.db_state.editor_model);

        let action_queue_sender = self.action_queue.sender();

        if ctx.input(|x| x.viewport().close_requested()) {
            if !self.ui_state.user_confirmed_should_quit {
                // If we haven't confirmed quit, intercept and send through a "confirm to quit" flow
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                action_queue_sender.queue_action(UIAction::Quit);
            }
        }

        ctx.input(|input| {
            if !input.raw.dropped_files.is_empty() {
                let dropped_files: Vec<_> = input
                    .raw
                    .dropped_files
                    .iter()
                    .map(|x| x.path.clone().unwrap())
                    .collect();
                action_queue_sender.try_set_modal_action(ImportFilesModal::new(
                    dropped_files,
                    self.asset_engine.importer_registry(),
                ));
            }
        });

        let asset_engine_state = {
            let mut editor_model_with_cache = EditorModelWithCache {
                editor_model: &mut self.db_state.editor_model,
                asset_path_cache: &self.ui_state.editor_model_ui_state.asset_path_cache,
            };

            self.asset_engine
                .update(&mut editor_model_with_cache)
                .unwrap()
        };

        match asset_engine_state {
            AssetEngineState::Idle => ctx.request_repaint_after(Duration::from_millis(1000)),
            _ => ctx.request_repaint(),
        }

        let clear_modal_action = if let Some(modal_action) = &mut self.modal_action {
            let context = ModalContext {
                egui_ctx: ctx,
                ui_state: &self.ui_state.editor_model_ui_state,
                asset_engine: &mut self.asset_engine,
                db_state: &self.db_state,
                action_queue: &action_queue_sender,
            };
            let control_flow = modal_action.draw(context);
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

        let default_font = ctx
            .style()
            .text_styles
            .get(&egui::TextStyle::Body)
            .unwrap()
            .clone();

        //if self.asset_engine.update()

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            match asset_engine_state {
                AssetEngineState::Importing(import_state) => {
                    let text = format!("Importing {}/{} assets", import_state.completed_job_count, import_state.total_job_count);
                    ui.add(egui::ProgressBar::new(import_state.completed_job_count as f32 / import_state.total_job_count as f32).text(text));
                },
                AssetEngineState::Building(build_state) => {
                    let text = format!("Building {}/{} assets", build_state.completed_job_count, build_state.total_job_count);
                    ui.add(egui::ProgressBar::new(build_state.completed_job_count as f32 / build_state.total_job_count as f32).text(text));

                }
                AssetEngineState::Idle => {
                    ui.horizontal(|ui| {
                        ui.label("Ready");

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            //let needs_build = self.asset_engine.needs_build();
                            let needs_build = true;
                            if ui.add_enabled(needs_build, egui::Button::new("Build")).clicked() {
                                self.asset_engine.queue_build_all();
                            }
                        });

                        // ui.layout(egui::Layout::right_to_left(egui::Align::TOP))
                        //
                        // ui.label("Ready");
                        // ui.separator();
                    });
                }
            }
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.set_enabled(self.modal_action.is_none());

            // The top panel is often a good place for a menu bar:
            crate::ui::components::draw_main_menu_bar(
                ctx,
                ui,
                &mut self.ui_state.egui_debug_ui_state,
                &action_queue_sender,
            );
        });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_enabled(self.modal_action.is_none());

                //TODO: Temp hack
                let mut first_selected = None;
                for selected in &self.ui_state.asset_gallery_ui_state.selected_assets {
                    first_selected = Some(*selected);
                    break;
                }

                crate::ui::components::draw_inspector(
                    &self.db_state.project_configuration,
                    ui,
                    &self.db_state.editor_model,
                    &action_queue_sender,
                    &self.ui_state.editor_model_ui_state,
                    first_selected,
                    &self.inspector_registry,
                );
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_enabled(self.modal_action.is_none());

                crate::ui::components::draw_asset_tree(
                    ui,
                    &self.db_state.editor_model,
                    &action_queue_sender,
                    &self.ui_state.editor_model_ui_state,
                    &mut self.ui_state.asset_tree_ui_state,
                );
            });

        //let mut frame = Frame::central_panel(&*ctx.style());
        egui::CentralPanel::default() /*.frame(frame)*/
            .show(ctx, |ui| {
                ui.set_enabled(self.modal_action.is_none());

                let mut fonts = FontsImpl::new(1.0, 1024, FontDefinitions::default());
                crate::ui::components::draw_asset_gallery(
                    ui,
                    &mut fonts,
                    &default_font,
                    &self.db_state,
                    &self.ui_state.editor_model_ui_state,
                    &self.ui_state.asset_tree_ui_state,
                    &mut self.ui_state.asset_gallery_ui_state,
                    &action_queue_sender,
                );
            });

        self.action_queue.process(
            &self.db_state.project_configuration,
            &mut self.db_state.editor_model,
            &mut self.asset_engine,
            &mut self.ui_state,
            &mut self.modal_action,
            ctx,
        );

        super::egui_debug_ui::show_egui_debug_ui(ctx, &mut self.ui_state.egui_debug_ui_state);

        // Finish the frame.
        profiling::finish_frame!();
    }
}

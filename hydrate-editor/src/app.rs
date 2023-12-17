use std::sync::Arc;
use crate::action_queue::{UIAction, UIActionQueueReceiver, UIActionQueueSender};
use crate::db_state::DbState;
use crate::egui_debug_ui::EguiDebugUiState;
use crate::modal_action::{ModalAction, ModalActionControlFlow, ModalContext};
use crate::persistent_app_state::PersistentAppState;
use crate::ui::components::inspector_system::{InspectorRegistry};
use crate::ui::components::{AssetGalleryUiState, AssetTreeUiState, InspectorUiState};
use crate::ui::modals::ImportFilesModal;
use crate::ui_state::EditorModelUiState;
use egui::{Ui, ViewportCommand, WidgetText};
use egui_tiles::{SimplificationOptions, TileId};
use hydrate_model::pipeline::{AssetEngine, AssetEngineState};
use hydrate_model::EditorModelWithCache;
use crate::image_loader::{ThumbnailImageLoader, AssetThumbnailTextureLoader};

#[derive(Debug, Copy, Clone, PartialEq)]
enum DockingPanelKind {
    AssetTree,
    AssetGallery,
    Inspector,
    ErrorList
}

struct MainUiContext<'a> {
    action_queue_sender: &'a UIActionQueueSender,
    db_state: &'a mut DbState,
    ui_state: &'a mut UiState,
    inspector_registry: &'a InspectorRegistry,
    thumbnail_image_loader: &'a ThumbnailImageLoader,
}

impl<'a> egui_tiles::Behavior<DockingPanelKind> for MainUiContext<'a> {
    // Ensures all windows have tabs
    fn simplification_options(&self) -> SimplificationOptions {
        let mut simplification_options = SimplificationOptions::default();
        simplification_options.all_panes_must_have_tabs = true;
        simplification_options
    }

    fn tab_title_for_pane(&mut self, pane: &DockingPanelKind) -> WidgetText {
        format!("{:?}", pane).into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut DockingPanelKind) -> egui_tiles::UiResponse {
        match *pane {
            DockingPanelKind::AssetTree => draw_asset_tree(ui, self),
            DockingPanelKind::AssetGallery => draw_asset_gallery(ui, self),
            DockingPanelKind::Inspector => draw_property_inspector(ui, self),
            DockingPanelKind::ErrorList => {
                ui.label("todo");
            }
        }

        egui_tiles::UiResponse::None
    }
}

fn draw_asset_tree(ui: &mut egui::Ui, ui_context: &mut MainUiContext) {
    crate::ui::components::draw_asset_tree(
        ui,
        &ui_context.db_state.editor_model,
        &ui_context.thumbnail_image_loader,
        ui_context.action_queue_sender,
        &ui_context.ui_state.editor_model_ui_state,
        &mut ui_context.ui_state.asset_tree_ui_state,
    );
}

fn draw_asset_gallery(ui: &mut egui::Ui, ui_context: &mut MainUiContext) {
    crate::ui::components::draw_asset_gallery(
        ui,
        ui_context.db_state,
        &ui_context.ui_state.editor_model_ui_state,
        &ui_context.ui_state.asset_tree_ui_state,
        &mut ui_context.ui_state.asset_gallery_ui_state,
        ui_context.action_queue_sender,
        ui_context.thumbnail_image_loader,
    );
}

fn draw_property_inspector(ui: &mut egui::Ui, ui_context: &mut MainUiContext) {
    crate::ui::components::draw_inspector(
        ui,
        &ui_context.db_state.editor_model,
        &ui_context.action_queue_sender,
        &ui_context.ui_state.editor_model_ui_state,
        &mut ui_context.ui_state.inspector_ui_state,
        ui_context.ui_state.asset_gallery_ui_state.selected_assets(),
        ui_context.ui_state.asset_gallery_ui_state.primary_selected_asset(),
        ui_context.inspector_registry,
        ui_context.thumbnail_image_loader,
    );
}

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
    thumbnail_image_loader: Arc<ThumbnailImageLoader>,
    dock_state: egui_tiles::Tree<DockingPanelKind>,
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

        let image_loader = Arc::new(ThumbnailImageLoader::new(
            db_state.editor_model.schema_set(),
            asset_engine.thumbnail_provider_registry(),
            asset_engine.thumbnail_system_state())
        );
        cc.egui_ctx.add_image_loader(image_loader.clone());

        let texture_loader = Arc::new(AssetThumbnailTextureLoader::new());
        cc.egui_ctx.add_texture_loader(texture_loader);

        let mut tiles = egui_tiles::Tiles::default();

        // let mut center_tabs = vec![];
        // center_tabs.push(tiles.insert_pane(DockingPanelKind::AssetGallery));
        // center_tabs.push(tiles.insert_pane(DockingPanelKind::ErrorList));
        // let central_tabs = tiles.insert_tab_tile(center_tabs);

        let mut root_tabs = vec![];
        root_tabs.push(tiles.insert_pane(DockingPanelKind::AssetTree));
        //root_tabs.push(central_tabs);
        root_tabs.push(tiles.insert_pane(DockingPanelKind::AssetGallery));
        root_tabs.push(tiles.insert_pane(DockingPanelKind::Inspector));
        let root = tiles.insert_tab_tile(root_tabs);

        let dock_state = egui_tiles::Tree::new("tree", root, tiles);

        HydrateEditorApp {
            db_state,
            asset_engine,
            persistent_state,
            ui_state: Default::default(),
            action_queue: UIActionQueueReceiver::default(),
            modal_action: None,
            inspector_registry,
            thumbnail_image_loader: image_loader.clone(),
            dock_state
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
        profiling::scope!("Main Thread");

        //
        // Main work here is caching information the rest of the UI is going to want to have (AssetId -> string path)
        //
        self.ui_state
            .editor_model_ui_state
            .update(&self.db_state.editor_model);

        //
        // Intercept window close to ask to save changes
        //
        let action_queue_sender = self.action_queue.sender();
        if ctx.input(|x| x.viewport().close_requested()) {
            if !self.ui_state.user_confirmed_should_quit {
                // If we haven't confirmed quit, intercept and send through a "confirm to quit" flow
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                action_queue_sender.queue_action(UIAction::Quit);
            }
        }

        //
        // Intercept files being dropped into the window to start an import UI flow
        //
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

        //
        // Handle any import, build, and thumbnail generation operations
        //
        let asset_engine_state = {
            let mut editor_model_with_cache = EditorModelWithCache {
                editor_model: &mut self.db_state.editor_model,
                asset_path_cache: &self.ui_state.editor_model_ui_state.asset_path_cache,
            };

            self.asset_engine
                .update(&mut editor_model_with_cache)
                .unwrap()
        };

        //
        // If we are in the middle of a build, we should repaint the UI so progress bars etc. update and any work that
        // is main-thread-only can happen promptly
        //
        match asset_engine_state {
            // App still seems to spin even when just requesting a 1hz update
            //AssetEngineState::Idle => ctx.request_repaint_after(Duration::from_millis(1000)),
            AssetEngineState::Idle => {},
            _ => ctx.request_repaint(),
        }

        //
        // If we have an active modal window open, draw it now
        //
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

        //
        // Draw the status bar, which is mainly a progress indicator for import/build
        //
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
                    });
                }
            }
        });

        //
        // Draw the menu bar on top
        //
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

        //
        // Now the content all in dockable tabs
        //
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui_context = MainUiContext {
                action_queue_sender: &action_queue_sender,
                db_state: &mut self.db_state,
                ui_state: &mut self.ui_state,
                inspector_registry: &self.inspector_registry,
                thumbnail_image_loader: &self.thumbnail_image_loader,
            };

            self.dock_state.ui(&mut ui_context, ui);
        });

        //
        // Handle any actions that were prompted by the UI
        //
        self.action_queue.process(
            &self.db_state.project_configuration,
            &mut self.db_state.editor_model,
            &mut self.asset_engine,
            &mut self.ui_state,
            &mut self.modal_action,
            ctx,
        );

        //
        // Draw egui debug ui if it's enabled
        //
        super::egui_debug_ui::show_egui_debug_ui(ctx, &mut self.ui_state.egui_debug_ui_state);

        //
        // Finish the frame.
        //
        profiling::finish_frame!();
    }
}

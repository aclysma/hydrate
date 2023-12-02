use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;
use crate::QueuedActions;
use hydrate_model::pipeline::AssetEngine;
use hydrate_model::{AssetId, HashSet};
use imgui::{im_str, PopupModal};

pub struct ConfirmQuitWithoutSavingModal {
    finished_first_draw: bool,
    unsaved_assets: HashSet<AssetId>,
}

impl ConfirmQuitWithoutSavingModal {
    pub fn new(unsaved_assets: HashSet<AssetId>) -> Self {
        ConfirmQuitWithoutSavingModal {
            finished_first_draw: false,
            unsaved_assets,
        }
    }
}

impl ModalAction for ConfirmQuitWithoutSavingModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        _imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        _ui_state: &mut UiState,
        _asset_engine: &mut AssetEngine,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("ConfirmSaveQuit"));
        }

        let result = PopupModal::new(imgui::im_str!("ConfirmSaveQuit")).build(ui, || {
            ui.text("Are you sure you want to quit? Unsaved changes will be lost.");

            imgui::ListBox::new(im_str!("##unsaved_assets")).build(ui, || {
                for asset_id in &self.unsaved_assets {
                    ui.text(im_str!(
                        "{}",
                        db_state
                            .editor_model
                            .asset_display_name_long(*asset_id, &db_state.asset_path_cache)
                    ));
                }
            });

            if ui.button(imgui::im_str!("Save Changes")) {
                ui.close_current_popup();
                action_queue.queue_action(QueuedActions::SaveAll);
                action_queue.queue_action(QueuedActions::QuitNoConfirm);

                return ModalActionControlFlow::End;
            }

            ui.same_line();

            if ui.button(imgui::im_str!("Discard Changes")) {
                ui.close_current_popup();
                action_queue.queue_action(QueuedActions::QuitNoConfirm);

                return ModalActionControlFlow::End;
            }

            ui.same_line();

            if ui.button(imgui::im_str!("Cancel")) {
                ui.close_current_popup();

                return ModalActionControlFlow::End;
            }

            ModalActionControlFlow::Continue
        });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

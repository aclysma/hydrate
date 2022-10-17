use imgui::PopupModal;
use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::QueuedActions;
use crate::ui_state::UiState;

#[derive(Default)]
pub struct ConfirmQuitWithoutSavingModal {
    finished_first_draw: bool
}

impl ModalAction for ConfirmQuitWithoutSavingModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("ConfirmSaveQuit"));
        }

        let result = PopupModal::new(imgui::im_str!("ConfirmSaveQuit")).build(ui, || {
            ui.text("Are you sure you want to quit? Unsaved changes will be lost.");

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

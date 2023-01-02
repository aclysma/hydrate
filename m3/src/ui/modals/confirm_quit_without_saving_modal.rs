use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;
use crate::QueuedActions;
use imgui::{im_str, PopupModal};
use nexdb::edit_context::EditContext;
use nexdb::{EditorModel, HashSet, ObjectId, ObjectLocation};
use crate::importers::ImporterRegistry;

pub struct ConfirmQuitWithoutSavingModal {
    finished_first_draw: bool,
    unsaved_objects: HashSet<ObjectId>,
    unsaved_locations: HashSet<ObjectLocation>,
}

impl ConfirmQuitWithoutSavingModal {
    pub fn new(model: &EditorModel) -> Self {
        ConfirmQuitWithoutSavingModal {
            finished_first_draw: false,
            unsaved_objects: model.root_edit_context().modified_objects().clone(),
            unsaved_locations: model.root_edit_context().modified_locations().clone(),
        }
    }
}

impl ModalAction for ConfirmQuitWithoutSavingModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        importer_registry: &ImporterRegistry,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("ConfirmSaveQuit"));
        }

        let result = PopupModal::new(imgui::im_str!("ConfirmSaveQuit")).build(ui, || {
            ui.text("Are you sure you want to quit? Unsaved changes will be lost.");

            imgui::ListBox::new(im_str!("##unsaved_objects")).build(ui, || {
                for object_id in &self.unsaved_objects {
                    ui.text(im_str!(
                        "{}",
                        db_state.editor_model.object_display_name_long(*object_id)
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

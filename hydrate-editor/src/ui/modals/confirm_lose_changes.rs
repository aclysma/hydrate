use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use crate::ui_state::EditorModelUiState;
use crate::DbState;
use hydrate_model::pipeline::AssetEngine;

// For revert all or quitting without saving
fn confirm_lose_changes<F: Fn(&mut egui::Ui, &mut ModalActionControlFlow) -> ()>(
    context: ModalContext,
    bottom_ui: F,
) -> ModalActionControlFlow {
    let mut control_flow = ModalActionControlFlow::Continue;
    default_modal_window("Save or Discard Changes?", context, |context, ui| {
        ui.label(format!("Changes to the following {} assets will be lost:", context.db_state.editor_model.root_edit_context().modified_assets().len()));
        ui.separator();
        let mut table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .auto_shrink([false, false])
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .min_scrolled_height(300.0)
            .max_scroll_height(300.0)
            .column(egui_extras::Column::remainder().clip(true));

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Asset Path");
                });
            })
            .body(|mut body| {
                let modified_assets = context
                    .db_state
                    .editor_model
                    .root_edit_context()
                    .modified_assets();
                for asset_id in modified_assets {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            let long_name = context
                                .db_state
                                .editor_model
                                .asset_path(*asset_id, &context.ui_state.asset_path_cache);
                            ui.label(long_name.as_str());
                        });
                    });
                }
            });
        ui.separator();

        (bottom_ui)(ui, &mut control_flow);
    });

    control_flow
}

// For revert all or quitting without saving
#[derive(Default)]
pub struct ConfirmRevertChanges;

impl ModalAction for ConfirmRevertChanges {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let action_queue = context.action_queue;
        confirm_lose_changes(context, |ui, control_flow| {
            ui.horizontal(|ui| {
                if ui.button("Revert all Changes").clicked() {
                    action_queue.queue_action(UIAction::RevertAllNoConfirm);
                    *control_flow = ModalActionControlFlow::End;
                }

                if ui.button("Cancel").clicked() {
                    *control_flow = ModalActionControlFlow::End;
                }
            });
        })
    }
}

// For revert all or quitting without saving
#[derive(Default)]
pub struct ConfirmQuitWithoutSaving;

impl ModalAction for ConfirmQuitWithoutSaving {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let action_queue = context.action_queue;
        confirm_lose_changes(context, |ui, control_flow| {
            ui.horizontal(|ui| {
                if ui.button("Save and Quit").clicked() {
                    action_queue.queue_action(UIAction::SaveAll);
                    action_queue.queue_action(UIAction::QuitNoConfirm);

                    *control_flow = ModalActionControlFlow::End;
                }

                if ui.button("Discard and Quit").clicked() {
                    action_queue.queue_action(UIAction::QuitNoConfirm);

                    *control_flow = ModalActionControlFlow::End;
                }

                if ui.button("Cancel").clicked() {
                    *control_flow = ModalActionControlFlow::End;
                }
            });
        })
    }
}

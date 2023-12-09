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
        ui.label(format!(
            "Changes to the following {} assets will be lost:",
            context
                .db_state
                .editor_model
                .root_edit_context()
                .modified_assets()
                .len()
        ));
        ui.separator();
        egui::ScrollArea::both()
            .max_width(f32::INFINITY)
            .max_height(300.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .auto_shrink([false, false])
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::exact(10.0).clip(true))
                    .column(
                        egui_extras::Column::initial(100.0)
                            .at_least(10.0)
                            .clip(true),
                    )
                    .column(
                        egui_extras::Column::initial(100.0)
                            .at_least(10.0)
                            .clip(true),
                    )
                    .column(
                        egui_extras::Column::initial(200.0)
                            .at_least(10.0)
                            .clip(true),
                    );

                let mut all_modified_assets: Vec<_> = context
                    .db_state
                    .editor_model
                    .root_edit_context()
                    .assets()
                    .iter()
                    .filter(|(asset_id, info)| {
                        context
                            .db_state
                            .editor_model
                            .root_edit_context()
                            .modified_assets()
                            .contains(asset_id)
                    })
                    .collect();

                all_modified_assets
                    .sort_by(|(_, lhs), (_, rhs)| lhs.asset_name().cmp(&rhs.asset_name()));

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Type");
                        });
                        header.col(|ui| {
                            ui.strong("Path");
                        });
                    })
                    .body(|mut body| {
                        for (&asset_id, asset_info) in all_modified_assets {
                            body.row(20.0, |mut row| {
                                let short_name = context
                                    .db_state
                                    .editor_model
                                    .root_edit_context()
                                    .asset_name_or_id_string(asset_id)
                                    .unwrap();
                                let long_name = context
                                    .db_state
                                    .editor_model
                                    .asset_path(asset_id, &context.ui_state.asset_path_cache);

                                row.col(|ui| {
                                    //TODO
                                    ui.label("M");
                                });
                                row.col(|ui| {
                                    ui.strong(short_name);
                                });
                                row.col(|ui| {
                                    let schema_display_name = asset_info
                                        .schema()
                                        .markup()
                                        .display_name
                                        .as_deref()
                                        .unwrap_or(asset_info.schema().name());
                                    ui.label(schema_display_name);
                                });
                                row.col(|ui| {
                                    ui.label(long_name.as_str());
                                });
                            });
                        }
                    });
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

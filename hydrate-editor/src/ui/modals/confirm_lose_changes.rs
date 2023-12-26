use std::path::PathBuf;
use hydrate_base::AssetId;
use hydrate_model::DataSetAssetInfo;
use hydrate_model::edit_context::EditContext;
use crate::action_queue::UIAction;
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};

#[derive(Copy, Clone)]
enum OperationKind {
    Create,
    Delete,
    Modify,
}

struct PendingOperationInfo<'a> {
    kind: OperationKind,
    path: &'a PathBuf,
    asset_id: AssetId,
    asset_info: &'a DataSetAssetInfo,

}

// For revert all or quitting without saving
fn confirm_lose_changes<F: Fn(&mut egui::Ui, &mut ModalActionControlFlow) -> ()>(
    context: ModalContext,
    bottom_ui: F,
) -> ModalActionControlFlow {
    let mut control_flow = ModalActionControlFlow::Continue;
    default_modal_window("Save or Discard Changes?", context, |context, ui| {
        let pending = &context.ui_state.pending_file_operations;

        fn add_pending_operation_info<'a>(
            pending_operation_info: &mut Vec<PendingOperationInfo<'a>>,
            operations: &'a Vec<(AssetId, PathBuf)>,
            edit_context: &'a EditContext,
            kind: OperationKind
        ) {
            for (asset_id, path) in operations {
                pending_operation_info.push(PendingOperationInfo {
                    kind,
                    asset_id: *asset_id,
                    asset_info: edit_context.assets().get(&asset_id).unwrap(),
                    path: &path
                });
            }
        }

        let mut all_modified_assets = Vec::default();
        let edit_context = context.db_state.editor_model.root_edit_context();
        add_pending_operation_info(&mut all_modified_assets, &pending.create_operations, edit_context, OperationKind::Create);
        add_pending_operation_info(&mut all_modified_assets, &pending.modify_operations, edit_context, OperationKind::Modify);
        add_pending_operation_info(&mut all_modified_assets, &pending.delete_operations, edit_context, OperationKind::Delete);

        all_modified_assets
            .sort_by(|lhs, rhs| lhs.path.cmp(rhs.path));

        ui.label(format!(
            "Changes to the following {} assets will be lost:",
            all_modified_assets.len()
        ));
        ui.separator();
        egui::ScrollArea::both()
            .max_width(f32::INFINITY)
            .max_height(300.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let table = egui_extras::TableBuilder::new(ui)
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
                    )
                    .column(
                        egui_extras::Column::initial(200.0)
                            .at_least(10.0)
                            .clip(true),
                    );;

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
                        header.col(|ui| {
                            ui.strong("File");
                        });
                    })
                    .body(|mut body| {
                        for modified_asset in all_modified_assets {
                            body.row(20.0, |mut row| {
                                let short_name = context
                                    .db_state
                                    .editor_model
                                    .root_edit_context()
                                    .asset_name_or_id_string(modified_asset.asset_id)
                                    .unwrap();
                                let long_name = context
                                    .db_state
                                    .editor_model
                                    .asset_display_name_long(modified_asset.asset_id, &context.ui_state.asset_path_cache);

                                row.col(|ui| {
                                    let text = match modified_asset.kind {
                                        OperationKind::Create => "C",
                                        OperationKind::Delete => "D",
                                        OperationKind::Modify => "M",
                                    };
                                    ui.label(text);
                                });
                                row.col(|ui| {
                                    ui.strong(short_name);
                                });
                                row.col(|ui| {
                                    let schema_display_name = modified_asset
                                        .asset_info
                                        .schema()
                                        .markup()
                                        .display_name
                                        .as_deref()
                                        .unwrap_or(modified_asset.asset_info.schema().name());
                                    ui.label(schema_display_name);
                                });
                                row.col(|ui| {
                                    ui.label(long_name.as_str());
                                });
                                row.col(|ui| {
                                    ui.label(modified_asset.path.to_string_lossy());
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

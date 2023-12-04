use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui_state::EditorModelUiState;
use egui::{Widget};
use hydrate_model::{AssetId, EditorModel, PropertyPath, Schema};
use crate::ui::modals::NewAssetModal;

use super::inspector_system::*;

#[derive(Default)]
pub struct InspectorUiState {}

pub fn draw_inspector(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    asset_id: Option<AssetId>,
    inspector_registry: &InspectorRegistry,
) {
    egui::ScrollArea::vertical()
        .max_width(f32::INFINITY)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Bail if nothing selected
            let Some(asset_id) = asset_id else {
                return;
            };

            // Bail if asset wasn't found in root edit context
            let edit_context = editor_model.root_edit_context();
            if !edit_context.has_asset(asset_id) {
                return;
            }

            //
            // Some basic info
            //
            ui.heading(format!(
                "{}",
                edit_context.asset_name_or_id_string(asset_id).unwrap()
            ));

            ui.label(format!(
                "{}",
                editor_model
                    .asset_display_name_long(asset_id, &editor_model_ui_state.asset_path_cache)
            ));

            ui.label(format!("{:?}", asset_id.as_uuid()));

            //
            // Import info (TODO: Make this a mouseover/icon?)
            //
            let import_info = edit_context.import_info(asset_id);
            if let Some(import_info) = import_info {
                ui.collapsing("Import Info", |ui| {
                    ui.label(format!(
                        "Imported From: {}",
                        import_info.source_file_path().to_string_lossy()
                    ));
                    ui.label(format!(
                        "Importable Name: {:?}",
                        import_info.importable_name().name()
                    ));
                });
            }

            //
            // Prototype state
            //
            if let Some(prototype) = edit_context.asset_prototype(asset_id) {
                ui.horizontal(|ui| {
                    if ui.button(">>").clicked() {
                        action_sender.queue_action(UIAction::ShowAssetInAssetGallery(prototype));
                    }

                    let prototype_display_name =
                        editor_model.asset_display_name_long(prototype, &editor_model_ui_state.asset_path_cache);

                    ui.label(format!("Prototype: {}", prototype_display_name));
                });
            }

            //
            // Explain that generated assets are not editable (TODO: Make this prettier)
            //
            let is_generated = editor_model.is_generated_asset(asset_id);
            if is_generated {
                ui.label(format!("This asset is generated from a source file and can't be modified unless it is persisted to disk. A new asset file will be created and source file changes will no longer affect it."));
            }


            //
            // Some actions that can be taken (TODO: Make a context menu?)
            //
            if is_generated {
                if ui.button("Persist Asset").clicked() {
                    action_sender.queue_action(UIAction::PersistAssets(vec![asset_id]));
                }
            }

            if ui.button("Use as prototype").clicked() {
                action_sender.try_set_modal_action(NewAssetModal::new_with_prototype(Some(editor_model.root_edit_context().asset_location(asset_id).unwrap()), asset_id))
            }

            if ui.button("Rebuild this Asset").clicked() {
                //app_state.asset_engine.queue_build_operation(asset_id);
                action_sender.queue_action(UIAction::ForceRebuild(vec![asset_id]));
            }

            ui.separator();


            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let read_only = is_generated;

                    let available_x = ui.available_width();
                    let mut table = egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .auto_shrink([true, false])
                        .resizable(true)
                        // vscroll and min/max scroll height make this table grow/shrink according to available size
                        .vscroll(false)
                        .min_scrolled_height(1.0)
                        .max_scroll_height(1.0)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(egui_extras::Column::initial(200.0).at_least(5.0).at_most(available_x * 0.9).clip(true))
                        .column(egui_extras::Column::remainder().at_least(5.0).at_most(available_x * 0.9).clip(true));

                    table.body(|mut body| {
                        super::inspector_system::draw_inspector_rows(
                            &mut body,
                            InspectorContext {
                                editor_model,
                                editor_model_ui_state,
                                action_sender,
                                asset_id,
                                property_name: "",
                                property_path: &PropertyPath::default(),
                                schema: &Schema::Record(
                                    editor_model.root_edit_context().data_set().asset_schema(asset_id).unwrap().fingerprint()
                                ),
                                inspector_registry,
                                read_only,
                            },
                            0
                        );
                    });
                });
        });
}

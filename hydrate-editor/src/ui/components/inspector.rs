use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::image_loader::ThumbnailImageLoader;
use crate::ui::modals::{MoveAssetsModal, NewAssetModal};
use crate::ui_state::EditorModelUiState;
use hydrate_model::{
    AssetId, EditorModel, HashSet, PropertyPath, Schema, SchemaDefRecordFieldMarkup,
};
use std::sync::Arc;

use super::inspector_system::*;

#[derive(Default)]
pub struct InspectorUiState {
    pinned_selection: Option<(AssetId, Arc<HashSet<AssetId>>)>,
}

pub fn draw_inspector(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    inspector_ui_state: &mut InspectorUiState,
    selected_assets_unpinned: &HashSet<AssetId>,
    primary_asset_id_unpinned: Option<AssetId>,
    inspector_registry: &InspectorRegistry,
    thumbnail_image_loader: &ThumbnailImageLoader,
) {
    egui::ScrollArea::vertical()
        .max_width(f32::INFINITY)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            //
            // This bit of code just lets us pick between using a pinned selection vs. whatever is currently selected.
            // The main complication here is we want a reference to the contents of the Arc<HashSet<AssetId>> in the
            // inspector_ui_state but also be able to mutate inspector_ui_state
            //
            let selected_assets_arc;
            let (primary_asset_id, selected_assets) = if let Some((primary_asset_id, selected_assets)) = &inspector_ui_state.pinned_selection {
                selected_assets_arc = Some(selected_assets.clone());
                let selected_assets = &**selected_assets_arc.as_ref().unwrap();
                (*primary_asset_id, selected_assets)
            } else {
                if let Some(primary_asset_id) = primary_asset_id_unpinned {
                    (primary_asset_id, selected_assets_unpinned)
                } else {
                    // Bail if nothing selected
                    return;
                }
            };

            assert!(selected_assets.contains(&primary_asset_id));

            // Bail if asset wasn't found in root edit context
            let edit_context = editor_model.root_edit_context();
            if !edit_context.has_asset(primary_asset_id) {
                return;
            }

            let mut all_are_same_schema = true;
            let mut are_any_generated = false;
            let primary_asset_schema = edit_context.asset_schema(primary_asset_id).unwrap();
            for selected_asset in selected_assets {
                if let Some(selected_schema) = edit_context.asset_schema(*selected_asset) {
                    if selected_schema.fingerprint() != primary_asset_schema.fingerprint() {
                        all_are_same_schema = false;
                    }

                    if editor_model.is_generated_asset(*selected_asset) {
                        are_any_generated = true;
                    }


                }
            }

            //
            // Some basic info
            //
            //let is_generated = editor_model.is_generated_asset(asset_id);

            let thumbnail_stack_size = crate::ui::thumbnail_stack_size();
            let (_, header_rect) = ui.allocate_space(egui::vec2(ui.available_width(), thumbnail_stack_size.y));
            let mut header_left_rect = header_rect;
            header_left_rect.max.x = 5f32.max(header_left_rect.max.x - thumbnail_stack_size.x);
            let header_left_clip_rect = header_left_rect;
            header_left_rect.min.x = header_left_rect.max.x.min(header_left_rect.min.x + 5.0);

            let mut header_right_rect = header_rect;
            header_right_rect.min.x = header_left_rect.max.x;

            let mut header_left = ui.child_ui(header_left_rect, egui::Layout::right_to_left(egui::Align::Min));
            header_left.set_clip_rect(header_left_clip_rect);

            let mut header_right = ui.child_ui(header_right_rect, egui::Layout::right_to_left(egui::Align::Min));

            crate::ui::draw_thumbnail_stack(&mut header_right, editor_model, thumbnail_image_loader, primary_asset_id, selected_assets.iter().copied());

            header_left.vertical(|ui| {
                let header_text = if selected_assets.len() > 1 {
                    format!(
                        "{} selected assets",
                        selected_assets.len()
                    )
                } else {
                    format!(
                        "{}",
                        edit_context.asset_name_or_id_string(primary_asset_id).unwrap()
                    )
                };
                ui.add(egui::Label::new(egui::RichText::new(header_text).heading()).truncate(true));

                ui.horizontal(|ui| {
                    if ui.selectable_label(inspector_ui_state.pinned_selection.is_some(), "Pin Selection").clicked() {
                        if inspector_ui_state.pinned_selection.is_some() {
                            inspector_ui_state.pinned_selection = None;
                        } else {
                            inspector_ui_state.pinned_selection = Some((primary_asset_id, Arc::new(selected_assets.clone())));
                        }
                    }

                    ui.menu_button("Actions...", |ui| {
                        if are_any_generated {
                            ui.label("One or more assets are generated and cannot be edited directly");
                        }

                        //
                        // Some actions that can be taken (TODO: Make a context menu?)
                        //
                        if are_any_generated {
                            if ui.button("Persist Asset").clicked() {
                                action_sender.queue_action(UIAction::PersistAssets(selected_assets.iter().copied().collect()));
                                ui.close_menu();
                            }
                        }

                        let can_use_as_prototype = editor_model.root_edit_context().import_info(primary_asset_id).is_none() && selected_assets.len() == 1;

                        if ui.add_enabled(can_use_as_prototype, egui::Button::new("Use as prototype")).clicked() {
                            action_sender.try_set_modal_action(NewAssetModal::new_with_prototype(
                                Some(editor_model.root_edit_context().asset_location(primary_asset_id).unwrap()),
                                primary_asset_id)
                            );
                            ui.close_menu();
                        }

                        if ui.button("Reimport And Rebuild").clicked() {
                            action_sender.queue_action(UIAction::ReimportAndRebuild(selected_assets.iter().copied().collect()));
                            ui.close_menu();
                        }

                        if ui.button("Rebuild").clicked() {
                            action_sender.queue_action(UIAction::ForceRebuild(selected_assets.iter().copied().collect()));
                            ui.close_menu();
                        }

                        if ui.button("Duplicate").clicked() {
                            action_sender.queue_action(UIAction::DuplicateAssets(selected_assets.iter().copied().collect()));
                            ui.close_menu();
                        }

                        let can_rename = selected_assets.len() == 1;
                        let move_or_rename_text = if can_rename {
                            "Move or Rename"
                        } else {
                            "Move"
                        };

                        if ui.add_enabled(!are_any_generated, egui::Button::new(move_or_rename_text)).clicked() {
                            let location = edit_context.asset_location(primary_asset_id).unwrap();
                            if can_rename {
                                // Single selected case, can move or rename
                                let name = edit_context.asset_name(primary_asset_id).unwrap();
                                action_sender.try_set_modal_action(MoveAssetsModal::new_single_asset(
                                    primary_asset_id,
                                    name.as_string().cloned().unwrap_or_else(|| primary_asset_id.to_string()),
                                    Some(location))
                                );
                            } else {
                                // Multiple selected case, move only
                                action_sender.try_set_modal_action(MoveAssetsModal::new_multiple_assets(
                                    vec![primary_asset_id],
                                    Some(location))
                                );
                            }

                            ui.close_menu();
                        }

                        let delete_button_string = if selected_assets.len() > 1 {
                            format!("Delete {} assets", selected_assets.len())
                        } else {
                            format!("Delete {}", edit_context.asset_name_or_id_string(primary_asset_id).unwrap())
                        };

                        if ui.add_enabled(!are_any_generated, egui::Button::new(delete_button_string)).clicked() {
                            action_sender.queue_action(UIAction::DeleteAssets(selected_assets.iter().copied().collect()));
                            ui.close_menu();
                        }
                    });
                });
            });

            if selected_assets.len() == 1 {
                ui.collapsing("Details", |ui| {
                    ui.label(format!(
                        "{}",
                        editor_model
                            .asset_display_name_long(primary_asset_id, &editor_model_ui_state.asset_path_cache)
                    ));

                    ui.label(format!("{:?}", primary_asset_id.as_uuid()));
                });

                //
                // Import info (TODO: Make this a mouseover/icon?)
                //
                let import_info = edit_context.import_info(primary_asset_id);
                if let Some(import_info) = import_info {
                    let mut path_reference_overrides: Vec<_> = edit_context.resolve_all_path_reference_overrides(primary_asset_id).unwrap().into_iter().collect();
                    path_reference_overrides.sort_by(|lhs, rhs| lhs.0.path().cmp(rhs.0.path()));
                    ui.collapsing("Import Info", |ui| {
                        ui.label(format!(
                            "Imported From: {}",
                            import_info.source_file()
                        ));
                        ui.label(format!(
                            "Importable Name: {:?}",
                            import_info.importable_name().name()
                        ));

                        for (k, v) in path_reference_overrides {
                            ui.label(format!(
                                "name {} value {}", k, v
                            ));
                        }

                    });
                }

                //
                // Prototype state
                //
                if let Some(prototype) = edit_context.asset_prototype(primary_asset_id) {
                    ui.horizontal(|ui| {
                        if ui.button(">>").clicked() {
                            action_sender.queue_action(UIAction::ShowAssetInAssetGallery(prototype));
                        }

                        let prototype_display_name =
                            editor_model.asset_display_name_long(prototype, &editor_model_ui_state.asset_path_cache);

                        ui.label(format!("Prototype: {}", prototype_display_name));
                    });
                }
            }


            //
            // Explain that generated assets are not editable (TODO: Make this prettier)
            //
            if are_any_generated {
                ui.label(format!("One or more selected assets are generated from a source file and can't be modified unless it is persisted to disk. A new asset file will be created and source file changes will no longer affect it."));
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if all_are_same_schema {
                        let read_only = are_any_generated;

                        let available_x = ui.available_width();
                        let table = egui_extras::TableBuilder::new(ui)
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
                                    selected_assets: &selected_assets,
                                    primary_asset_id: primary_asset_id,
                                    property_default_display_name: "",
                                    property_path: &PropertyPath::default(),
                                    field_markup: &SchemaDefRecordFieldMarkup::default(),
                                    schema: &Schema::Record(
                                        editor_model.root_edit_context().data_set().asset_schema(primary_asset_id).unwrap().fingerprint()
                                    ),
                                    inspector_registry,
                                    thumbnail_image_loader,
                                    read_only,
                                },
                                0,
                            );
                        });
                    } else {
                        ui.label("Property inspector unavailable. The selection contains multiple asset types.");
                    }
                });
        });
}

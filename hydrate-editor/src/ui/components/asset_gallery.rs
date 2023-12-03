use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui::modals::{NewAssetModal, TestModal};
use crate::ui_state::{EditorModelUiState};
use egui::epaint::text::FontsImpl;
use egui::{Color32, FontDefinitions, FontId, Layout, SelectableLabel, Widget};
use hydrate_model::{AssetId, AssetLocation, AssetName, DataSetAssetInfo, EndContextBehavior, HashSet};
use std::sync::Arc;
use crate::DbState;
use crate::ui::components::{AssetTreeUiState, schema_record_selector};

#[derive(Default, PartialEq, Copy, Clone)]
pub enum AssetGalleryViewMode {
    #[default]
    Table,
    Grid,
}

#[derive(Default, PartialEq, Copy, Clone)]
pub enum AssetGalleryViewLocationFilteringMode {
    #[default]
    AllChildren,
    DirectChildOnly,
}

pub struct AssetGalleryUiState {
    search_string: String,
    pub selected_assets: HashSet<AssetId>,
    view_mode: AssetGalleryViewMode,
    location_filtering_mode: AssetGalleryViewLocationFilteringMode,
    tile_size: f32,
}

impl Default for AssetGalleryUiState {
    fn default() -> Self {
        AssetGalleryUiState {
            search_string: String::default(),
            selected_assets: Default::default(),
            view_mode: Default::default(),
            location_filtering_mode: Default::default(),
            tile_size: 150.0,
        }
    }
}

pub fn draw_asset_gallery(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_tree_ui_state: &AssetTreeUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
) {
    //ui.label("asset gallery");

    //println!("available {:?}", ui.available_width());
    let (toolbar_id, toolbar_rect) = ui.allocate_space(egui::vec2(ui.available_width(), 30.0));

    //ui.child_ui(toolbar_rect)

    let path_filter = asset_tree_ui_state.selected_tree_node;

    let mut toolbar_ui_left = ui.child_ui(
        toolbar_rect,
        egui::Layout::left_to_right(egui::Align::Center),
    );

    if toolbar_ui_left.selectable_label(asset_gallery_ui_state.view_mode == AssetGalleryViewMode::Grid, "Grid").clicked() {
        asset_gallery_ui_state.view_mode = AssetGalleryViewMode::Grid;
    }

    if toolbar_ui_left.selectable_label(asset_gallery_ui_state.view_mode == AssetGalleryViewMode::Table, "Table").clicked() {
        asset_gallery_ui_state.view_mode = AssetGalleryViewMode::Table;
    }

    toolbar_ui_left.add(egui::Separator::default().vertical());

    let mut show_all_children = asset_gallery_ui_state.location_filtering_mode == AssetGalleryViewLocationFilteringMode::AllChildren;
    toolbar_ui_left.checkbox(&mut show_all_children, "Show All Children");
    asset_gallery_ui_state.location_filtering_mode = if show_all_children {
        AssetGalleryViewLocationFilteringMode::AllChildren
    } else {
        AssetGalleryViewLocationFilteringMode::DirectChildOnly
    };

    toolbar_ui_left.add(egui::Separator::default().vertical());

    toolbar_ui_left.label("Search:");
    egui::TextEdit::singleline(&mut asset_gallery_ui_state.search_string)
        .desired_width(250.0)
        .show(&mut toolbar_ui_left);

    // let mut selected = "First";
    // egui::ComboBox::from_label("Select one!")
    //     .selected_text(format!("{:?}", selected))
    //     .show_ui(&mut child_ui, |ui| {
    //         ui.selectable_value(&mut selected, "First", "First");
    //         ui.selectable_value(&mut selected, "Second", "Second");
    //         ui.selectable_value(&mut selected, "Third", "Third");
    //     });

    if toolbar_ui_left.available_width() > 200.0 {
        let mut toolbar_ui_right = ui.child_ui(
            toolbar_rect,
            egui::Layout::right_to_left(egui::Align::Center),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::TOP), |ui| {
            if toolbar_ui_right.button("New Asset").clicked() {
                action_queue.try_set_modal_action(NewAssetModal::new(asset_tree_ui_state.selected_tree_node));
            }

            toolbar_ui_right.add_visible(asset_gallery_ui_state.view_mode == AssetGalleryViewMode::Grid,
            egui::Slider::new(&mut asset_gallery_ui_state.tile_size, 50.0..=150.0)
                .clamp_to_range(true)
                .show_value(false));
        });
    }

    ui.separator();

    let mut all_assets: Vec<_> = db_state
        .editor_model
        .root_edit_context().assets().iter().filter(|(&asset_id, info)| {
        if db_state.editor_model.is_path_node_or_root(info.schema().fingerprint()) {
            return false;
        }

        if !asset_gallery_ui_state.search_string.is_empty() {
            let long_name = db_state.editor_model.asset_path(asset_id, &ui_state.asset_path_cache);
            if !long_name.as_str().to_lowercase().contains(&asset_gallery_ui_state.search_string.to_lowercase()) {
                return false;
            }
        }

        if let Some(path_filter) = path_filter {
            if show_all_children {
                // Is child or indirect child of the selected directory
                if !db_state.editor_model.root_edit_context().data_set().asset_location_chain(asset_id).unwrap().contains(&path_filter) {
                    return false;
                }
            } else {
                // Exactly matches
                if info.asset_location() != path_filter {
                    return false;
                }
            }
        }

        true
    }).collect();

    all_assets.sort_by(|(_, lhs), (_, rhs)| lhs.asset_name().cmp(&rhs.asset_name()));

    let view_mode = asset_gallery_ui_state.view_mode;
    match view_mode {
        AssetGalleryViewMode::Table => {
            egui::ScrollArea::both()
                .max_width(f32::INFINITY)
                .max_height(f32::INFINITY)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_asset_gallery_list(
                        ui,
                        fonts_impl,
                        font_id,
                        db_state,
                        ui_state,
                        asset_gallery_ui_state,
                        action_queue,
                        &all_assets
                    );
               });
        }
        AssetGalleryViewMode::Grid => {
            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .max_height(f32::INFINITY)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_asset_gallery_tile_grid(
                        ui,
                        fonts_impl,
                        font_id,
                        db_state,
                        ui_state,
                        asset_gallery_ui_state,
                        action_queue,
                        &all_assets
                    );
                });
        }
    }
}

fn draw_asset_gallery_list(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
    all_assets: &Vec<(&AssetId, &DataSetAssetInfo)>
) {
    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 2.0);

    let mut table = egui_extras::TableBuilder::new(ui)
        .striped(true)
        .auto_shrink([true, false])
        .resizable(true)
        // vscroll and min/max scroll height make this table grow/shrink according to available size
        .vscroll(false)
        .min_scrolled_height(1.0)
        .max_scroll_height(1.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::initial(200.0).at_least(40.0).clip(true))
        .column(egui_extras::Column::initial(100.0).at_least(40.0).clip(true))
        .column(egui_extras::Column::remainder());

    table
        .header(20.0, |mut header| {
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
            for (&asset_id, asset_info) in all_assets {
                body.row(20.0, |mut row| {
                    let short_name = db_state.editor_model.root_edit_context().asset_name_or_id_string(asset_id).unwrap();
                    let long_name = db_state.editor_model.asset_path(asset_id, &ui_state.asset_path_cache);
                    let is_generated = db_state.editor_model.is_generated_asset(asset_id);

                    row.col(|ui| {
                        crate::ui::drag_drop::drag_source(
                            ui,
                            egui::Id::new(asset_id),
                            DragDropPayload::AssetReference(asset_id),
                            |ui| {
                            let is_selected = asset_gallery_ui_state.selected_assets.contains(&asset_id);
                            let response = egui::SelectableLabel::new(is_selected, &short_name).ui(ui);
                            if response.clicked() {
                                asset_gallery_ui_state.selected_assets.clear();
                                asset_gallery_ui_state.selected_assets.insert(asset_id);
                            }
                            response.context_menu(|ui| {
                                asset_context_menu(ui, action_queue, asset_id, is_generated, &short_name, asset_info.asset_location());
                            })
                        });
                    });
                    row.col(|ui| {
                        ui.label(asset_info.schema().name());
                    });
                    row.col(|ui| {
                        ui.label(long_name.as_str());
                    });

                });
            }
        });


}

fn draw_asset_gallery_tile_grid(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
    all_assets: &Vec<(&AssetId, &DataSetAssetInfo)>
) {
    ui.with_layout(
        Layout::left_to_right(egui::Align::TOP).with_main_wrap(true),
        |ui| {
            // for (_, asset_info) in &ui_state.all_asset_info {
            //     draw_asset_gallery_tile(
            //         ui,
            //         fonts_impl,
            //         font_id,
            //         asset_gallery_ui_state,
            //         asset_info,
            //         action_queue,
            //         all_assets,
            //     );
            // }
            for (asset_id, asset_info) in all_assets {
                draw_asset_gallery_tile(
                    ui,
                    fonts_impl,
                    font_id,
                    db_state,
                    ui_state,
                    asset_gallery_ui_state,
                    **asset_id,
                    *asset_info,
                    action_queue,
                );
            }
        },
    );
}

fn draw_asset_gallery_tile(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    asset_id: AssetId,
    asset_info: &DataSetAssetInfo,
    action_queue: &UIActionQueueSender,
) {
    let short_name = db_state.editor_model.root_edit_context().asset_name_or_id_string(asset_id).unwrap();
    let long_name = db_state.editor_model.asset_path(asset_id, &ui_state.asset_path_cache);
    let is_generated = db_state.editor_model.is_generated_asset(asset_id);
    let is_dirty = db_state.editor_model.root_edit_context().modified_assets().contains(&asset_id);

    crate::ui::drag_drop::drag_source(
        ui,
        egui::Id::new(asset_id),
        DragDropPayload::AssetReference(asset_id),
        |ui| {
            let mut is_on = false;

            let desired_size = egui::vec2(asset_gallery_ui_state.tile_size, asset_gallery_ui_state.tile_size + 30.0);
            let thumbnail_size = egui::vec2(asset_gallery_ui_state.tile_size, asset_gallery_ui_state.tile_size);

            let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
            // ui.allocate_ui(desired_size, |ui| {
            //     ui.painter().rect_stroke(thumbnail_size, 3.0, egui::Stroke::new(2.0, egui::Color32::from_gray(50)));
            //     ui.label("hi");
            // });

            let mut thumbnail_rect = rect;
            thumbnail_rect.max.y = thumbnail_rect
                .max
                .y
                .min(thumbnail_rect.min.y + thumbnail_size.y);
            let mut text_rect = rect;
            text_rect.min.y = thumbnail_rect.max.y;

            if response.clicked() {
                asset_gallery_ui_state.selected_assets.clear();
                asset_gallery_ui_state.selected_assets.insert(asset_id);
            }

            if ui.is_visible() {
                let how_on = ui.ctx().animate_bool(response.id, is_on);
                let visuals = ui.style().interact_selectable(&response, is_on);
                let radius = 3.0;

                if asset_gallery_ui_state
                    .selected_assets
                    .contains(&asset_id)
                {
                    ui.painter()
                        .rect_filled(rect, radius, ui.style().visuals.selection.bg_fill);
                }
                ui.painter().rect_stroke(
                    thumbnail_rect,
                    radius,
                    egui::Stroke::new(2.0, egui::Color32::from_gray(50)),
                );

                let anchor =
                    egui::Pos2::new((text_rect.min.x + text_rect.max.x) / 2.0, text_rect.min.y);

                let text_color = if is_dirty {
                    egui::Color32::from_rgb(255, 255, 0)
                } else if is_generated {
                    egui::Color32::from_rgb(150, 150, 150)
                } else {
                    egui::Color32::from_rgb(230, 230, 230)
                };

                let mut layout_job = egui::epaint::text::LayoutJob::single_section(
                    short_name.clone(),
                    egui::epaint::text::TextFormat::simple(font_id.clone(), text_color),
                );
                layout_job.wrap.max_width = text_rect.max.x - text_rect.min.x;
                layout_job.wrap.max_rows = 1;
                layout_job.wrap.break_anywhere = false;
                let galley = egui::epaint::text::layout(fonts_impl, Arc::new(layout_job));
                let text = galley.rows[0].text();

                ui.painter().text(
                    anchor,
                    egui::Align2::CENTER_TOP,
                    text,
                    font_id.clone(),
                    text_color,
                );
            } else {
                println!("not visible");
            }

            let is_generated = is_generated;
            let response = response.context_menu(move |ui| {
                asset_context_menu(ui, action_queue, asset_id, is_generated, &short_name, asset_info.asset_location());
            });

            response
        },
    );
}

fn asset_context_menu(ui: &mut egui::Ui, action_queue: &UIActionQueueSender, asset_id: AssetId, is_generated: bool, name: &str, location: AssetLocation) {
    if is_generated {
        ui.label("This asset is generated and cannot be edited directly");
    }
    if ui
        .add_enabled(
            !is_generated,
            egui::Button::new(format!("Delete {}", name)),
        )
        .clicked()
    {
        action_queue.queue_edit("delete asset", vec![asset_id], move |edit_context| {
            edit_context.delete_asset(asset_id).unwrap();
            Ok(EndContextBehavior::Finish)
        });
        ui.close_menu();
    }

    if ui.button("Use as prototype for new asset").clicked() {

        action_queue.try_set_modal_action(NewAssetModal::new_with_prototype(Some(location), asset_id));

        // action_queue.queue_edit("delete asset", vec![asset_id], move |edit_context| {
        //     let old_location = edit_context.asset_location(asset_id).unwrap().clone();
        //     let old_name = edit_context.asset_name(asset_id).unwrap().clone();
        //     let new_name = format!("New from {:?}", asset_id);
        //
        //
        //
        //     edit_context
        //         .new_asset_from_prototype(
        //             &AssetName::new(new_name),
        //             &old_location,
        //             asset_id,
        //         )
        //         .unwrap();
        //     Ok(EndContextBehavior::Finish)
        // });
        ui.close_menu();
    }
}
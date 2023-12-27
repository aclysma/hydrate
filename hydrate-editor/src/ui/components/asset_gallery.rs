use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::components::AssetTreeUiState;
use crate::ui::drag_drop::DragDropPayload;
use crate::ui::modals::{MoveAssetsModal, NewAssetModal};
use crate::ui_state::EditorModelUiState;
use crate::DbState;
use egui::epaint::text::FontsImpl;
use egui::{Color32, FontId, Layout, Ui, Widget};
use hydrate_model::{AssetId, AssetLocation, DataSetAssetInfo, EditorModel, HashSet};
use std::sync::Arc;
use egui::text::LayoutJob;
use hydrate_model::edit_context::EditContext;
use hydrate_model::pipeline::ThumbnailProviderRegistry;
use crate::image_loader::ThumbnailImageLoader;

#[derive(Default, PartialEq, Copy, Clone)]
pub enum AssetGalleryViewMode {
    Table,
    #[default]
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
    all_selectable_assets: HashSet<AssetId>,
    selected_assets: HashSet<AssetId>,
    primary_selected_asset: Option<AssetId>,
    previous_shift_select_range_begin: Option<AssetId>,
    previous_shift_select_range_end: Option<AssetId>,
    view_mode: AssetGalleryViewMode,
    location_filtering_mode: AssetGalleryViewLocationFilteringMode,
    tile_size: f32,
}

impl Default for AssetGalleryUiState {
    fn default() -> Self {
        AssetGalleryUiState {
            search_string: String::default(),
            all_selectable_assets: HashSet::default(),
            selected_assets: Default::default(),
            primary_selected_asset: None,
            previous_shift_select_range_begin: None,
            previous_shift_select_range_end: None,
            view_mode: Default::default(),
            location_filtering_mode: Default::default(),
            tile_size: 128.0,
        }
    }
}

impl AssetGalleryUiState {
    // All the assets that are not being filtered out.
    pub fn all_selectable_assets(&self) -> &HashSet<AssetId> {
        &self.all_selectable_assets
    }

    pub fn selected_assets(&self) -> &HashSet<AssetId> {
        &self.selected_assets
    }

    pub fn primary_selected_asset(&self) -> Option<AssetId> {
        self.primary_selected_asset
    }

    pub fn set_selection(&mut self, asset_id: Option<AssetId>) {
        self.selected_assets.clear();
        self.primary_selected_asset = None;
        self.previous_shift_select_range_begin = None;
        self.previous_shift_select_range_end = None;

        if let Some(asset_id) = asset_id {
            self.selected_assets.insert(asset_id);
            self.primary_selected_asset = Some(asset_id);
            self.previous_shift_select_range_begin = Some(asset_id);
        }
    }

    pub fn all_selected(&mut self) -> bool {
        if self.all_selectable_assets.len() != self.selected_assets.len() {
            return false;
        }

        for asset_id in &self.all_selectable_assets {
            if !self.selected_assets.contains(asset_id) {
                return false;
            }
        }

        true
    }

    pub fn toggle_select_all(&mut self) {
        if !self.all_selected() {
            self.select_all();
        } else {
            self.select_none();
        }
    }

    pub fn select_one(&mut self, asset_id: AssetId) {
        // Normal clicks clear current selection and then select the clicked asset
        self.select_none();

        self.selected_assets.insert(asset_id);
        self.primary_selected_asset = Some(asset_id);
        self.previous_shift_select_range_begin = Some(asset_id);
        self.previous_shift_select_range_end = None;
    }

    pub fn select_all(&mut self) {
        // If we had something as our primary selection and it's not selectable, and we select all, stop selecting it
        if let Some(primary_selected_asset) = self.primary_selected_asset {
            if !self.all_selectable_assets.contains(&primary_selected_asset) {
                self.primary_selected_asset = None;
            }
        }

        self.selected_assets = self.all_selectable_assets.clone();
        if self.primary_selected_asset.is_none() {
            for &asset_id in &self.selected_assets {
                self.primary_selected_asset = Some(asset_id);
                break;
            }
        }

        self.previous_shift_select_range_begin = None;
        self.previous_shift_select_range_end = None;
    }

    pub fn select_none(&mut self) {
        self.primary_selected_asset = None;
        self.selected_assets.clear();
        self.previous_shift_select_range_begin = None;
        self.previous_shift_select_range_end = None;
    }
}

pub fn draw_asset_gallery(
    ui: &mut egui::Ui,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_tree_ui_state: &AssetTreeUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
    thumbnail_image_loader: &ThumbnailImageLoader,
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
    toolbar_ui_left.set_clip_rect(toolbar_rect);

    toolbar_ui_left.allocate_space(egui::vec2(0.0, 0.0));
    if toolbar_ui_left.button("New Asset").clicked() {
        action_queue.try_set_modal_action(NewAssetModal::new(
            asset_tree_ui_state.selected_tree_node,
        ));
    }

    toolbar_ui_left.separator();

    if toolbar_ui_left
        .selectable_label(
            asset_gallery_ui_state.view_mode == AssetGalleryViewMode::Grid,
            "Grid",
        )
        .clicked()
    {
        asset_gallery_ui_state.view_mode = AssetGalleryViewMode::Grid;
    }

    if toolbar_ui_left
        .selectable_label(
            asset_gallery_ui_state.view_mode == AssetGalleryViewMode::Table,
            "Table",
        )
        .clicked()
    {
        asset_gallery_ui_state.view_mode = AssetGalleryViewMode::Table;
    }

    toolbar_ui_left.add(egui::Separator::default().vertical());

    let mut show_all_children = asset_gallery_ui_state.location_filtering_mode
        == AssetGalleryViewLocationFilteringMode::AllChildren;
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

    toolbar_ui_left.add_visible(
        asset_gallery_ui_state.view_mode == AssetGalleryViewMode::Grid,
        egui::Slider::new(&mut asset_gallery_ui_state.tile_size, 50.0..=150.0)
            .clamp_to_range(true)
            .show_value(false),
    );

    ui.separator();

    let mut all_assets: Vec<_> = db_state
        .editor_model
        .root_edit_context()
        .assets()
        .iter()
        .filter(|(&asset_id, info)| {
            if db_state
                .editor_model
                .is_path_node_or_root(info.schema().fingerprint())
            {
                return false;
            }

            if !asset_gallery_ui_state.search_string.is_empty() {
                let Some(long_name) = db_state
                    .editor_model
                    .asset_path(asset_id, &ui_state.asset_path_cache) else {
                    return false;
                };

                if !long_name
                    .as_str()
                    .to_lowercase()
                    .contains(&asset_gallery_ui_state.search_string.to_lowercase())
                {
                    return false;
                }
            }

            if let Some(path_filter) = path_filter {
                if show_all_children {
                    // Is child or indirect child of the selected directory
                    if !db_state
                        .editor_model
                        .root_edit_context()
                        .data_set()
                        .asset_location_chain(asset_id)
                        .unwrap()
                        .contains(&path_filter)
                    {
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
        })
        .collect();

    all_assets.sort_by(|(_, lhs), (_, rhs)| lhs.asset_name().cmp(&rhs.asset_name()));

    asset_gallery_ui_state.all_selectable_assets.clear();
    for (&asset_id, _) in &all_assets {
        asset_gallery_ui_state.all_selectable_assets.insert(asset_id);
    }

    let (_, mut next_rect) = ui.allocate_space(ui.available_size());
    next_rect.min.x += 8.0;
    next_rect.max.x -= 8.0;
    let mut child_ui = ui.child_ui(next_rect, egui::Layout::top_down(egui::Align::Center));

    let view_mode = asset_gallery_ui_state.view_mode;
    match view_mode {
        AssetGalleryViewMode::Table => {
            egui::ScrollArea::both()
                .max_width(child_ui.available_width())
                .max_height(f32::INFINITY)
                .auto_shrink([false, false])
                .show(&mut child_ui, |ui| {
                    draw_asset_gallery_list(
                        ui,
                        db_state,
                        ui_state,
                        asset_gallery_ui_state,
                        action_queue,
                        &all_assets,
                        thumbnail_image_loader
                    );
                });
        }
        AssetGalleryViewMode::Grid => {
            egui::ScrollArea::vertical()
                .max_width(child_ui.available_width())
                .max_height(f32::INFINITY)
                .auto_shrink([false, false])
                .show(&mut child_ui, |ui| {
                    draw_asset_gallery_tile_grid(
                        ui,
                        db_state,
                        ui_state,
                        asset_gallery_ui_state,
                        action_queue,
                        &all_assets,
                        thumbnail_image_loader,
                    );
                });
        }
    }
}

fn draw_asset_gallery_list(
    ui: &mut egui::Ui,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
    all_assets: &Vec<(&AssetId, &DataSetAssetInfo)>,
    thumbnail_image_loader: &ThumbnailImageLoader,
) {
    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 2.0);

    let table = egui_extras::TableBuilder::new(ui)
        .striped(true)
        .auto_shrink([true, false])
        .resizable(true)
        // vscroll and min/max scroll height make this table grow/shrink according to available size
        .vscroll(false)
        .min_scrolled_height(1.0)
        .max_scroll_height(1.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(
            egui_extras::Column::initial(200.0)
                .at_least(40.0)
                .clip(true),
        )
        .column(
            egui_extras::Column::initial(100.0)
                .at_least(40.0)
                .clip(true),
        )
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
                    let short_name = db_state
                        .editor_model
                        .root_edit_context()
                        .asset_name_or_id_string(asset_id)
                        .unwrap();
                    let long_name = db_state
                        .editor_model
                        .asset_path(asset_id, &ui_state.asset_path_cache);
                    let is_generated = db_state.editor_model.is_generated_asset(asset_id);
                    let is_dirty = ui_state.edited_objects.contains(&asset_id);

                    row.col(|ui| {
                        crate::ui::drag_drop::drag_source(
                            ui,
                            egui::Id::new(asset_id),
                            &db_state.editor_model,
                            ui_state,
                            thumbnail_image_loader,
                            asset_gallery_ui_state,
                            |asset_gallery_ui_state| create_drag_payload(asset_gallery_ui_state, asset_id),
                            |ui, asset_gallery_ui_state| {
                                let is_selected =
                                    asset_gallery_ui_state.selected_assets.contains(&asset_id);

                                let text_color = if is_dirty {
                                    egui::Color32::from_rgb(255, 255, 100)
                                } else if is_generated {
                                    egui::Color32::from_rgb(97, 150, 199)
                                } else {
                                    egui::Color32::from_rgb(230, 230, 230)
                                };
                                ui.style_mut().visuals.override_text_color = Some(text_color);

                                let response =
                                    egui::SelectableLabel::new(is_selected, &short_name).ui(ui);

                                if response.clicked() {
                                    handle_asset_selection(asset_gallery_ui_state, asset_id, ui, is_selected, all_assets);
                                }
                                response.context_menu(|ui| {
                                    asset_context_menu(
                                        ui,
                                        asset_gallery_ui_state,
                                        action_queue,
                                        asset_id,
                                        &short_name,
                                        asset_info.asset_location(),
                                        &db_state
                                            .editor_model
                                    );
                                })
                            },
                        );
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
                        ui.label(long_name.as_ref().map(|x| x.as_str()).unwrap_or_default());
                    });
                });
            }
        });
}

fn create_drag_payload(asset_gallery_ui_state: &mut AssetGalleryUiState, asset_id: AssetId) -> DragDropPayload {
    if asset_gallery_ui_state.selected_assets.contains(&asset_id) {
        DragDropPayload::AssetReferences(asset_id, asset_gallery_ui_state.selected_assets.iter().copied().collect())
    } else {
        DragDropPayload::AssetReferences(asset_id, vec![asset_id])
    }
}

fn draw_asset_gallery_tile_grid(
    ui: &mut egui::Ui,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
    all_assets: &Vec<(&AssetId, &DataSetAssetInfo)>,
    thumbnail_image_loader: &ThumbnailImageLoader,
) {
    ui.style_mut().spacing.item_spacing = egui::vec2(12.0, 12.0);
    ui.with_layout(
        Layout::left_to_right(egui::Align::TOP).with_main_wrap(true),
        |ui| {
            for (asset_id, asset_info) in all_assets {
                draw_asset_gallery_tile(
                    ui,
                    db_state,
                    ui_state,
                    asset_gallery_ui_state,
                    **asset_id,
                    *asset_info,
                    action_queue,
                    all_assets,
                    thumbnail_image_loader,
                );
            }
        },
    );
}

fn draw_asset_gallery_tile(
    ui: &mut egui::Ui,
    db_state: &DbState,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    asset_id: AssetId,
    asset_info: &DataSetAssetInfo,
    action_queue: &UIActionQueueSender,
    all_assets: &Vec<(&AssetId, &DataSetAssetInfo)>,
    thumbnail_image_loader: &ThumbnailImageLoader,
) {
    let short_name = db_state
        .editor_model
        .root_edit_context()
        .asset_name_or_id_string(asset_id)
        .unwrap();
    let is_generated = db_state.editor_model.is_generated_asset(asset_id);
    let is_dirty = ui_state.edited_objects.contains(&asset_id);

    crate::ui::drag_drop::drag_source(
        ui,
        egui::Id::new(asset_id),
        &db_state.editor_model,
        ui_state,
        thumbnail_image_loader,
        asset_gallery_ui_state,
        |asset_gallery_ui_state| create_drag_payload(asset_gallery_ui_state, asset_id),
        |ui, asset_gallery_ui_state| {
            let is_on = false;

            let desired_size = egui::vec2(
                asset_gallery_ui_state.tile_size,
                asset_gallery_ui_state.tile_size + 30.0,
            );
            let thumbnail_size = egui::vec2(
                asset_gallery_ui_state.tile_size,
                asset_gallery_ui_state.tile_size,
            );

            let is_selected = asset_gallery_ui_state.selected_assets.contains(&asset_id);
            let is_primary_selected = asset_gallery_ui_state.primary_selected_asset == Some(asset_id);
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
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
                handle_asset_selection(asset_gallery_ui_state, asset_id, ui, is_selected, all_assets);
            }

            if ui.is_rect_visible(thumbnail_rect) {
                //let visuals = ui.style().interact_selectable(&response, is_on);
                let radius = 3.0;

                // Paint the selection background
                if is_selected {
                    ui.painter()
                        .rect_filled(rect, radius, ui.style().visuals.selection.bg_fill);
                }

                let thumbnail_uri = thumbnail_image_loader.thumbnail_uri_for_asset_with_fingerprint(asset_id, asset_info.schema().fingerprint());
                //egui::Image::new(thumbnail_uri).max_size(thumbnail_size).paint_at(ui, thumbnail_rect);
                let mut thumbnail_ui = ui.child_ui(thumbnail_rect, egui::Layout::centered_and_justified(egui::Direction::LeftToRight));
                thumbnail_ui.add(egui::Image::new(thumbnail_uri).max_size(thumbnail_size));

                let border_color = if is_primary_selected {
                    egui::Color32::from_rgb(255, 255, 0)
                } else if is_selected {
                    egui::Color32::from_rgb(255, 255, 255)
                } else {
                    egui::Color32::from_gray(50)
                };

                ui.painter().rect_stroke(
                    thumbnail_rect,
                    radius,
                    egui::Stroke::new(3.0, border_color),
                );
                let anchor =
                    egui::Pos2::new((text_rect.min.x + text_rect.max.x) / 2.0, text_rect.min.y + 8.0);

                let text_color = if is_dirty {
                    egui::Color32::from_rgb(255, 255, 100)
                } else if is_generated {
                    egui::Color32::from_rgb(97, 150, 199)
                } else {
                    egui::Color32::from_rgb(230, 230, 230)
                };

                let font_id = ui.ctx().style().text_styles.get(&egui::TextStyle::Body).unwrap().clone();
                let galley = ui.fonts(|x| {
                    let mut layout_job = LayoutJob::single_section(
                        short_name.clone(),
                        egui::epaint::text::TextFormat::simple(font_id.clone(), text_color)
                    );
                    layout_job.wrap.max_width = text_rect.max.x - text_rect.min.x;
                    layout_job.wrap.max_rows = 1;
                    layout_job.wrap.break_anywhere = false;

                    x.layout_job(layout_job)
                });

                let text = galley.rows[0].text();

                ui.painter().text(
                    anchor,
                    egui::Align2::CENTER_TOP,
                    text,
                    font_id.clone(),
                    text_color,
                );
            }

            let is_generated = is_generated;
            let response = response.context_menu(move |ui| {
                asset_context_menu(
                    ui,
                    asset_gallery_ui_state,
                    action_queue,
                    asset_id,
                    &short_name,
                    asset_info.asset_location(),
                    &db_state
                        .editor_model
                );
            });

            response
        },
    );
}

fn asset_context_menu(
    ui: &mut egui::Ui,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
    asset_id: AssetId,
    name: &str,
    location: AssetLocation,
    editor_model: &EditorModel,
) {
    if !asset_gallery_ui_state.selected_assets.contains(&asset_id) {
        asset_gallery_ui_state.select_one(asset_id);
    }

    let mut are_any_generated = false;
    for asset_id in &asset_gallery_ui_state.selected_assets {
        if editor_model.is_generated_asset(*asset_id) {
            are_any_generated = true;
            break;
        }
    }

    let mut are_any_generated = false;
    for asset_id in &asset_gallery_ui_state.selected_assets {
        if editor_model.is_generated_asset(*asset_id) {
            are_any_generated = true;
            break;
        }
    }

    if are_any_generated {
        ui.label("One or more assets are generated and cannot be edited directly");
    }

    if ui.button("Duplicate").clicked() {
        action_queue.queue_action(UIAction::DuplicateAssets(asset_gallery_ui_state.selected_assets.iter().copied().collect()));
        ui.close_menu();
    }

    let can_rename = asset_gallery_ui_state.selected_assets.len() == 1 && asset_gallery_ui_state.primary_selected_asset == Some(asset_id);
    let move_or_rename_text = if can_rename {
        "Move or Rename"
    } else {
        "Move"
    };

    if ui.add_enabled(!are_any_generated, egui::Button::new(move_or_rename_text)).clicked() {
        let current_name = editor_model.root_edit_context().asset_name(asset_id).unwrap();
        let current_location = editor_model.root_edit_context().asset_location(asset_id).unwrap();

        if can_rename {
            action_queue.try_set_modal_action(MoveAssetsModal::new_single_asset(
                asset_id,
                current_name.as_string().cloned().unwrap_or_else(|| asset_id.to_string()),
                Some(current_location)
            ));
        } else {
            action_queue.try_set_modal_action(MoveAssetsModal::new_multiple_assets(
                asset_gallery_ui_state.selected_assets.iter().copied().collect(),
                Some(current_location)
            ));
        }

        ui.close_menu();
    }

    let delete_button_string = if asset_gallery_ui_state.selected_assets.len() > 1 {
        format!("Delete {} assets", asset_gallery_ui_state.selected_assets.len())
    } else {
        format!("Delete {}", name)
    };

    if ui
        .add_enabled(!are_any_generated, egui::Button::new(delete_button_string))
        .clicked()
    {
        action_queue.queue_action(UIAction::DeleteAssets(asset_gallery_ui_state.selected_assets.iter().copied().collect()));
        ui.close_menu();
    }

    let can_use_as_prototype = editor_model.root_edit_context().import_info(asset_id).is_none() && asset_gallery_ui_state.selected_assets.len() == 1;
    if ui.add_enabled(can_use_as_prototype, egui::Button::new("Use as prototype for new asset")).clicked() {
        action_queue
            .try_set_modal_action(NewAssetModal::new_with_prototype(Some(location), asset_id));
        ui.close_menu();
    }
}

fn handle_asset_selection(
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    asset_id: AssetId,
    ui: &mut Ui,
    is_selected: bool,
    all_assets: &Vec<(&AssetId, &DataSetAssetInfo)>
) {
    let mut primary_index = None;
    let mut selected_index = None;
    let mut shift_select_begin_index = None;
    let mut shift_select_end_index = None;

    for (i, (id, _)) in all_assets.iter().enumerate() {
        if Some(**id) == asset_gallery_ui_state.primary_selected_asset {
            primary_index = Some(i);
        }

        if Some(**id) == asset_gallery_ui_state.previous_shift_select_range_begin {
            shift_select_begin_index = Some(i);
        }

        if Some(**id) == asset_gallery_ui_state.previous_shift_select_range_end {
            shift_select_end_index = Some(i);
        }

        if **id == asset_id {
            selected_index = Some(i);
        }
    }

    // selected index should exist but primary index might be none
    let selected_index = selected_index.unwrap();

    let (shift_held, command_held) = ui.input(|input| (input.modifiers.shift, input.modifiers.command));

    if command_held {
        // Command-clicking toggles selection
        if command_held && is_selected {
            // make something else primary?
            asset_gallery_ui_state.selected_assets.remove(&asset_id);
            asset_gallery_ui_state.primary_selected_asset = None;
            if let Some(shift_select_end_index) = shift_select_end_index {
                // If we deselect an item while we have a shift select range, we adjust the shift select range start
                // and maybe end position
                if shift_select_end_index > selected_index {
                    asset_gallery_ui_state.previous_shift_select_range_begin = Some(*all_assets[selected_index + 1].0);
                } else if shift_select_end_index < selected_index {
                    asset_gallery_ui_state.previous_shift_select_range_begin = Some(*all_assets[selected_index - 1].0);
                } else {
                    // Assumed not none is end index is set
                    let shift_select_begin_index = shift_select_begin_index.unwrap();
                    if selected_index > shift_select_begin_index {
                        asset_gallery_ui_state.previous_shift_select_range_begin = Some(*all_assets[selected_index - 1].0);
                        asset_gallery_ui_state.previous_shift_select_range_end = Some(*all_assets[selected_index - 1].0);
                    } else if selected_index < shift_select_begin_index{
                        asset_gallery_ui_state.previous_shift_select_range_begin = Some(*all_assets[selected_index + 1].0);
                        asset_gallery_ui_state.previous_shift_select_range_end = Some(*all_assets[selected_index + 1].0);
                    }
                }
            }
        } else {
            asset_gallery_ui_state.selected_assets.insert(asset_id);
            asset_gallery_ui_state.primary_selected_asset = Some(asset_id);
            asset_gallery_ui_state.previous_shift_select_range_begin = Some(asset_id);
            asset_gallery_ui_state.previous_shift_select_range_end = None;
        }
    } else if shift_held {
        // Shift-clicking adds/removes a range of objects to the selection
        if let Some(shift_select_begin_index) = shift_select_begin_index {
            // Undo the previous range selection
            if let Some(shift_select_end_index) = shift_select_end_index {
                let remove_selection_range = if shift_select_begin_index < shift_select_end_index {
                    shift_select_begin_index..=shift_select_end_index
                } else {
                    shift_select_end_index..=shift_select_begin_index
                };

                for i in remove_selection_range {
                    let (&asset_id_in_range, _) = all_assets[i];
                    asset_gallery_ui_state.selected_assets.remove(&asset_id_in_range);
                }
            }

            // Add the new range selection
            let add_selection_range = if shift_select_begin_index < selected_index {
                shift_select_begin_index..=selected_index
            } else {
                selected_index..=shift_select_begin_index
            };

            for i in add_selection_range {
                let (&asset_id_in_range, _) = all_assets[i];
                asset_gallery_ui_state.selected_assets.insert(asset_id_in_range);
            }

            let (&clicked_asset_id, _) = all_assets[selected_index];
            asset_gallery_ui_state.previous_shift_select_range_end = Some(clicked_asset_id);
            asset_gallery_ui_state.primary_selected_asset = Some(clicked_asset_id);
        } else {
            // We don't have a begin range so treat this as a plain click
            asset_gallery_ui_state.selected_assets.insert(asset_id);
            asset_gallery_ui_state.primary_selected_asset = Some(asset_id);
            asset_gallery_ui_state.previous_shift_select_range_begin = Some(asset_id);
        }
    } else {
        asset_gallery_ui_state.select_one(asset_id);
    }

    if asset_gallery_ui_state.previous_shift_select_range_end.is_some() {
        assert!(asset_gallery_ui_state.previous_shift_select_range_begin.is_some());
    }
}
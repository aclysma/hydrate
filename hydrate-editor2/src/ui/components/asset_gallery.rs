
use std::sync::Arc;
use egui::epaint::text::FontsImpl;
use egui::{FontDefinitions, FontId, Layout, SelectableLabel};
use hydrate_model::{AssetId, AssetName, EndContextBehavior, HashSet};
use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui::modals::TestModal;
use crate::ui_state::{AssetInfo, EditorModelUiState};

#[derive(Default)]
pub struct AssetGalleryUiState {
    search_string: String,
    pub selected_assets: HashSet<AssetId>,
}

pub fn draw_asset_gallery(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
) {
    //ui.label("asset gallery");

    //println!("available {:?}", ui.available_width());
    let (toolbar_id, toolbar_rect) = ui.allocate_space(egui::vec2(ui.available_width(), 30.0));

    //ui.child_ui(toolbar_rect)


    let mut child_ui =
        ui.child_ui(toolbar_rect, egui::Layout::left_to_right(egui::Align::Center));

    //ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
    if child_ui.button("button 1").clicked() {
        action_queue.try_set_modal_action(TestModal {

        });
    }
    child_ui.button("button 2");
    child_ui.button("button 3");
    egui::TextEdit::singleline(&mut asset_gallery_ui_state.search_string).desired_width(50.0).show(&mut child_ui);
    child_ui.button("button 3");

    let mut selected = "First";
    egui::ComboBox::from_label("Select one!")
        .selected_text(format!("{:?}", selected))
        .show_ui(&mut child_ui, |ui| {
            ui.selectable_value(&mut selected, "First", "First");
            ui.selectable_value(&mut selected, "Second", "Second");
            ui.selectable_value(&mut selected, "Third", "Third");
        }
        );
    //});

    if child_ui.available_width() > 200.0 {

        let mut child_ui =
            ui.child_ui(toolbar_rect, egui::Layout::right_to_left(egui::Align::Center));


        //ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
        child_ui.button("button 1");
        child_ui.button("button 2");
        child_ui.button("button 3");
        //});
    }

    ui.separator();


    egui::ScrollArea::vertical()
        .max_width(f32::INFINITY)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            draw_asset_gallery_tile_grid(ui, fonts_impl, font_id, ui_state, asset_gallery_ui_state, action_queue);
        });
}

fn draw_asset_gallery_tile_grid(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    ui_state: &EditorModelUiState,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    action_queue: &UIActionQueueSender,
) {
    ui.with_layout(Layout::left_to_right(egui::Align::TOP).with_main_wrap(true), |ui| {
        for (_, asset_info) in &ui_state.all_asset_info {
            draw_asset_gallery_tile(ui, fonts_impl, font_id, asset_gallery_ui_state, asset_info, action_queue);
        }
    });
}

fn draw_asset_gallery_tile(
    ui: &mut egui::Ui,
    fonts_impl: &mut FontsImpl,
    font_id: &FontId,
    asset_gallery_ui_state: &mut AssetGalleryUiState,
    asset_info: &AssetInfo,
    action_queue: &UIActionQueueSender,
) {
    crate::ui::drag_drop::drag_source(ui, egui::Id::new(asset_info.id), DragDropPayload::AssetReference(asset_info.id), |ui| {
        let mut is_on = false;

        let desired_size = egui::vec2(150.0, 190.0);
        let thumbnail_size = egui::vec2(150.0, 150.0);

        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        // ui.allocate_ui(desired_size, |ui| {
        //     ui.painter().rect_stroke(thumbnail_size, 3.0, egui::Stroke::new(2.0, egui::Color32::from_gray(50)));
        //     ui.label("hi");
        // });

        let asset_name_as_string = asset_info.name.as_string().cloned().unwrap_or_else(|| asset_info.id.to_string());

        let mut thumbnail_rect = rect;
        thumbnail_rect.max.y = thumbnail_rect.max.y.min(thumbnail_rect.min.y + thumbnail_size.y);
        let mut text_rect = rect;
        text_rect.min.y = thumbnail_rect.max.y;

        if response.clicked() {
            asset_gallery_ui_state.selected_assets.clear();
            asset_gallery_ui_state.selected_assets.insert(asset_info.id);
        }

        if ui.is_visible() {
            let how_on = ui.ctx().animate_bool(response.id, is_on);
            let visuals = ui.style().interact_selectable(&response, is_on);
            let radius = 3.0;

            if asset_gallery_ui_state.selected_assets.contains(&asset_info.id) {
                ui.painter().rect_filled(rect, radius, ui.style().visuals.selection.bg_fill);
            }
            ui.painter()
                .rect_stroke(thumbnail_rect, radius, egui::Stroke::new(2.0, egui::Color32::from_gray(50)));

            let anchor = egui::Pos2::new((text_rect.min.x + text_rect.max.x) / 2.0, text_rect.min.y);

            let text_color = if asset_info.is_dirty {
                egui::Color32::from_rgb(255, 255, 0)
            } else if asset_info.is_generated {
                egui::Color32::from_rgb(150, 150, 150)
            } else {
                egui::Color32::from_rgb(230, 230, 230)
            };

            let mut layout_job = egui::epaint::text::LayoutJob::single_section(
                asset_name_as_string.clone(),
                egui::epaint::text::TextFormat::simple(font_id.clone(), text_color)
            );
            layout_job.wrap.max_width = text_rect.max.x - text_rect.min.x;
            layout_job.wrap.max_rows = 1;
            layout_job.wrap.break_anywhere = false;
            let galley = egui::epaint::text::layout(fonts_impl, Arc::new(layout_job));
            let text = galley.rows[0].text();

            ui.painter().text(anchor, egui::Align2::CENTER_TOP, text, font_id.clone(), text_color);
        } else {
            println!("not visible");
        }

        let is_generated = asset_info.is_generated;
        let asset_id = asset_info.id;
        let response = response.context_menu(move |ui| {
            if is_generated {
                ui.label("This asset is generated and cannot be edited directly");
            }
            if ui.add_enabled(!is_generated, egui::Button::new(format!("Delete {}", &asset_name_as_string))).clicked() {
                action_queue.queue_edit("delete asset", vec![asset_id], move |edit_context| {
                    edit_context.delete_asset(asset_id).unwrap();
                    Ok(EndContextBehavior::Finish)
                });
                ui.close_menu();
            }

            if ui.button("Use as prototype for new asset").clicked() {
                action_queue.queue_edit("delete asset", vec![asset_id], move |edit_context| {
                    let old_location = edit_context.asset_location(asset_id).unwrap().clone();
                    let old_name = edit_context.asset_name(asset_id).unwrap().clone();
                    let new_name = format!("New from {:?}", asset_id);

                    edit_context.new_asset_from_prototype(&AssetName::new(new_name), &old_location, asset_id).unwrap();
                    Ok(EndContextBehavior::Finish)
                });
                ui.close_menu();
            }

            // else {
            //
            //     // if ui.button().clicked() {
            //     //     let asset_id = asset_info.id;
            //     //     if asset_info.is_generated {
            //     //
            //     //     } else {
            //     //     }
            //     //
            //     // }
            // }
        });

        response
    });
}
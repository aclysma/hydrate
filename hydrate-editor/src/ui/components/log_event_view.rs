use egui::Widget;
use hydrate_model::EditorModel;
use hydrate_model::pipeline::AssetEngine;
use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui_state::EditorModelUiState;

#[derive(Default)]
pub struct LogEventViewUiState {
    search_string: String,
}

pub fn draw_log_event_view(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    editor_model_ui_state: &EditorModelUiState,
    asset_engine: &AssetEngine,
    log_event_view_ui_state: &mut LogEventViewUiState,
    action_queue_sender: &UIActionQueueSender,
) {




    ui.allocate_space(egui::vec2(0.0, 0.0));
    ui.horizontal(|ui| {
        ui.allocate_space(egui::vec2(0.0, 0.0));

        let mut selected = "aaa";
        egui::ComboBox::new("build_selector", "Build Selection").selected_text("aaa").show_ui(ui, |ui| {
            ui.selectable_value(&mut selected, "aaa", "aaa");
            ui.selectable_value(&mut selected, "bbb", "bbb");
            ui.selectable_value(&mut selected, "ccc", "ccc");
        });

        ui.button("test");
        ui.label("Search:");
        ui.add(egui::TextEdit::singleline(&mut log_event_view_ui_state.search_string)
            .desired_width(250.0));
    });

    ui.separator();

    ui.style_mut().spacing = ui.ctx().style().spacing.clone();

    let (_, mut next_rect) = ui.allocate_space(ui.available_size());
    next_rect.min.x += 8.0;
    next_rect.max.x -= 8.0;
    let mut child_ui = ui.child_ui(next_rect, egui::Layout::top_down(egui::Align::Center));
    egui::ScrollArea::both()
        .max_width(f32::INFINITY)
        .max_height(f32::INFINITY)
        .auto_shrink([false, false])
        .show(&mut child_ui, |ui| {
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
                        .clip(true))
                .column(
                    egui_extras::Column::initial(30.0)
                        .at_least(30.0)
                        .clip(true))
                .column(egui_extras::Column::remainder());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Source");
                    });
                    header.col(|ui| {
                        ui.strong("Level");
                    });
                    header.col(|ui| {
                        ui.strong("Message");
                    });
                })
                .body(|mut body| {
                    if let Some(build_log_data) = asset_engine.most_recent_build_log_data() {
                        for log_event in build_log_data.log_events() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{:?}", log_event.level));
                                });
                                row.col(|ui| {
                                    if let Some(asset_id) = log_event.asset_id {
                                        let long_name = editor_model
                                            .asset_display_name_long(asset_id, &editor_model_ui_state.asset_path_cache);

                                        if ui.button(">>").clicked() {
                                            action_queue_sender.queue_action(UIAction::ShowAssetInAssetGallery(asset_id));
                                        }

                                        ui.label(long_name);
                                    } else if let Some(job_id) = log_event.job_id {
                                        let assets = build_log_data.assets_relying_on_job(job_id);

                                        if assets.len() >= 1 {
                                            if ui.button(">>").clicked() {
                                                action_queue_sender.queue_action(UIAction::ShowAssetInAssetGallery(assets[0]));
                                            }
                                        }

                                        if assets.len() > 1 {
                                            let long_name = editor_model
                                                .asset_display_name_long(assets[0], &editor_model_ui_state.asset_path_cache);
                                            ui.label(format!("[+{}] {}", assets.len() - 1, long_name));
                                        } else if assets.len() == 1 {
                                            let long_name = editor_model
                                                .asset_display_name_long(assets[0], &editor_model_ui_state.asset_path_cache);
                                            ui.label(format!("{}", long_name));
                                        } else {
                                            ui.label(format!("Job {}", job_id.as_uuid()));
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(&log_event.message);
                                });
                            });
                        }
                    }
                });
        });
}
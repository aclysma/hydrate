use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui_state::EditorModelUiState;
use egui::Widget;
use hydrate_model::pipeline::{AssetEngine, BuildLogData, ImportLogData, LogData, LogDataRef};
use hydrate_model::EditorModel;
use serde_json::to_string;
use uuid::Uuid;

#[derive(Default)]
pub struct LogEventViewUiState {
    selected_log: Option<Uuid>,
    search_string: String,
}

pub fn draw_import_log(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    editor_model_ui_state: &EditorModelUiState,
    log_event_view_ui_state: &mut LogEventViewUiState,
    action_queue_sender: &UIActionQueueSender,
    import_log_data: &ImportLogData,
) {
    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 2.0);
    ui.push_id("import_log", |ui| {
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
            .column(egui_extras::Column::initial(30.0).at_least(30.0).clip(true))
            .column(egui_extras::Column::remainder());

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Level");
                });
                header.col(|ui| {
                    ui.strong("Source");
                });
                header.col(|ui| {
                    ui.strong("Message");
                });
            })
            .body(|mut body| {
                for log_event in import_log_data.log_events() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{:?}", log_event.level));
                        });
                        row.col(|ui| {
                            ui.label(log_event.path.to_string_lossy());
                        });
                        row.col(|ui| {
                            ui.label(&log_event.message);
                        });
                    });
                }
            });
    });
}

pub fn draw_build_log(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    editor_model_ui_state: &EditorModelUiState,
    log_event_view_ui_state: &mut LogEventViewUiState,
    action_queue_sender: &UIActionQueueSender,
    build_log_data: &BuildLogData,
) {
    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 2.0);
    ui.push_id("build_log", |ui| {
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
            .column(egui_extras::Column::initial(30.0).at_least(30.0).clip(true))
            .column(egui_extras::Column::remainder());

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Level");
                });
                header.col(|ui| {
                    ui.strong("Source");
                });
                header.col(|ui| {
                    ui.strong("Message");
                });
            })
            .body(|mut body| {
                for log_event in build_log_data.log_events() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{:?}", log_event.level));
                        });
                        row.col(|ui| {
                            if let Some(asset_id) = log_event.asset_id {
                                let long_name = editor_model.asset_display_name_long(
                                    asset_id,
                                    &editor_model_ui_state.asset_path_cache,
                                );

                                if ui.button(">>").clicked() {
                                    action_queue_sender
                                        .queue_action(UIAction::ShowAssetInAssetGallery(asset_id));
                                }

                                ui.label(long_name);
                            } else if let Some(job_id) = log_event.job_id {
                                let assets = build_log_data.assets_relying_on_job(job_id);

                                if assets.len() >= 1 {
                                    if ui.button(">>").clicked() {
                                        action_queue_sender.queue_action(
                                            UIAction::ShowAssetInAssetGallery(assets[0]),
                                        );
                                    }
                                }

                                if assets.len() > 1 {
                                    let long_name = editor_model.asset_display_name_long(
                                        assets[0],
                                        &editor_model_ui_state.asset_path_cache,
                                    );
                                    ui.label(format!("[+{}] {}", assets.len() - 1, long_name));
                                } else if assets.len() == 1 {
                                    let long_name = editor_model.asset_display_name_long(
                                        assets[0],
                                        &editor_model_ui_state.asset_path_cache,
                                    );
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
            });
    });
}

pub fn draw_log_event_view(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    editor_model_ui_state: &EditorModelUiState,
    asset_engine: &AssetEngine,
    log_event_view_ui_state: &mut LogEventViewUiState,
    action_queue_sender: &UIActionQueueSender,
    previous_log_data: &[LogData],
) {
    let format_description = time::format_description::parse("[hour]:[minute]:[second]").unwrap();

    fn log_label(
        log_data: &LogData,
        time_format: &[time::format_description::FormatItem],
    ) -> String {
        let formatted_start_time = time::OffsetDateTime::from(log_data.start_time())
            .format(time_format)
            .unwrap();
        match log_data {
            LogData::Import(import_data) => format!("Import at {}", formatted_start_time),
            LogData::Build(build_data) => format!("Build at {}", formatted_start_time),
        }
    }

    ui.allocate_space(egui::vec2(0.0, 0.0));
    ui.horizontal(|ui| {
        ui.allocate_space(egui::vec2(0.0, 0.0));

        let mut selected_label_formatted = None;
        if let Some(selected_log) = &log_event_view_ui_state.selected_log {
            if let Some(selected_log) = previous_log_data.iter().find(|x| x.id() == *selected_log) {
                selected_label_formatted = Some(log_label(selected_log, &format_description));
            }
        };

        let selected_label = if let Some(selected_label_formatted) = &selected_label_formatted {
            selected_label_formatted.as_str()
        } else {
            log_event_view_ui_state.selected_log = None;
            "Most Recent"
        };

        let selected_log = &mut log_event_view_ui_state.selected_log;
        egui::ComboBox::new("build_selector", "Build Selection")
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(selected_log, None, "Most Recent");

                for previous_log in previous_log_data.iter().rev() {
                    let formatted_start_time =
                        time::OffsetDateTime::from(previous_log.start_time())
                            .format(&format_description)
                            .unwrap();
                    let label = match previous_log {
                        LogData::Import(import_data) => {
                            format!("Import at {}", formatted_start_time)
                        }
                        LogData::Build(build_data) => format!("Build at {}", formatted_start_time),
                    };

                    ui.selectable_value(selected_log, Some(previous_log.id()), label);
                }
            });

        /*
        ui.button("test");
        ui.label("Search:");
        ui.add(egui::TextEdit::singleline(&mut log_event_view_ui_state.search_string)
            .desired_width(250.0));

         */
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
                        .clip(true),
                )
                .column(egui_extras::Column::initial(30.0).at_least(30.0).clip(true))
                .column(egui_extras::Column::remainder());

            if let Some(selected_log_id) = log_event_view_ui_state.selected_log {
                //
                // Draw the selected log
                //
                let log = previous_log_data.iter().find(|x| x.id() == selected_log_id);
                match log {
                    Some(log_data) => match log_data {
                        LogData::Import(import_log_data) => draw_import_log(
                            ui,
                            editor_model,
                            editor_model_ui_state,
                            log_event_view_ui_state,
                            action_queue_sender,
                            import_log_data,
                        ),
                        LogData::Build(build_log_data) => draw_build_log(
                            ui,
                            editor_model,
                            editor_model_ui_state,
                            log_event_view_ui_state,
                            action_queue_sender,
                            build_log_data,
                        ),
                    },
                    None => {}
                }
            } else {
                //
                // Draw either the log from current in-progress task or the most recent log
                //
                match asset_engine.current_task_log_data() {
                    LogDataRef::Import(import_log_data) => draw_import_log(
                        ui,
                        editor_model,
                        editor_model_ui_state,
                        log_event_view_ui_state,
                        action_queue_sender,
                        import_log_data,
                    ),
                    LogDataRef::Build(build_log_data) => draw_build_log(
                        ui,
                        editor_model,
                        editor_model_ui_state,
                        log_event_view_ui_state,
                        action_queue_sender,
                        build_log_data,
                    ),
                    LogDataRef::None => {
                        // Pick the most recent previous build
                        let mut most_recent_log_data: Option<&LogData> = None;
                        for previous_log in previous_log_data {
                            if let Some(most_recent) = most_recent_log_data {
                                if most_recent.start_time() < previous_log.start_time() {
                                    most_recent_log_data = Some(previous_log);
                                }
                            } else {
                                most_recent_log_data = Some(previous_log);
                            }
                        }

                        if let Some(most_recent_log_data) = most_recent_log_data {
                            match most_recent_log_data {
                                LogData::Import(import_log_data) => draw_import_log(
                                    ui,
                                    editor_model,
                                    editor_model_ui_state,
                                    log_event_view_ui_state,
                                    action_queue_sender,
                                    import_log_data,
                                ),
                                LogData::Build(build_log_data) => draw_build_log(
                                    ui,
                                    editor_model,
                                    editor_model_ui_state,
                                    log_event_view_ui_state,
                                    action_queue_sender,
                                    build_log_data,
                                ),
                            }
                        }
                    }
                }
            }
        });
}

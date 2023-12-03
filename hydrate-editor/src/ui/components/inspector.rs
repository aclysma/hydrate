use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui_state::EditorModelUiState;
use eframe::epaint::Color32;
use egui::{Response, TextStyle, Widget};
use hydrate_model::value::ValueEnum;
use hydrate_model::{
    AssetId, EditorModel, EndContextBehavior, HashSet, NullOverride, PropertyPath, Schema,
    SchemaNamedType, SchemaSet, Value,
};
use std::sync::Arc;
use crate::ui::modals::NewAssetModal;

fn join_field_path(
    lhs: &str,
    rhs: &str,
) -> String {
    assert!(!rhs.is_empty());
    if lhs.is_empty() {
        rhs.to_string()
    } else {
        format!("{}.{}", lhs, rhs)
    }
}

#[derive(Copy, Clone)]
struct InspectorContext<'a> {
    editor_model: &'a EditorModel,
    editor_model_ui_state: &'a EditorModelUiState,
    action_sender: &'a UIActionQueueSender,
    asset_id: AssetId,
    property_path: &'a str,
    property_name: &'a str,
    schema: &'a hydrate_model::Schema,
    read_only: bool,
}

fn simple_value_property<
    F: FnOnce(&mut egui::Ui, InspectorContext) -> Option<(Value, EndContextBehavior)>,
>(
    ui: &mut egui::Ui,
    ctx: InspectorContext,
    f: F,
) {
    let has_override = ctx
        .editor_model
        .root_edit_context()
        .has_property_override(ctx.asset_id, ctx.property_path)
        .unwrap();
    let assets_to_edit = vec![ctx.asset_id];

    ui.horizontal(|ui| {
        ui.set_enabled(!ctx.read_only);
        if !has_override {
            ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(150));
        } else {
            ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(255));
        }
        let label_response = egui::Label::new(ctx.property_name).ui(ui);
        show_property_context_menu(ctx, label_response);

        if let Some((new_value, end_context_behavior)) = f(
            ui,
            ctx,
        ) {
            let property_path_moved = ctx.property_path.to_string();
            ctx.action_sender
                .queue_edit("property_editor", assets_to_edit, move |edit_context| {
                    edit_context
                        .set_property_override(ctx.asset_id, property_path_moved, Some(new_value))
                        .unwrap();
                    Ok(end_context_behavior)
                });
        }
    });
}

fn end_context_behavior_for_drag_value(
    ctx: InspectorContext,
    response: egui::Response,
    new_value: Value,
) -> Option<(Value, EndContextBehavior)> {
    let behavior = if response.lost_focus() || response.drag_released() {
        EndContextBehavior::Finish
    } else {
        EndContextBehavior::AllowResume
    };

    Some((new_value, behavior))
}

fn end_context_behavior_for_text_field(
    ctx: InspectorContext,
    response: egui::Response,
    new_value: Value,
) -> Option<(Value, EndContextBehavior)> {
    let behavior = if response.lost_focus() || response.drag_released() {
        EndContextBehavior::Finish
    } else {
        EndContextBehavior::AllowResume
    };

    Some((new_value, behavior))
}

fn end_context_behavior_for_checkbox(
    ctx: InspectorContext,
    response: egui::Response,
    new_value: Value,
) -> Option<(Value, EndContextBehavior)> {
    let behavior = EndContextBehavior::Finish;

    Some((new_value, behavior))
}

fn draw_inspector_property(
    ui: &mut egui::Ui,
    ctx: InspectorContext,
) {
    ui.push_id(ctx.property_path, |ui| {
        match ctx.schema {
            Schema::Nullable(inner_schema) => {
                let null_override = ctx
                    .editor_model
                    .root_edit_context()
                    .get_null_override(ctx.asset_id, ctx.property_path)
                    .unwrap();
                let resolved_null_override = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_null_override(ctx.asset_id, ctx.property_path)
                    .unwrap();

                ui.horizontal(|ui| {
                    ui.set_enabled(!ctx.read_only);
                    if null_override == NullOverride::Unset {
                        ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(150));
                    } else {
                        ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(255));
                    }
                    // No response/context menu for now, the buttons act to "clear" this field
                    ui.label(ctx.property_name);

                    let mut new_null_override = None;
                    if resolved_null_override != NullOverride::SetNonNull {
                        if ui.button("Set Non-Null").clicked() {
                            new_null_override = Some(NullOverride::SetNonNull);
                        }
                    } else {
                        if ui.button("Set Null").clicked() {
                            new_null_override = Some(NullOverride::SetNull);
                        }
                    }

                    ui.add_enabled_ui(null_override != NullOverride::Unset, |ui| {
                        if ui.button("Inherit null status").clicked() {
                            new_null_override = Some(NullOverride::Unset);
                        }
                    });

                    if let Some(new_null_override) = new_null_override {
                        let captured_property_path = ctx.property_path.to_string();
                        ctx.action_sender.queue_edit(
                            "property_editor",
                            vec![ctx.asset_id],
                            move |edit_context| {
                                edit_context.set_null_override(
                                    ctx.asset_id,
                                    captured_property_path,
                                    new_null_override,
                                )?;
                                Ok(EndContextBehavior::Finish)
                            },
                        );
                    }
                });

                if resolved_null_override == NullOverride::SetNonNull {
                    ui.indent(ctx.property_path, |ui| {
                        let field_path = join_field_path(ctx.property_path, "value");
                        draw_inspector_property(
                            ui,
                            InspectorContext {
                                property_name: "value",
                                property_path: &field_path,
                                schema: &*inner_schema,
                                ..ctx
                            },
                        )
                    });
                }
            }
            Schema::Boolean => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_boolean()
                    .unwrap();
                let response = egui::Checkbox::new(&mut value, "").ui(ui);
                if response.changed() {
                    end_context_behavior_for_checkbox(ctx, response, Value::Boolean(value))
                } else {
                    None
                }
            }),
            Schema::I32 => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_i32()
                    .unwrap();
                let response = egui::DragValue::new(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_drag_value(ctx, response, Value::I32(value))
                } else {
                    None
                }
            }),
            Schema::I64 => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_i64()
                    .unwrap();
                let response = egui::DragValue::new(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_drag_value(ctx, response, Value::I64(value))
                } else {
                    None
                }
            }),
            Schema::U32 => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_u32()
                    .unwrap();
                let response = egui::DragValue::new(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_drag_value(ctx, response, Value::U32(value))
                } else {
                    None
                }
            }),
            Schema::U64 => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_u64()
                    .unwrap();
                let response = egui::DragValue::new(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_drag_value(ctx, response, Value::U64(value))
                } else {
                    None
                }
            }),
            Schema::F32 => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_f32()
                    .unwrap();
                let response = egui::DragValue::new(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_drag_value(ctx, response, Value::F32(value))
                } else {
                    None
                }
            }),
            Schema::F64 => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let response = egui::DragValue::new(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_drag_value(ctx, response, Value::F64(value))
                } else {
                    None
                }
            }),
            Schema::Bytes => {
                ui.label(format!(
                    "{}: Unsupported Schema::Bytes Property",
                    ctx.property_name
                ));
            }
            Schema::String => simple_value_property(ui, ctx, |ui, ctx| {
                let mut value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .to_string();
                let response = egui::TextEdit::singleline(&mut value).ui(ui);
                if response.changed() {
                    end_context_behavior_for_text_field(
                        ctx,
                        response,
                        Value::String(Arc::new(value)),
                    )
                } else {
                    None
                }
            }),
            // Schema::StaticArray(_) => unimplemented!(),
            Schema::DynamicArray(schema) => {
                let resolved = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_dynamic_array(ctx.asset_id, ctx.property_path)
                    .unwrap();
                let overrides = ctx
                    .editor_model
                    .root_edit_context()
                    .get_dynamic_array_overrides(ctx.asset_id, ctx.property_path)
                    .unwrap();
                //ui.push_id(ctx.property_path, |ui| {
                ui.collapsing("elements", |ui| {
                    // The immutable inherited elements
                    for id in &resolved[0..(resolved.len() - overrides.len())] {
                        let id_as_string = id.to_string();
                        let field_path = join_field_path(ctx.property_path, &id_as_string);
                        let header = format!("{} (inherited)", id_as_string);
                        ui.collapsing(&header, |ui| {
                            draw_inspector_property(
                                ui,
                                InspectorContext {
                                    property_name: &id_as_string,
                                    property_path: &field_path,
                                    schema: schema.item_type(),
                                    read_only: true,
                                    ..ctx
                                },
                            )
                        });
                    }

                    // The elements added by this asset
                    for id in overrides {
                        let id_as_string = id.to_string();
                        let field_path = join_field_path(ctx.property_path, &id_as_string);
                        ui.collapsing(&id_as_string, |ui| {
                            draw_inspector_property(
                                ui,
                                InspectorContext {
                                    property_name: &id_as_string,
                                    property_path: &field_path,
                                    schema: schema.item_type(),
                                    ..ctx
                                },
                            )
                        });
                    }
                });
                //});
            }
            // Schema::Map(_) => unimplemented!(),
            Schema::AssetRef(_) => {
                let resolved_value = ctx
                    .editor_model
                    .root_edit_context()
                    .resolve_property(ctx.asset_id, ctx.property_path)
                    .unwrap();
                let has_override = ctx
                    .editor_model
                    .root_edit_context()
                    .has_property_override(ctx.asset_id, ctx.property_path)
                    .unwrap();

                let asset_ref = resolved_value.as_asset_ref().unwrap();

                ui.horizontal(|ui| {
                    if !has_override {
                        ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(150));
                    } else {
                        ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(255));
                    }
                    let label_response = ui.label(ctx.property_name);
                    show_property_context_menu(ctx, label_response);

                    let mut label_string = if asset_ref.is_null() {
                        "not set".to_string()
                    } else {
                        let asset_path = ctx.editor_model.asset_path(asset_ref, &ctx.editor_model_ui_state.asset_path_cache);
                        asset_path.as_str().to_string()
                    };

                    if ui.add_enabled(!asset_ref.is_null(), egui::Button::new(">>")).clicked() {
                        ctx.action_sender.queue_action(UIAction::ShowAssetInAssetGallery(asset_ref));
                    }

                    // Set enabled after the "go to" button
                    ui.set_enabled(!ctx.read_only);

                    let can_accept_what_is_being_dragged = !ctx.read_only;
                    let response = crate::ui::drag_drop::drop_target(
                        ui,
                        can_accept_what_is_being_dragged,
                        |ui| {
                            ui.add_enabled_ui(false, |ui| {
                                ui.text_edit_singleline(&mut label_string);
                            })
                        },
                    )
                    .response;

                    if let Some(payload) =
                        crate::ui::drag_drop::try_take_dropped_payload(ui, &response)
                    {
                        match payload {
                            DragDropPayload::AssetReference(payload_asset_id) => {
                                //println!("Dropped {:?} over {:?}", asset_id, ctx.property_path);
                                let captured_property_path = ctx.property_path.to_string();
                                let asset_id = ctx.asset_id;
                                ctx.action_sender.queue_edit(
                                    "property_editor",
                                    vec![ctx.asset_id],
                                    move |edit_context| {
                                        edit_context
                                            .set_property_override(
                                                asset_id,
                                                captured_property_path,
                                                Some(Value::AssetRef(payload_asset_id)),
                                            )
                                            .unwrap();
                                        Ok(EndContextBehavior::Finish)
                                    },
                                );
                            }
                            _ => unimplemented!(),
                        }
                    }
                });
            }
            Schema::Record(schema_fingerprint) | Schema::Enum(schema_fingerprint) | Schema::Fixed(schema_fingerprint) => {
                let schema = ctx
                    .editor_model
                    .schema_set()
                    .find_named_type_by_fingerprint(*schema_fingerprint)
                    .unwrap();
                match schema {
                    SchemaNamedType::Record(record_schema) => {
                        let widgets = |ui: &mut egui::Ui| {
                            for field in record_schema.fields() {
                                if field.field_schema().is_dynamic_array() || field.field_schema().is_static_array() || field.field_schema().is_nullable() || field.field_schema().is_record() {
                                    let field_path = join_field_path(ctx.property_path, field.name());

                                    ui.label(field.name());
                                    draw_inspector_property(
                                        ui,
                                        InspectorContext {
                                            property_name: field.name(),
                                            property_path: &field_path,
                                            schema: field.field_schema(),
                                            ..ctx
                                        },
                                    );

                                    ui.end_row();
                                }
                            }

                            egui::Grid::new("properties").show(ui, |ui| {
                                for field in record_schema.fields() {
                                    if field.field_schema().is_dynamic_array() || field.field_schema().is_static_array() || field.field_schema().is_nullable() || field.field_schema().is_record() {
                                        continue;
                                    }

                                    let field_path = join_field_path(ctx.property_path, field.name());

                                    ui.label(field.name());
                                    draw_inspector_property(
                                        ui,
                                        InspectorContext {
                                            property_name: field.name(),
                                            property_path: &field_path,
                                            schema: field.field_schema(),
                                            ..ctx
                                        },
                                    );

                                    ui.end_row();
                                }
                            });
                        };

                        if ctx.property_path.is_empty() {
                            widgets(ui)
                        } else {
                            egui::CollapsingHeader::new(record_schema.name()).default_open(true).show(ui, |ui| {
                                widgets(ui)
                            });
                        }
                    }
                    SchemaNamedType::Enum(enum_schema) => {
                        //ui.push_id(ctx.property_path, |ui| {
                        let resolved = ctx
                            .editor_model
                            .root_edit_context()
                            .resolve_property(ctx.asset_id, ctx.property_path)
                            .unwrap();
                        let has_override = ctx
                            .editor_model
                            .root_edit_context()
                            .has_property_override(ctx.asset_id, ctx.property_path)
                            .unwrap();
                        let old_symbol_name = resolved.as_enum().unwrap().symbol_name().to_string();
                        let mut selected_symbol_name = old_symbol_name.clone();
                        let asset_id = ctx.asset_id;

                        ui.horizontal(|ui| {
                            ui.set_enabled(!ctx.read_only);
                            if !has_override {
                                ui.style_mut().visuals.override_text_color =
                                    Some(Color32::from_gray(150));
                            } else {
                                ui.style_mut().visuals.override_text_color =
                                    Some(Color32::from_gray(255));
                            }
                            let label_response = ui.label(ctx.property_name);
                            show_property_context_menu(ctx, label_response);

                            let response = egui::ComboBox::new(ctx.property_path, "")
                                .selected_text(&selected_symbol_name)
                                .show_ui(ui, |ui| {
                                    for symbol in enum_schema.symbols() {
                                        ui.selectable_value(
                                            &mut selected_symbol_name,
                                            symbol.name().to_string(),
                                            symbol.name(),
                                        );
                                    }
                                })
                                .response;

                            if old_symbol_name != selected_symbol_name {
                                let new_value = Value::Enum(ValueEnum::new(selected_symbol_name));
                                let captured_property_path = ctx.property_path.to_string();
                                ctx.action_sender.queue_edit(
                                    "property_editor",
                                    vec![asset_id],
                                    move |edit_context| {
                                        edit_context.set_property_override(
                                            asset_id,
                                            captured_property_path,
                                            Some(new_value),
                                        )?;
                                        Ok(EndContextBehavior::Finish)
                                    },
                                );
                            }

                            response
                        });
                    }
                    // SchemaNamedType::Fixed(_) => unimplemented!(),
                    _ => {
                        ui.label(format!("unimplemented {:?} {}", schema, ctx.property_name));
                    }
                }
            }
            _ => {
                ui.label(format!(
                    "unimplemented {:?} {}",
                    ctx.schema, ctx.property_name
                ));
            }
        }
    });
}

fn show_property_context_menu(
    ctx: InspectorContext,
    response: Response,
) -> Response {
    let asset_id = ctx.asset_id;
    response.context_menu(|ui| {
        let has_override = ctx
            .editor_model
            .root_edit_context()
            .has_property_override(asset_id, ctx.property_path)
            .unwrap();
        if ui
            .add_enabled(
                has_override && !ctx.read_only,
                egui::Button::new("Clear Override"),
            )
            .clicked()
        {
            let captured_property_path = ctx.property_path.to_string();
            ctx.action_sender
                .queue_edit("property_editor", vec![asset_id], move |edit_context| {
                    edit_context.set_property_override(asset_id, captured_property_path, None)?;
                    Ok(EndContextBehavior::Finish)
                });
            ui.close_menu();
        }

        let has_prototype = ctx
            .editor_model
            .root_edit_context()
            .asset_prototype(asset_id)
            .is_some();
        if ui
            .add_enabled(
                has_prototype && !ctx.read_only,
                egui::Button::new("Apply Override"),
            )
            .clicked()
        {
            let captured_property_path = ctx.property_path.to_string();
            ctx.action_sender
                .queue_edit("property_editor", vec![asset_id], move |edit_context| {
                    edit_context
                        .apply_property_override_to_prototype(asset_id, captured_property_path)?;
                    Ok(EndContextBehavior::Finish)
                });
            ui.close_menu();
        }
    })
}

#[derive(Default)]
pub struct InspectorUiState {}

pub fn draw_inspector(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    asset_id: Option<AssetId>,
) {
    egui::ScrollArea::vertical()
        .max_width(f32::INFINITY)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label("hi");

            let available_x = ui.available_width();

            // can I make tables share column widths?

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

            fn add_empty_collapsing_header(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> bool {
                let openness = egui::CollapsingHeader::new(text).show_unindented(ui, |ui| {

                }).openness;
                openness > 0.5
            }

            table
                // .header(20.0, |mut header| {
                //     header.col(|ui| {
                //         ui.strong("Name");
                //     });
                //     header.col(|ui| {
                //         ui.strong("Type");
                //     });
                // })
                .body(|mut body| {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            crate::ui::add_icon_spacing(ui);
                            ui.label("a");
                        });
                        row.col(|ui| {
                            ui.label("b");
                        });
                    });

                    let mut is_open = false;


                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            // no spacing because the collapsing header uses it
                            is_open = add_empty_collapsing_header(ui, "test");
                        });
                        row.col(|ui| {
                            //ui.label("b");
                        });
                    });

                    if is_open {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                crate::ui::add_indent_spacing(ui);
                                crate::ui::add_icon_spacing(ui);
                                ui.label("a fasd faf asdf asf asf ");
                            });
                            row.col(|ui| {
                                ui.label("b");
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                crate::ui::add_indent_spacing(ui);
                                crate::ui::add_icon_spacing(ui);
                                ui.label("a fasd faf asdf asf asf ");
                            });
                            row.col(|ui| {
                                ui.label("b");
                            });
                        });

                        let mut is_open2 = false;
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                crate::ui::add_indent_spacing(ui);
                                // no spacing because the collapsing header uses it
                                is_open2 = add_empty_collapsing_header(ui, "a fasd faf asdf asf asf ");
                            });
                            row.col(|ui| {
                                ui.label("b");
                            });
                        });

                        if is_open2 {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    crate::ui::add_indent_spacing(ui);
                                    crate::ui::add_indent_spacing(ui);
                                    crate::ui::add_icon_spacing(ui);
                                    ui.label("a");
                                });
                                row.col(|ui| {
                                    ui.label("ba sdfasdfasdf asd fsd fasdf as fsadf ");
                                });
                            });
                        }
                    }



                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            crate::ui::add_icon_spacing(ui);
                            ui.label("a");
                        });
                        row.col(|ui| {
                            ui.label("ba sdfasdfasdf asd fsd fasdf as fsadf ");
                        });
                    });
                });


            // // egui::Grid::new("some_unique_id").show(ui, |ui| {
            // //     ui.label("First row, first column");
            // //     ui.label("First row, second column");
            // //     ui.end_row();
            // //
            // //     ui.label("Second row, first column");
            // //     ui.label("Second row, second column");
            // //     ui.label("Second row, third column");
            // //     ui.end_row();
            // //
            // //     ui.horizontal(|ui| { ui.label("Same"); ui.label("cell"); });
            // //     ui.label("Third row, second column");
            // //     ui.end_row();
            // // });
            // //
            // // egui::Grid::new("test_ui")
            // //     .striped(true)
            // //     .min_col_width(50.0)
            // //     .max_col_width(50.0)
            // //     .num_columns(2)
            // //     .show(ui, |ui| {
            // //         ui.label("1");
            // //         ui.label("2");
            // //         ui.end_row();
            // //         ui.label("1 asdf asdf asdf sadf sda f as f");
            // //         ui.label("2");
            // //         ui.end_row();
            // //         ui.label("1");
            // //         ui.label("2a sfsad fasd fasd fasdf asdf asddf asdf asd f");
            // //         ui.end_row();
            // //         ui.label("1");
            // //         ui.label("2");
            // //         ui.end_row();
            // //     });
            //
            //
            // ui.horizontal_top(|ui| {
            //     //ui.all
            //     ui.add_sized(egui::vec2(100.0, 30.0), egui::Label::new("test text").truncate(true));
            //     ui.label("some value");
            // });
            // ui.horizontal_top(|ui| {
            //     //ui.all
            //     ui.add_sized(egui::vec2(100.0, 30.0), egui::Label::new("test text that is very long").truncate(true));
            //     ui.label("some value");
            // });
            // // ui.horizontal(|ui| {
            // //     //ui.all
            // //     ui.add_sized(egui::vec2(100.0, 30.0), egui::CollapsingHeader::new("test").show(ui, |ui| {}));
            // //     ui.label("some value");
            // // });
            //
            // ui.horizontal_top(|ui| {
            //     //ui.all
            //     ui.add_sized(egui::vec2(100.0, 30.0), egui::Label::new("test text").truncate(true));
            //     ui.label("some value");
            // });
            //
            //
            //
            // let openness = egui::CollapsingHeader::new("test").show(ui, |ui| {
            // }).openness;
            //
            // if openness > 0.1 {
            //
            //     egui::Grid::new("test_u2")
            //         .striped(true)
            //         .min_col_width(5.0)
            //         .num_columns(2)
            //         .show(ui, |ui| {
            //             ui.label("    1");
            //             ui.label("2");
            //             ui.end_row();
            //             ui.label("    1 asdf asdf asdf sadf sda f as f");
            //             ui.label("2");
            //             ui.end_row();
            //             ui.label("    1");
            //             ui.label("2a sfsad fasd fasd fasdf asdf asddf asdf asd f");
            //             ui.end_row();
            //             ui.label("    1");
            //             ui.label("2");
            //             ui.end_row();
            //         });
            // }

            if let Some(asset_id) = asset_id
            {
                let edit_context = editor_model.root_edit_context();
                if !edit_context.has_asset(asset_id) {
                    return;
                }

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

                let is_generated = editor_model.is_generated_asset(asset_id);
                if is_generated {
                    ui.label(format!("This asset is generated from a source file and can't be modified unless it is persisted to disk. A new asset file will be created and source file changes will no longer affect it."));
                }

                if is_generated {
                    if ui.button("Persist Asset").clicked() {
                        action_sender.queue_action(UIAction::PersistAssets(vec![asset_id]));
                    }
                }

                if ui.button("Use as prototype").clicked() {
                    action_sender.try_set_modal_action(NewAssetModal::new_with_prototype(Some(editor_model.root_edit_context().asset_location(asset_id).unwrap()), asset_id))
                }

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

                if ui.button("Rebuild this Asset").clicked() {
                    //app_state.asset_engine.queue_build_operation(asset_id);
                    action_sender.queue_action(UIAction::ForceRebuild(vec![asset_id]));
                }

                ui.separator();

                let read_only = is_generated;

                // let mut table = egui_extras::TableBuilder::new(ui)
                //     .striped(true)
                //     .auto_shrink([true, false])
                //     .resizable(true)
                //     // vscroll and min/max scroll height make this table grow/shrink according to available size
                //     .vscroll(false)
                //     .min_scrolled_height(1.0)
                //     .max_scroll_height(1.0)
                //     .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                //     .column(egui_extras::Column::initial(200.0).at_least(5.0).at_most(available_x * 0.9).clip(true))
                //     .column(egui_extras::Column::remainder().at_least(5.0).at_most(available_x * 0.9).clip(true));
                //
                // table.body(|body| {
                //
                // });

                draw_inspector_property(
                    ui,
                    InspectorContext {
                        editor_model,
                        editor_model_ui_state,
                        action_sender,
                        asset_id,
                        property_name: "",
                        property_path: "",
                        schema: &Schema::Record(
                            editor_model.root_edit_context().data_set().asset_schema(asset_id).unwrap().fingerprint()
                        ),
                        read_only,
                    },
                )
            }
        });
}

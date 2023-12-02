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
            InspectorContext {
                editor_model: ctx.editor_model,
                action_sender: ctx.action_sender,
                asset_id: ctx.asset_id,
                property_path: ctx.property_path,
                property_name: ctx.property_name,
                schema: ctx.schema,
                read_only: ctx.read_only,
            },
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
                    ui.set_enabled(!ctx.read_only);
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
                        asset_ref.to_string()
                    };

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
            Schema::NamedType(schema_fingerprint) => {
                let schema = ctx
                    .editor_model
                    .schema_set()
                    .find_named_type_by_fingerprint(*schema_fingerprint)
                    .unwrap();
                match schema {
                    SchemaNamedType::Record(record_schema) => {
                        for field in record_schema.fields() {
                            let field_path = join_field_path(ctx.property_path, field.name());

                            draw_inspector_property(
                                ui,
                                InspectorContext {
                                    property_name: field.name(),
                                    property_path: &field_path,
                                    schema: field.field_schema(),
                                    ..ctx
                                },
                            );
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
    asset_id: AssetId,
) {
    let edit_context = editor_model.root_edit_context();
    if !edit_context.has_asset(asset_id) {
        return;
    }

    ui.label(format!("ID: {:?}", asset_id));

    let name = edit_context.asset_name(asset_id);
    let location = edit_context.asset_location(asset_id).unwrap();

    ui.label(format!(
        "Name: {}",
        name.unwrap().as_string().cloned().unwrap_or_default()
    ));
    let import_info = edit_context.import_info(asset_id);
    if let Some(import_info) = import_info {
        ui.label(format!(
            "Imported From: {}",
            import_info.source_file_path().to_string_lossy()
        ));
        ui.label(format!(
            "Importable Name: {:?}",
            import_info.importable_name().name()
        ));
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

    ui.label(format!(
        "Path Node: {}",
        editor_model
            .asset_display_name_long(location.path_node_id(), &editor_model_ui_state.path_lookup)
    ));

    if ui.button("Force Rebuild").clicked() {
        //app_state.asset_engine.queue_build_operation(asset_id);
        action_sender.queue_action(UIAction::ForceRebuild(vec![asset_id]));
    }

    if let Some(prototype) = edit_context.asset_prototype(asset_id) {
        ui.horizontal(|ui| {
            if ui.button(">>").clicked() {
                // let grid_state = &mut app_state.ui_state.asset_browser_state.grid_state;
                // grid_state.first_selected = Some(prototype);
                // grid_state.last_selected = Some(prototype);
                // grid_state.selected_items.clear();
                // grid_state.selected_items.insert(prototype);
                action_sender.queue_action(UIAction::ShowAssetInAssetGallery(asset_id));
            }

            let prototype_display_name =
                editor_model.asset_display_name_long(prototype, &editor_model_ui_state.path_lookup);

            ui.label(format!("Prototype: {}", prototype_display_name));
        });
    }

    let read_only = is_generated;
    draw_inspector_property(
        ui,
        InspectorContext {
            editor_model,
            action_sender,
            asset_id,
            property_name: "",
            property_path: "",
            schema: &Schema::NamedType(
                editor_model_ui_state
                    .all_asset_info
                    .get(&asset_id)
                    .unwrap()
                    .schema
                    .fingerprint(),
            ),
            read_only,
        },
    )
}

use std::sync::Arc;
use eframe::epaint::Color32;
use egui::{Response, TextStyle, Widget};
use hydrate_model::{AssetId, EditorModel, EndContextBehavior, HashSet, NullOverride, PropertyPath, Schema, SchemaNamedType, SchemaSet, Value};
use hydrate_model::value::ValueEnum;
use crate::action_queue::UIActionQueueSender;
use crate::ui::drag_drop::DragDropPayload;
use crate::ui_state::EditorModelUiState;

fn join_field_path(lhs: &str, rhs: &str) -> String {
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

fn simple_value_property<F: FnOnce(&mut egui::Ui, InspectorContext) -> (Option<Value>, egui::Response)>(
    ui: &mut egui::Ui,
    ctx: InspectorContext,
    f: F
) {
    let has_override = ctx.editor_model.root_edit_context().has_property_override(ctx.asset_id, ctx.property_path).unwrap();
    let assets_to_edit = vec![ctx.asset_id];

    //ui.style_mut().drag_value_text_style = TextStyle::Name(Arc::new("CustomStyle".to_string()))
    //ui.style_mut().visuals/. = Color32::from_rgb(255, 0, 0);
    ui.horizontal(|ui| {
        if has_override {
            ui.style_mut().visuals.override_text_color = Some(Color32::from_rgb(255, 255, 0));
        }
        egui::Label::new(ctx.property_name).ui(ui);

        let (new_value, response) = f(ui, InspectorContext {
            editor_model: ctx.editor_model,
            action_sender: ctx.action_sender,
            asset_id: ctx.asset_id,
            property_path: ctx.property_path,
            property_name: ctx.property_name,
            schema: ctx.schema,
            read_only: ctx.read_only,
        });

        let committed = response.lost_focus() || response.drag_released();
        if response.changed() || committed {
            let end_context_behavior = if committed {
                println!("finish");
                EndContextBehavior::Finish
            } else {
                println!("allow resume");
                EndContextBehavior::AllowResume
            };

            let property_path_moved = ctx.property_path.to_string();
            ctx.action_sender.queue_edit("property_editor", assets_to_edit, move |edit_context| {
                edit_context.set_property_override(ctx.asset_id, property_path_moved, new_value).unwrap();
                Ok(end_context_behavior)
            });
        }
    });
}

fn draw_inspector_property(
    ui: &mut egui::Ui,
    ctx: InspectorContext
) {
    match ctx.schema {
        Schema::Nullable(inner_schema) => {
            let null_override = ctx.editor_model.root_edit_context().get_null_override(ctx.asset_id, ctx.property_path).unwrap();
            let resolved_null_override = ctx.editor_model.root_edit_context().resolve_null_override(ctx.asset_id, ctx.property_path).unwrap();

            ui.horizontal(|ui| {
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
                    ctx.action_sender.queue_edit("property_editor", vec![ctx.asset_id], move |edit_context| {
                        edit_context.set_null_override(ctx.asset_id, captured_property_path, new_null_override)?;
                        Ok(EndContextBehavior::Finish)
                    });
                }
            });

            if resolved_null_override == NullOverride::SetNonNull {
                ui.indent(ctx.property_path, |ui| {
                    let field_path = join_field_path(ctx.property_path, "value");
                    draw_inspector_property(ui, InspectorContext {
                        property_name: "value",
                        property_path: &field_path,
                        schema: &*inner_schema,
                        ..ctx
                    })
                });
            }
        }
        Schema::Boolean => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_boolean().unwrap();
                    let response = egui::Checkbox::new(&mut value, "").ui(ui);
                    let new_value = Some(Value::Boolean(value));
                    (new_value, response)
                }
            )
        },
        Schema::I32 => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_i32().unwrap();
                    let response = egui::DragValue::new(&mut value).ui(ui);
                    let new_value = Some(Value::I32(value));
                    (new_value, response)
                }
            )
        },
        Schema::I64 => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_i64().unwrap();
                    let response = egui::DragValue::new(&mut value).ui(ui);
                    let new_value = Some(Value::I64(value));
                    (new_value, response)
                }
            )
        },
        Schema::U32 => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_u32().unwrap();
                    let response = egui::DragValue::new(&mut value).ui(ui);
                    let new_value = Some(Value::U32(value));
                    (new_value, response)
                }
            )
        },
        Schema::U64 => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_u64().unwrap();
                    let response = egui::DragValue::new(&mut value).ui(ui);
                    let new_value = Some(Value::U64(value));
                    (new_value, response)
                }
            )
        },
        Schema::F32 => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_f32().unwrap();
                    let response = egui::DragValue::new(&mut value).ui(ui);
                    let new_value = Some(Value::F32(value));
                    (new_value, response)
                }
            )
        },
        Schema::F64 => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_f64().unwrap();
                    let response = egui::DragValue::new(&mut value).ui(ui);
                    let new_value = Some(Value::F64(value));
                    (new_value, response)
                }
            )
        },
        Schema::Bytes => {
            ui.label(format!("{}: Unsupported Schema::Bytes Property", ctx.property_name));
        },
        Schema::String => {
            simple_value_property(
                ui,
                ctx,
                |ui, ctx| {
                    let mut value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap().as_string().unwrap().to_string();
                    let response = egui::TextEdit::singleline(&mut value).ui(ui);
                    let new_value = Some(Value::String(Arc::new(value)));
                    (new_value, response)
                }
            )
        }
        // Schema::StaticArray(_) => unimplemented!(),
        Schema::DynamicArray(schema) => {
            let resolved = ctx.editor_model.root_edit_context().resolve_dynamic_array(ctx.asset_id, ctx.property_path).unwrap();
            let overrides = ctx.editor_model.root_edit_context().get_dynamic_array_overrides(ctx.asset_id, ctx.property_path).unwrap();
            ui.push_id(ctx.property_path, |ui| {
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
                                }
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
                                }
                            )
                        });
                    }
                });
            });
        }
        // Schema::Map(_) => unimplemented!(),
        Schema::AssetRef(_) => {
            let resolved_value = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap();
            let has_override = ctx.editor_model.root_edit_context().has_property_override(ctx.asset_id, ctx.property_path).unwrap();


            let asset_ref = resolved_value.as_asset_ref().unwrap();

            ui.horizontal(|ui| {
                if has_override {
                    ui.style_mut().visuals.override_text_color = Some(Color32::from_rgb(255, 255, 0));
                }

                let mut label_string = if asset_ref.is_null() {
                    "not set".to_string()
                } else {
                    asset_ref.to_string()
                };
                ui.label(ctx.property_name);

                let can_accept_what_is_being_dragged = true;
                let response = crate::ui::drag_drop::drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                    ui.add_enabled_ui(false, |ui| {
                        ui.text_edit_singleline(&mut label_string);
                    })
                }).response;

                if let Some(payload) = crate::ui::drag_drop::try_take_dropped_payload(ui, &response) {
                    match payload {
                        DragDropPayload::AssetReference(payload_asset_id) => {
                            //println!("Dropped {:?} over {:?}", asset_id, ctx.property_path);
                            let captured_property_path = ctx.property_path.to_string();
                            let asset_id = ctx.asset_id;
                            ctx.action_sender.queue_edit("property_editor", vec![ctx.asset_id], move |edit_context| {
                                edit_context.set_property_override(asset_id, captured_property_path, Some(Value::AssetRef(payload_asset_id))).unwrap();
                                Ok(EndContextBehavior::Finish)
                            });
                        },
                        _ => unimplemented!()
                    }
                }
            });

        }
        Schema::NamedType(schema_fingerprint) => {
            let schema = ctx.editor_model.schema_set().find_named_type_by_fingerprint(*schema_fingerprint).unwrap();
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
                            }
                        );
                    }
                }
                SchemaNamedType::Enum(enum_schema) => {
                    let resolved = ctx.editor_model.root_edit_context().resolve_property(ctx.asset_id, ctx.property_path).unwrap();
                    let has_override = ctx.editor_model.root_edit_context().has_property_override(ctx.asset_id, ctx.property_path).unwrap();
                    let old_symbol_name = resolved.as_enum().unwrap().symbol_name().to_string();
                    let mut selected_symbol_name = old_symbol_name.clone();
                    let asset_id = ctx.asset_id;

                    ui.horizontal(|ui| {
                        if has_override {
                            ui.style_mut().visuals.override_text_color = Some(Color32::from_rgb(255, 255, 0));
                        }

                        ui.label(ctx.property_name);

                        let response = egui::ComboBox::new(ctx.property_path, "")
                            .selected_text(&selected_symbol_name)
                            .show_ui(ui, |ui| {
                                for symbol in enum_schema.symbols() {
                                    ui.selectable_value(&mut selected_symbol_name, symbol.name().to_string(), symbol.name());
                                }
                            }).response;

                        if old_symbol_name != selected_symbol_name {
                            let new_value = Value::Enum(ValueEnum::new(selected_symbol_name));
                            let captured_property_path = ctx.property_path.to_string();
                            ctx.action_sender.queue_edit("property_editor", vec![asset_id], move |edit_context| {
                                edit_context.set_property_override(asset_id, captured_property_path, Some(new_value)).unwrap();
                                Ok(EndContextBehavior::Finish)
                            });
                        }
                    });
                }
                // SchemaNamedType::Fixed(_) => unimplemented!(),
                _ => { ui.label(format!("unimplemented {:?} {}", schema, ctx.property_name)); },
            }
        }
        _ => { ui.label(format!("unimplemented {:?} {}", ctx.schema, ctx.property_name)); },
    }
}

pub fn draw_inspector(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    asset_id: AssetId,
) {
    draw_inspector_property(
        ui,
        InspectorContext {
            editor_model,
            action_sender,
            asset_id,
            property_name: "",
            property_path: "",
            schema: &Schema::NamedType(editor_model_ui_state.all_asset_info.get(&asset_id).unwrap().schema.fingerprint()),
            read_only: false,
        }
    )
}


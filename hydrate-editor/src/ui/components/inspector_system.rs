use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui_state::EditorModelUiState;
use eframe::epaint::Color32;
use egui::{Response, Widget, WidgetText};
use hydrate_model::value::ValueEnum;
use hydrate_model::{AssetId, EditorModel, EndContextBehavior, HashMap, NullOverride, PropertyPath, Schema, SchemaFingerprint, SchemaNamedType, SchemaRecord, SchemaSet, Value};
use std::sync::Arc;

pub fn show_property_context_menu(
    ctx: InspectorContext,
    response: Response,
) -> Response {
    let asset_id = ctx.asset_id;
    response.context_menu(|ui| {
        let has_override = ctx
            .editor_model
            .root_edit_context()
            .has_property_override(asset_id, ctx.property_path.path())
            .unwrap();
        if ui
            .add_enabled(
                has_override && !ctx.read_only,
                egui::Button::new("Clear Override"),
            )
            .clicked()
        {
            ctx.action_sender.queue_action(UIAction::SetProperty(asset_id,  ctx.property_path.clone(), None, EndContextBehavior::Finish));
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
            ctx.action_sender.queue_action(UIAction::ApplyPropertyOverrideToPrototype(asset_id, ctx.property_path.clone()));
            ui.close_menu();
        }
    })
}

fn add_empty_collapsing_header(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> bool {
    let openness = egui::CollapsingHeader::new(text).show_unindented(ui, |ui| {}).openness;
    openness > 0.5
}

#[derive(Copy, Clone)]
pub struct InspectorContext<'a> {
    pub editor_model: &'a EditorModel,
    pub editor_model_ui_state: &'a EditorModelUiState,
    pub action_sender: &'a UIActionQueueSender,
    pub asset_id: AssetId,
    pub property_path: &'a PropertyPath,
    pub property_name: &'a str,
    pub schema: &'a hydrate_model::Schema,
    pub inspector_registry: &'a InspectorRegistry,
    pub read_only: bool,
}


//Override AssetRef to show images or other preview info
// - Actually we can just always show thumbnail?
//Override array items to have extra buttons or a friendly title when collapsed?
// - Do I just do this for records?
//Override Records to show X/Y/Z on same line or a matrix in a more square form
//Change if we want a value to be a slider, text entry, etc?
// what about colors vs. position vectors etc.?
//Maybe I make a single code implementation that is data driven?
pub trait RecordInspector {
    fn can_draw_as_single_value(&self) -> bool {
        false
    }

    fn draw_inspector_value(
        &self,
        ui: &mut egui::Ui,
        ctx: InspectorContext,
    ) {
        unimplemented!()
    }

    fn draw_inspector_rows(
        &self,
        table_body: &mut egui_extras::TableBody,
        ctx: InspectorContext,
        record: &SchemaRecord,
        indent_level: u32,
    );
}

#[derive(Default)]
struct DefaultRecordInspector;
impl RecordInspector for DefaultRecordInspector {
    fn draw_inspector_rows(
        &self,
        table_body: &mut egui_extras::TableBody,
        ctx: InspectorContext,
        record: &SchemaRecord,
        indent_level: u32
    ) {
        //
        // Draw the fields
        //
        for field in record.fields() {
            let field_path = ctx.property_path.push(field.name());
            let ctx = InspectorContext {
                property_name: field.name(),
                property_path: &field_path,
                schema: field.field_schema(),
                ..ctx
            };
            draw_inspector_rows(table_body, ctx, indent_level);
        }
    }
}


#[derive(Default)]
pub struct InspectorRegistry {
    overrides: HashMap<SchemaFingerprint, Box<dyn RecordInspector>>,
    default: DefaultRecordInspector,
}

impl InspectorRegistry {
    pub fn get_override(&self, fingerprint: SchemaFingerprint) -> &dyn RecordInspector {
        if let Some(inspector_override) = self.overrides.get(&fingerprint) {
            &**inspector_override
        } else {
            &self.default
        }
    }

    pub fn register_override(&mut self, fingerprint: SchemaFingerprint, inspector_impl: impl RecordInspector + 'static) {
        let old = self.overrides.insert(fingerprint, Box::new(inspector_impl));
        assert!(old.is_none());
    }
}

fn set_override_text_color_for_has_override_status(ctx: InspectorContext, ui: &mut egui::Ui) {
    let has_override = ctx
        .editor_model
        .root_edit_context()
        .has_property_override(ctx.asset_id, ctx.property_path.path())
        .unwrap();

    if !has_override {
        ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(150));
    } else {
        ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(255));
    }
}

pub fn simple_value_property<
    F: FnOnce(&mut egui::Ui, InspectorContext) -> Option<(Value, EndContextBehavior)>,
>(
    ui: &mut egui::Ui,
    ctx: InspectorContext,
    f: F,
) {
    ui.horizontal(|ui| {
        ui.set_enabled(!ctx.read_only);
        set_override_text_color_for_has_override_status(ctx, ui);

        if let Some((new_value, end_context_behavior)) = f(
            ui,
            ctx,
        ) {
            ctx.action_sender.queue_action(UIAction::SetProperty(
                ctx.asset_id,
                ctx.property_path.clone(),
                Some(new_value),
                end_context_behavior
            ));
        }
    });
}

//
// These handle the quirks of how a UI control is manipulated and when we decide to "commit" an undo step
//
fn end_context_behavior_for_drag_value(
    response: egui::Response,
) -> EndContextBehavior {
    if response.lost_focus() || response.drag_released() {
        EndContextBehavior::Finish
    } else {
        EndContextBehavior::AllowResume
    }
}

fn end_context_behavior_for_text_field(
    response: egui::Response,
) -> EndContextBehavior {
    if response.lost_focus() || response.drag_released() {
        EndContextBehavior::Finish
    } else {
        EndContextBehavior::AllowResume
    }
}

pub fn draw_indented_label(ui: &mut egui::Ui, indent_level: u32, text: impl Into<WidgetText>) -> Response {
    for _ in 0..indent_level {
        crate::ui::add_indent_spacing(ui);
    }
    crate::ui::add_icon_spacing(ui);
    ui.label(text)
}

pub fn draw_indented_collapsible_label(ui: &mut egui::Ui, indent_level: u32, text: impl Into<WidgetText>) -> bool {
    for _ in 0..indent_level {
        crate::ui::add_indent_spacing(ui);
    }
    add_empty_collapsing_header(ui, text)
}

pub fn draw_basic_inspector_row<F: FnOnce(&mut egui::Ui, InspectorContext)>(
    body: &mut egui_extras::TableBody,
    ctx: InspectorContext,
    indent_level: u32,
    f: F
) {
    body.row(20.0, |mut row| {
        row.col(|mut ui| {
            let label_response = draw_indented_label(ui, indent_level, ctx.property_name);
            show_property_context_menu(ctx, label_response);
        });
        row.col(|mut ui| {
            ui.push_id(ctx.property_path.path(), |ui| {
                f(ui, ctx);
            });
        });
    });
}

fn can_draw_as_single_value(schema: &Schema, inspector_registry: &InspectorRegistry) -> bool {
    match schema {
        Schema::Boolean => true,
        Schema::I32 => true,
        Schema::I64 => true,
        Schema::U32 => true,
        Schema::U64 => true,
        Schema::F32 => true,
        Schema::F64 => true,
        Schema::Bytes => true,
        Schema::String => true,
        Schema::AssetRef(_) => true,
        Schema::Enum(_) => true,
        Schema::Record(fingerprint) => {
            inspector_registry.get_override(*fingerprint).can_draw_as_single_value()
        }
        _ => false,
    }
}

pub fn draw_inspector_value(
    ui: &mut egui::Ui,
    ctx: InspectorContext,
) {
    match ctx.schema {
        Schema::Boolean => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_boolean()
                .unwrap();
            let response = egui::Checkbox::new(&mut value, "").ui(ui);
            if response.changed() {
                Some((Value::Boolean(value), EndContextBehavior::Finish))
            } else {
                None
            }
        }),
        Schema::I32 => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_i32()
                .unwrap();
            let response = egui::DragValue::new(&mut value).ui(ui);
            if response.changed() {
                Some((Value::I32(value), end_context_behavior_for_drag_value(response)))
            } else {
                None
            }
        }),
        Schema::I64 => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_i64()
                .unwrap();
            let response = egui::DragValue::new(&mut value).ui(ui);
            if response.changed() {
                Some((Value::I64(value), end_context_behavior_for_drag_value(response)))
            } else {
                None
            }
        }),
        Schema::U32 => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_u32()
                .unwrap();
            let response = egui::DragValue::new(&mut value).ui(ui);
            if response.changed() {
                Some((Value::U32(value), end_context_behavior_for_drag_value(response)))
            } else {
                None
            }
        }),
        Schema::U64 => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_u64()
                .unwrap();
            let response = egui::DragValue::new(&mut value).ui(ui);
            if response.changed() {
                Some((Value::U64(value), end_context_behavior_for_drag_value(response)))
            } else {
                None
            }
        }),
        Schema::F32 => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_f32()
                .unwrap();
            let response = egui::DragValue::new(&mut value).ui(ui);
            if response.changed() {
                Some((Value::F32(value), end_context_behavior_for_drag_value(response)))
            } else {
                None
            }
        }),
        Schema::F64 => simple_value_property(ui, ctx, |ui, ctx| {
            let mut value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_f64()
                .unwrap();
            let response = egui::DragValue::new(&mut value).ui(ui);
            if response.changed() {
                Some((Value::F64(value), end_context_behavior_for_drag_value(response)))
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
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap()
                .as_string()
                .unwrap()
                .to_string();
            let response = egui::TextEdit::singleline(&mut value).desired_width(ui.available_width()).ui(ui);
            if response.changed() {
                Some((Value::String(Arc::new(value)), end_context_behavior_for_text_field(response)))
            } else {
                None
            }
        }),
        Schema::AssetRef(_) => {
            let resolved_value = ctx
                .editor_model
                .root_edit_context()
                .resolve_property(ctx.asset_id, ctx.property_path.path())
                .unwrap();

            let asset_ref = resolved_value.as_asset_ref().unwrap();

            ui.horizontal(|ui| {
                set_override_text_color_for_has_override_status(ctx, ui);

                // The GO TO ASSET button
                if ui.add_enabled(!asset_ref.is_null(), egui::Button::new(">>")).clicked() {
                    ctx.action_sender.queue_action(UIAction::ShowAssetInAssetGallery(asset_ref));
                }

                // Set enabled after the "go to" button
                ui.set_enabled(!ctx.read_only);

                // Draw the text field and enable it as a drop target
                let can_accept_what_is_being_dragged = !ctx.read_only;
                let response = crate::ui::drag_drop::drop_target(
                    ui,
                    can_accept_what_is_being_dragged,
                    |ui| {
                        ui.add_enabled_ui(false, |ui| {
                            let mut label_string = if asset_ref.is_null() {
                                "not set".to_string()
                            } else {
                                let asset_path = ctx.editor_model.asset_path(asset_ref, &ctx.editor_model_ui_state.asset_path_cache);
                                asset_path.as_str().to_string()
                            };

                            ui.add(egui::TextEdit::singleline(&mut label_string).desired_width(ui.available_width() - 30.0));
                        })
                    },
                ).response;

                // Handle dropping an asset onto the text field
                if let Some(payload) =
                    crate::ui::drag_drop::try_take_dropped_payload(ui, &response)
                {
                    match payload {
                        DragDropPayload::AssetReference(payload_asset_id) => {
                            ctx.action_sender.queue_action(UIAction::SetProperty(
                                ctx.asset_id,
                                ctx.property_path.clone(),
                                Some(Value::AssetRef(payload_asset_id)),
                                EndContextBehavior::Finish
                            ));
                        }
                        _ => log::error!("Payload type not expected when dropping onto a asset reference text field"),
                    }
                }

                // Button to clear the asset ref field
                if ui.add_enabled(!asset_ref.is_null(), egui::Button::new("X")).clicked() {
                    ctx.action_sender.queue_action(UIAction::SetProperty(ctx.asset_id, ctx.property_path.clone(), None, EndContextBehavior::Finish));
                }
            });
        }
        Schema::Enum(schema_fingerprint) => {
            let schema = ctx
                .editor_model
                .schema_set()
                .find_named_type_by_fingerprint(*schema_fingerprint)
                .unwrap();
            match schema {
                SchemaNamedType::Record(record_schema) => panic!("An enum schema is referencing a record"),
                SchemaNamedType::Enum(enum_schema) => {
                    //ui.push_id(ctx.property_path, |ui| {
                    let resolved = ctx
                        .editor_model
                        .root_edit_context()
                        .resolve_property(ctx.asset_id, ctx.property_path.path())
                        .unwrap();

                    let old_symbol_name = resolved.as_enum().unwrap().symbol_name().to_string();
                    let mut selected_symbol_name = old_symbol_name.clone();
                    let asset_id = ctx.asset_id;

                    ui.horizontal(|ui| {
                        ui.set_enabled(!ctx.read_only);
                        set_override_text_color_for_has_override_status(ctx, ui);

                        let response = egui::ComboBox::new(ctx.property_path.path(), "")
                            .selected_text(&selected_symbol_name)
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for symbol in enum_schema.symbols() {
                                    ui.selectable_value(
                                        &mut selected_symbol_name,
                                        symbol.name().to_string(),
                                        symbol.name(),
                                    );
                                }
                            });

                        if old_symbol_name != selected_symbol_name {
                            let new_value = Value::Enum(ValueEnum::new(selected_symbol_name));
                            ctx.action_sender.queue_action(UIAction::SetProperty(
                                asset_id,
                                ctx.property_path.clone(),
                                Some(new_value),
                                EndContextBehavior::Finish)
                            );
                        }
                    });
                }
            }
        },
        Schema::Record(schema_fingerprint) => {
            let inspector_impl = ctx.inspector_registry.get_override(*schema_fingerprint);
            if !inspector_impl.can_draw_as_single_value() {
                ui.label("SCHEMA ERROR: Inspector can't draw as single value");
            } else {
                // find the record?
                let record = ctx.editor_model.schema_set().find_named_type_by_fingerprint(*schema_fingerprint);
                if let Some(record) = record {
                    match record {
                        SchemaNamedType::Record(record) => inspector_impl.draw_inspector_value(ui, ctx),
                        _ => { ui.label("SCHEMA ERROR: Type referenced by Schema::Record is not a record"); }
                    }
                } else {
                    ui.label("SCHEMA ERROR: Type not found");
                }
            }
        },
        _ => { ui.label(format!("Schema {:?} cannot be drawn as a single value", ctx.schema)); }
    }
}

pub fn draw_inspector_rows(
    body: &mut egui_extras::TableBody,
    ctx: InspectorContext,
    indent_level: u32,
) {
    match ctx.schema {
        Schema::Nullable(inner_schema) => {
            let null_override = ctx
                .editor_model
                .root_edit_context()
                .get_null_override(ctx.asset_id, ctx.property_path.path())
                .unwrap();
            let resolved_null_override = ctx
                .editor_model
                .root_edit_context()
                .resolve_null_override(ctx.asset_id, ctx.property_path.path())
                .unwrap();

            let mut is_visible = false;

            body.row(20.0, |mut row| {
                row.col(|ui| {
                    ui.push_id(format!("{} inspector_label_column", ctx.property_path.path()), |ui| {
                        if resolved_null_override == NullOverride::SetNonNull {
                            is_visible = draw_indented_collapsible_label(ui, indent_level, ctx.property_name)
                        } else {
                            draw_indented_label(ui, indent_level, ctx.property_name);
                        }
                    });
                });
                row.col(|ui| {
                    ui.push_id(format!("{} inspector_value_column", ctx.property_path.path()), |ui| {
                        ui.set_enabled(!ctx.read_only);
                        if null_override == NullOverride::Unset {
                            ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(150));
                        } else {
                            ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(255));
                        }

                        let mut new_null_override = None;
                        if ui.selectable_label(resolved_null_override == NullOverride::Unset, "Inherit").clicked() {
                            new_null_override = Some(NullOverride::Unset);
                        }
                        if ui.selectable_label(resolved_null_override == NullOverride::SetNull, "No Value").clicked() {
                            new_null_override = Some(NullOverride::SetNull);
                        }
                        if ui.selectable_label(resolved_null_override == NullOverride::SetNonNull, "Has Value").clicked() {
                            new_null_override = Some(NullOverride::SetNonNull);
                        }

                        if let Some(new_null_override) = new_null_override {
                            ctx.action_sender.queue_action(UIAction::SetNullOverride(
                                ctx.asset_id,
                                ctx.property_path.clone(),
                                new_null_override
                            ));

                        }
                    });
                });
            });
            if is_visible {
                if resolved_null_override == NullOverride::SetNonNull {
                    let field_path = ctx.property_path.push("value");
                    draw_inspector_rows(
                        body,
                        InspectorContext {
                            property_name: "value",
                            property_path: &field_path,
                            schema: &*inner_schema,
                            ..ctx
                        },
                        indent_level + 1
                    );
                }
            }
        }
        Schema::Boolean => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::I32 => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::I64 => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::U32 => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::U64 => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::F32 => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::F64 => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        Schema::Bytes => {
            draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                draw_inspector_value(ui, ctx);
            });
        }
        Schema::String => draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
            draw_inspector_value(ui, ctx);
        }),
        //TODO: Implement static array
        Schema::StaticArray(_) => {
            draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                ui.label(format!(
                    "unimplemented {:?} {}",
                    ctx.schema, ctx.property_name
                ));
            });
        },
        Schema::DynamicArray(schema) => {
            let resolved = ctx
                .editor_model
                .root_edit_context()
                .resolve_dynamic_array(ctx.asset_id, ctx.property_path.path())
                .unwrap();
            let overrides = ctx
                .editor_model
                .root_edit_context()
                .get_dynamic_array_overrides(ctx.asset_id, ctx.property_path.path())
                .unwrap();
            let mut is_visible = false;

            body.row(20.0, |mut row| {
                row.col(|ui| {
                    ui.push_id(format!("{} inspector_label_column", ctx.property_path.path()), |ui| {
                        for i in 0..indent_level {
                            crate::ui::add_indent_spacing(ui);
                        }

                        is_visible = add_empty_collapsing_header(ui, ctx.property_name)
                    });
                });
                row.col(|ui| {
                    ui.push_id(format!("{} inspector_value_column", ctx.property_path.path()), |ui| {
                        ui.set_enabled(!ctx.read_only);

                        if ui.button("+").clicked() {
                            ctx.action_sender.queue_action(UIAction::AddDynamicArrayOverride(ctx.asset_id, ctx.property_path.clone()));
                        }

                        // if overrides.is_empty() {
                        //     ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(150));
                        // } else {
                        //     ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(255));
                        // }

                        // button to add elements?
                    });
                });
            });

            let can_use_inline_values = can_draw_as_single_value(schema.item_type(), ctx.inspector_registry);

            if is_visible {
                let mut override_index = 0;
                for id in &resolved[0..(resolved.len() - overrides.len())] {
                    let id_as_string = id.to_string();
                    let field_path = ctx.property_path.push(&id_as_string);
                    let label = format!("[{}] (inherited)", override_index);

                    let mut is_override_visible = false;
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.push_id(format!("{} inspector_label_column", id), |ui| {
                                for i in 0..(indent_level + 1) {
                                    crate::ui::add_indent_spacing(ui);
                                }

                                if can_use_inline_values {
                                    crate::ui::add_icon_spacing(ui);
                                    ui.label(label);
                                } else {
                                    is_override_visible = add_empty_collapsing_header(ui, label)
                                }

                            });
                        });
                        row.col(|ui| {
                            //TODO: Could do basic values in here...
                            if can_use_inline_values {
                                draw_inspector_value(ui, InspectorContext {
                                    property_name: &id_as_string,
                                    property_path: &field_path,
                                    schema: schema.item_type(),
                                    read_only: true,
                                    ..ctx
                                });
                            }
                        });
                    });

                    if !can_use_inline_values && is_override_visible {
                        draw_inspector_rows(
                            body,
                            InspectorContext {
                                property_name: &id_as_string,
                                property_path: &field_path,
                                schema: schema.item_type(),
                                read_only: true,
                                ..ctx
                            },
                            indent_level + 2
                        );
                    }

                    override_index += 1;
                }

                for id in overrides {
                    let id_as_string = id.to_string();
                    let field_path = ctx.property_path.push(&id_as_string);
                    let label = format!("[{}]", override_index);

                    let mut is_override_visible = false;
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.push_id(format!("{} inspector_label_column", id), |ui| {
                                for i in 0..(indent_level + 1) {
                                    crate::ui::add_indent_spacing(ui);
                                }
                                // Up button?
                                // Down button?
                                // Delete button?

                                if can_use_inline_values {
                                    crate::ui::add_icon_spacing(ui);
                                    ui.label(label);
                                } else {
                                    is_override_visible = add_empty_collapsing_header(ui, label)
                                }

                                ui.button("⬆");
                                ui.button("⬇");
                                ui.button("🗑");
                            });
                        });
                        row.col(|ui| {
                            if can_use_inline_values {
                                draw_inspector_value(ui, InspectorContext {
                                    property_name: &id_as_string,
                                    property_path: &field_path,
                                    schema: schema.item_type(),
                                    ..ctx
                                });
                            }
                        });
                    });

                    if !can_use_inline_values && is_override_visible {
                        draw_inspector_rows(
                            body,
                            InspectorContext {
                                property_name: &id_as_string,
                                property_path: &field_path,
                                schema: schema.item_type(),
                                ..ctx
                            },
                            indent_level + 2
                        );
                    }

                    override_index += 1;
                }
            }
        }
        //TODO: Implement map
        Schema::Map(_) => {
            draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                ui.label(format!(
                    "unimplemented {:?} {}",
                    ctx.schema, ctx.property_name
                ));
            });
        }

        Schema::AssetRef(_) => {
            draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                draw_inspector_value(ui, ctx);
            });
        }
        // We don't support drawing records as simple values. This function draws into a single cell of a table
        // and a record needs to add rows to the table. Maybe later we could rewrite this function to handle records
        // earlier by drawing multiple rows early in the function
        Schema::Record(schema_fingerprint) => {
            let inspector_impl = ctx.inspector_registry.get_override(*schema_fingerprint);
            // find the record?
            let record = ctx.editor_model.schema_set().find_named_type_by_fingerprint(*schema_fingerprint);
            if let Some(record) = record {
                match record {
                    SchemaNamedType::Record(record) => inspector_impl.draw_inspector_rows(body, ctx, record, indent_level),
                    _ => {
                        draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                            ui.label("SCHEMA ERROR: Type referenced by Schema::Record is not a record");
                        });
                    }
                }
            } else {
                draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                    ui.label("SCHEMA ERROR: Type not found");
                });
            }
        }
        Schema::Enum(schema_fingerprint) => {
            draw_basic_inspector_row(body, ctx, indent_level, |ui, ctx| {
                let schema = ctx
                    .editor_model
                    .schema_set()
                    .find_named_type_by_fingerprint(*schema_fingerprint)
                    .unwrap();
                match schema {
                    SchemaNamedType::Record(record_schema) => panic!("An enum schema is referencing a record"),
                    SchemaNamedType::Enum(enum_schema) => {
                        draw_inspector_value(ui, ctx);
                    }
                }
            });
        }
    }
}

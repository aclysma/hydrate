use crate::app_state::AppState;
use crate::ui::asset_browser_grid_drag_drop::AssetBrowserGridPayload;
use crate::ui_state::UiState;
use hydrate_model::edit_context::EditContext;
use hydrate_model::{EndContextBehavior, Schema};
use imgui::im_str;

fn draw_property_style<F: FnOnce(&imgui::Ui)>(
    ui: &imgui::Ui,
    property_inherited: bool,
    property_overridden: bool,
    f: F,
) {
    let _inherited_style_token = if property_inherited {
        Some(ui.push_style_color(imgui::StyleColor::Text, [0.2, 0.2, 0.2, 1.0]))
    } else {
        None
    };

    let _overridden_style_token = if property_overridden {
        Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
    } else {
        None
    };

    (f)(ui);
}

fn draw_inspector_simple_property<
    F: FnOnce(&imgui::Ui, &hydrate_model::Value) -> Option<hydrate_model::Value>,
>(
    ui: &imgui::Ui,
    _ui_state: &UiState,
    edit_context: &mut EditContext,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    _property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    f: F,
) {
    let v = if property_inherited {
        if let Some(value) = edit_context.resolve_property(object_id, &property_path) {
            value
        } else {
            edit_context.schema_set().default_value_for_schema(schema)
            //Value::default_for_schema(schema).clone()
        }
    } else {
        edit_context
            .get_property_override(object_id, &property_path)
            .unwrap()
    };

    let new_value = (f)(ui, v);

    unsafe {
        if imgui::sys::igBeginPopupContextItem(
            im_str!("popup").as_ptr(), /*std::ptr::null()*/
            imgui::sys::ImGuiPopupFlags_MouseButtonRight as _,
        ) {
            if imgui::MenuItem::new(im_str!("Clear Override")).build(ui) {
                edit_context.remove_property_override(object_id, &property_path);
            }

            if imgui::MenuItem::new(im_str!("Apply Override")).build(ui) {
                edit_context.apply_property_override_to_prototype(object_id, &property_path);
            }

            imgui::sys::igEndPopup();
        }
    }

    if let Some(new_value) = new_value {
        edit_context.set_property_override(object_id, &property_path, new_value);
    }
}

fn draw_inspector_simple_property_bool(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_boolean().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = ui.checkbox(&property_im_str, &mut v);

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::Boolean(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_i32(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_i32().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);

            if ui.is_item_active() {
                //println!("active");
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::I32(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_u32(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_u32().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::U32(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_i64(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_i64().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::I64(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_u64(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_u64().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::U64(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_f32(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_f32().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::F32(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_f64(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_f64().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::F64(v))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_string(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let v = value.as_string().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let mut value = im_str!("{}", &v);
            let modified = imgui::InputText::new(ui, &property_im_str, &mut value)
                .resize_buffer(true)
                .build();

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::String(value.to_string()))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_object_ref(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    _is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let v = value.as_object_ref().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let mut value = im_str!("{}", v.as_uuid());
            imgui::InputText::new(ui, &property_im_str, &mut value)
                .read_only(true)
                .build();

            if let Some(payload) =
                crate::ui::asset_browser_grid_drag_drop::asset_browser_grid_objects_drag_target_printf(
                    ui,
                    &ui_state.asset_browser_state.grid_state,
                )
            {
                match payload {
                    AssetBrowserGridPayload::Single(object_id) => {
                        *is_editing_complete = true;
                        Some(Value::ObjectRef(object_id))
                    }
                    AssetBrowserGridPayload::AllSelected => None,
                }
            } else {
                None
            }

            // if modified {
            //     Some(Value::String(value.to_string()))
            // } else {
            //     None
            // }
        },
    )

    // draw_inspector_simple_property(
    //     ui,
    //     edit_context,
    //     object_id,
    //     property_path,
    //     property_name,
    //     schema,
    //     property_inherited,
    //     |ui, value| {
    //         let mut v = value.as_f32().unwrap();
    //         let property_im_str = im_str!("{}", &property_name);
    //         let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);
    //         if modified {
    //             Some(Value::F32(v))
    //         } else {
    //             None
    //         }
    //     },
    // )
}

fn draw_inspector_nexdb_property(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: hydrate_model::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
) {
    use hydrate_model::*;

    match schema {
        Schema::Nullable(inner_schema) => {
            let property_inherited = !edit_context
                .get_null_override(object_id, &property_path)
                .is_some();
            let mut is_nulled = edit_context
                .resolve_is_null(object_id, &property_path)
                .unwrap_or(true);

            if imgui::CollapsingHeader::new(&im_str!("{}", property_name)).build(ui) {
                draw_property_style(ui, property_inherited, false, |ui| {
                    ui.text(property_path);
                });

                ui.indent();

                if is_nulled {
                    if ui.button(im_str!("Set Non-Null")) {
                        edit_context.set_null_override(
                            object_id,
                            property_path,
                            NullOverride::SetNonNull,
                        );
                        is_nulled = false;
                    }
                } else {
                    if ui.button(im_str!("Set Null")) {
                        edit_context.set_null_override(
                            object_id,
                            property_path,
                            NullOverride::SetNull,
                        );
                        is_nulled = true;
                    }
                }

                ui.same_line();
                if ui.button(im_str!("Inherit Null Status")) {
                    edit_context.remove_null_override(object_id, property_path);
                }

                if !is_nulled {
                    ui.indent();

                    let inner_property_path = if property_path.is_empty() {
                        "value".to_string()
                    } else {
                        format!("{}.value", property_path)
                    };

                    let id_token = ui.push_id(&inner_property_path);
                    draw_inspector_nexdb_property(
                        ui,
                        ui_state,
                        edit_context,
                        is_editing,
                        is_editing_complete,
                        object_id,
                        &inner_property_path,
                        "value",
                        &*inner_schema,
                    );
                    id_token.pop();
                    ui.unindent();
                }
                ui.unindent();
            }
        }
        Schema::Boolean => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_bool(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::I32 => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_i32(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::I64 => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_i64(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::U32 => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_u32(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::U64 => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_u64(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::F32 => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_f32(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::F64 => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_f64(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::Bytes => {}
        Schema::Buffer => {}
        Schema::String => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_string(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::StaticArray(_) => {}
        Schema::DynamicArray(array) => {
            let resolve = edit_context.resolve_dynamic_array(object_id, &property_path);
            let overrides: Vec<_> = edit_context
                .get_dynamic_array_overrides(object_id, &property_path)
                .map(|x| x.cloned().collect())
                .unwrap_or_default();

            ui.text(im_str!("{}", property_name));
            if imgui::CollapsingHeader::new(&im_str!("elements")).build(ui) {
                ui.indent();
                for id in &resolve[0..(resolve.len() - overrides.len())] {
                    // inherited
                    let field_path = format!("{}.{}", property_path, id);
                    let id_token = ui.push_id(&field_path);
                    draw_inspector_nexdb_property(
                        ui,
                        ui_state,
                        edit_context,
                        is_editing,
                        is_editing_complete,
                        object_id,
                        &field_path,
                        &id.to_string(),
                        array.item_type(),
                    );
                    id_token.pop();
                }

                for id in overrides {
                    let field_path = format!("{}.{}", property_path, id);
                    let id_token = ui.push_id(&field_path);
                    draw_inspector_nexdb_property(
                        ui,
                        ui_state,
                        edit_context,
                        is_editing,
                        is_editing_complete,
                        object_id,
                        &field_path,
                        &id.to_string(),
                        array.item_type(),
                    );
                    id_token.pop();
                }
                ui.unindent();
            }
        }
        Schema::Map(_) => {}
        //Schema::RecordRef(_) => {}
        Schema::ObjectRef(_named_type_fingerprint) => {
            let property_inherited = !edit_context.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_object_ref(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    object_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                );
            });
        }
        Schema::NamedType(named_type_fingerprint) => {
            let named_type = edit_context
                .find_named_type_by_fingerprint(*named_type_fingerprint)
                .unwrap()
                .clone();
            match named_type {
                SchemaNamedType::Record(record) => {
                    if property_path.is_empty()
                        || imgui::CollapsingHeader::new(&im_str!("{}", property_name)).build(ui)
                    {
                        // Don't indent if we are at root level
                        if !property_path.is_empty() {
                            ui.indent();
                        }

                        for field in record.fields() {
                            let field_path = if !property_path.is_empty() {
                                format!("{}.{}", property_path, field.name())
                            } else {
                                field.name().to_string()
                            };

                            let id_token = ui.push_id(field.name());
                            draw_inspector_nexdb_property(
                                ui,
                                ui_state,
                                edit_context,
                                is_editing,
                                is_editing_complete,
                                object_id,
                                &field_path,
                                field.name(),
                                field.field_schema(),
                            );
                            id_token.pop();
                        }

                        // Don't indent if we are at root level
                        if !property_path.is_empty() {
                            ui.unindent();
                        }
                    }
                }
                SchemaNamedType::Enum(_) => {}
                SchemaNamedType::Fixed(_) => {}
            }
        } // Schema::Record(record) => {
          //     if property_path.is_empty() || imgui::CollapsingHeader::new(&im_str!("{}", property_name)).build(ui) {
          //         ui.indent();
          //         for field in record.fields() {
          //             let field_path = if !property_path.is_empty() {
          //                 format!("{}.{}", property_path, field.name())
          //             } else {
          //                 field.name().to_string()
          //             };
          //
          //             let id_token = ui.push_id(field.name());
          //             draw_inspector_nexdb_property(ui, edit_context, object_id, &field_path, field.name(), field.field_schema());
          //             id_token.pop();
          //         }
          //         ui.unindent();
          //     }
          // }
          // Schema::Enum(_) => {
          //
          // }
          // Schema::Fixed(_) => {
          //
          // }
    }
}

pub fn draw_inspector_nexdb(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    object_id: hydrate_model::ObjectId,
) {
    let ui_state = &mut app_state.ui_state;
    app_state
        .db_state
        .editor_model
        .root_edit_context_mut()
        .with_undo_context("PropertyInspector", |edit_context| {
            let schema = edit_context.object_schema(object_id).clone();
            let mut is_editing = false;
            let mut is_editing_complete = false;

            if let Some(schema) = schema {
                draw_inspector_nexdb_property(
                    ui,
                    ui_state,
                    edit_context,
                    &mut is_editing,
                    &mut is_editing_complete,
                    object_id,
                    "",
                    "",
                    &Schema::NamedType(schema.fingerprint()),
                );
            } else {
                ui.text("WARNING: Could not find schema");
            }

            if is_editing && !is_editing_complete {
                EndContextBehavior::AllowResume
            } else {
                EndContextBehavior::Finish
            }
        });
}

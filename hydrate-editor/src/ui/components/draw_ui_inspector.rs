use crate::app_state::AppState;
use crate::ui::asset_browser_grid_drag_drop::AssetBrowserGridPayload;
use crate::ui::ImguiDisableHelper;
use crate::ui_state::UiState;
use hydrate_model::edit_context::EditContext;
use hydrate_model::value::ValueEnum;
use hydrate_model::{EndContextBehavior, Schema, SchemaEnum, Value};
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    _property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
    f: F,
) {
    let _disabled_helper = ImguiDisableHelper::new(read_only);
    let v = if property_inherited {
        if let Some(value) = edit_context.resolve_property(asset_id, &property_path) {
            value
        } else {
            Value::default_for_schema(schema, edit_context.schema_set())
        }
    } else {
        edit_context
            .get_property_override(asset_id, &property_path)
            .unwrap()
    };

    let new_value = (f)(ui, v);

    unsafe {
        if imgui::sys::igBeginPopupContextItem(
            im_str!("popup").as_ptr(), /*std::ptr::null()*/
            imgui::sys::ImGuiPopupFlags_MouseButtonRight as _,
        ) {
            if imgui::MenuItem::new(im_str!("Clear Override")).build(ui) {
                edit_context.remove_property_override(asset_id, &property_path);
            }

            if imgui::MenuItem::new(im_str!("Apply Override")).build(ui) {
                edit_context
                    .apply_property_override_to_prototype(asset_id, &property_path)
                    .unwrap();
            }

            imgui::sys::igEndPopup();
        }
    }

    if let Some(new_value) = new_value {
        edit_context
            .set_property_override(asset_id, &property_path, new_value)
            .unwrap();
    }
}

fn draw_inspector_simple_property_enum(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    schema_enum: &SchemaEnum,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
        |ui, value| {
            let v = value.as_enum().unwrap();
            let property_im_str = im_str!("{}", &property_name);

            let items: Vec<_> = schema_enum.symbols().iter().map(|x| x.name()).collect();
            let items_imstr: Vec<_> = items.iter().map(|x| im_str!("{}", x)).collect();
            let items_imstr_ref: Vec<_> = items_imstr.iter().map(|x| x).collect();
            let mut selected = items
                .iter()
                .position(|x| *x == v.symbol_name())
                .unwrap_or_default();

            let modified = imgui::ComboBox::new(&property_im_str).build_simple_string(
                ui,
                &mut selected,
                items_imstr_ref.as_slice(),
            );

            if ui.is_item_active() {
                *is_editing = true;
            }

            if ui.is_item_deactivated_after_edit() {
                *is_editing_complete = true;
            }

            if modified {
                Some(Value::Enum(ValueEnum::new(items[selected].to_string())))
            } else {
                None
            }
        },
    )
}

fn draw_inspector_simple_property_bool(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
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

fn draw_inspector_asset_ref(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    _is_editing: &mut bool,
    is_editing_complete: &mut bool,
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    property_inherited: bool,
    read_only: bool,
) {
    use hydrate_model::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        edit_context,
        asset_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        read_only,
        |ui, value| {
            let v = value.as_asset_ref().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let mut value = im_str!("{}", v.as_uuid());
            imgui::InputText::new(ui, &property_im_str, &mut value)
                .read_only(true)
                .build();

            if let Some(payload) =
                crate::ui::asset_browser_grid_drag_drop::asset_browser_grid_assets_drag_target_printf(
                    ui,
                    &ui_state.asset_browser_state.grid_state,
                )
            {
                match payload {
                    AssetBrowserGridPayload::Single(asset_id) => {
                        *is_editing_complete = true;
                        Some(Value::AssetRef(asset_id))
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
    //     asset_id,
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

fn draw_inspector_unimplemented_property(
    ui: &imgui::Ui,
    property_name: &str,
    type_name: &str,
) {
    ui.text(im_str!(
        "Unsupported property {} {}",
        property_name,
        type_name
    ));
}

fn draw_inspector_nexdb_property(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    edit_context: &mut EditContext,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    asset_id: hydrate_model::AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    read_only: bool,
) {
    use hydrate_model::*;

    match schema {
        Schema::Nullable(inner_schema) => {
            let property_inherited = !edit_context
                .get_null_override(asset_id, &property_path)
                .is_some();
            let mut is_nulled = edit_context
                .resolve_is_null(asset_id, &property_path)
                .unwrap_or(true);

            if imgui::CollapsingHeader::new(&im_str!("{}", property_name)).build(ui) {
                draw_property_style(ui, property_inherited, false, |ui| {
                    ui.text(property_path);
                });

                ui.indent();

                let disable_helper = ImguiDisableHelper::new(read_only);

                if is_nulled {
                    if ui.button(im_str!("Set Non-Null")) {
                        edit_context.set_null_override(
                            asset_id,
                            property_path,
                            NullOverride::SetNonNull,
                        );
                        is_nulled = false;
                    }
                } else {
                    if ui.button(im_str!("Set Null")) {
                        edit_context.set_null_override(
                            asset_id,
                            property_path,
                            NullOverride::SetNull,
                        );
                        is_nulled = true;
                    }
                }

                ui.same_line();
                if ui.button(im_str!("Inherit Null Status")) {
                    edit_context.remove_null_override(asset_id, property_path);
                }

                drop(disable_helper);
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
                        asset_id,
                        &inner_property_path,
                        "value",
                        &*inner_schema,
                        read_only,
                    );
                    id_token.pop();
                    ui.unindent();
                }
                ui.unindent();
            }
        }
        Schema::Boolean => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_bool(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::I32 => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_i32(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::I64 => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_i64(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::U32 => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_u32(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::U64 => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_u64(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::F32 => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_f32(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::F64 => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_f64(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::Bytes => {
            draw_inspector_unimplemented_property(ui, property_name, "bytes");
        }
        Schema::Buffer => {
            draw_inspector_unimplemented_property(ui, property_name, "buffer");
        }
        Schema::String => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_string(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
                );
            });
        }
        Schema::StaticArray(_) => {
            draw_inspector_unimplemented_property(ui, property_name, "static array");
        }
        Schema::DynamicArray(array) => {
            let resolve = edit_context.resolve_dynamic_array(asset_id, &property_path);
            let overrides: Vec<_> = edit_context
                .get_dynamic_array_overrides(asset_id, &property_path)
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
                        asset_id,
                        &field_path,
                        &id.to_string(),
                        array.item_type(),
                        read_only,
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
                        asset_id,
                        &field_path,
                        &id.to_string(),
                        array.item_type(),
                        read_only,
                    );
                    id_token.pop();
                }
                ui.unindent();
            }
        }
        Schema::Map(_) => {
            draw_inspector_unimplemented_property(ui, property_name, "map");
        }
        //Schema::RecordRef(_) => {}
        Schema::AssetRef(_named_type_fingerprint) => {
            let property_inherited = !edit_context.has_property_override(asset_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_asset_ref(
                    ui,
                    ui_state,
                    edit_context,
                    is_editing,
                    is_editing_complete,
                    asset_id,
                    property_path,
                    property_name,
                    schema,
                    property_inherited,
                    read_only,
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
                                asset_id,
                                &field_path,
                                field.name(),
                                field.field_schema(),
                                read_only,
                            );
                            id_token.pop();
                        }

                        // Don't indent if we are at root level
                        if !property_path.is_empty() {
                            ui.unindent();
                        }
                    }
                }
                SchemaNamedType::Enum(schema_enum) => {
                    // draw_inspector_unimplemented_property(
                    //     ui,
                    //     property_name,
                    //     "enum"
                    // );
                    let property_inherited =
                        !edit_context.has_property_override(asset_id, &property_path);
                    draw_inspector_simple_property_enum(
                        ui,
                        ui_state,
                        edit_context,
                        is_editing,
                        is_editing_complete,
                        asset_id,
                        property_path,
                        property_name,
                        schema,
                        property_inherited,
                        &schema_enum,
                        read_only,
                    );
                }
                SchemaNamedType::Fixed(_) => {
                    draw_inspector_unimplemented_property(ui, property_name, "fixed");
                }
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
          //             draw_inspector_nexdb_property(ui, edit_context, asset_id, &field_path, field.name(), field.field_schema());
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
    asset_id: hydrate_model::AssetId,
    read_only: bool,
) {
    let ui_state = &mut app_state.ui_state;
    app_state
        .db_state
        .editor_model
        .root_edit_context_mut()
        .with_undo_context("PropertyInspector", |edit_context| {
            let schema = edit_context.asset_schema(asset_id).clone();
            let mut is_editing = false;
            let mut is_editing_complete = false;

            if let Some(schema) = schema {
                draw_inspector_nexdb_property(
                    ui,
                    ui_state,
                    edit_context,
                    &mut is_editing,
                    &mut is_editing_complete,
                    asset_id,
                    "",
                    "",
                    &Schema::NamedType(schema.fingerprint()),
                    read_only,
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

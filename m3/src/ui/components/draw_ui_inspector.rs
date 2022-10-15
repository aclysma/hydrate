use crate::app_state::{AppState, UiState};
use crate::imgui_support::ImguiManager;
use imgui::im_str;
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImVec2,
};
use nexdb::{Database, Schema, DataSetDiffSet};
use std::convert::TryInto;
use crate::ui::asset_browser_grid_drag_drop::AssetBrowserGridPayload;

fn draw_property_style<F: FnOnce(&imgui::Ui)>(
    ui: &imgui::Ui,
    property_inherited: bool,
    property_overridden: bool,
    f: F,
) {
    let inherited_style_token = if property_inherited {
        Some(ui.push_style_color(imgui::StyleColor::Text, [0.2, 0.2, 0.2, 1.0]))
    } else {
        None
    };

    let overridden_style_token = if property_overridden {
        Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
    } else {
        None
    };

    (f)(ui);
}

fn draw_inspector_simple_property<F: FnOnce(&imgui::Ui, nexdb::Value) -> Option<nexdb::Value>>(
    ui: &imgui::Ui,
    ui_state: &UiState,
    db: &mut Database,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
    f: F,
) {
    use nexdb::*;
    let mut v = if property_inherited {
        if let Some(value) = db.resolve_property(object_id, &property_path) {
            value.clone()
        } else {
            db.schema_set().default_value_for_schema(schema)
            //Value::default_for_schema(schema).clone()
        }
    } else {
        db.get_property_override(object_id, &property_path)
            .unwrap()
            .clone()
    };

    let new_value = (f)(ui, v);

    unsafe {
        if imgui::sys::igBeginPopupContextItem(
            im_str!("popup").as_ptr(), /*std::ptr::null()*/
            imgui::sys::ImGuiPopupFlags_MouseButtonRight as _,
        ) {
            if imgui::MenuItem::new(im_str!("Clear Override")).build(ui) {
                db.remove_property_override(object_id, &property_path);
            }

            if imgui::MenuItem::new(im_str!("Apply Override")).build(ui) {
                db.apply_property_override_to_prototype(object_id, &property_path);
            }

            imgui::sys::igEndPopup();
        }
    }

    if let Some(new_value) = new_value {
        db.set_property_override(object_id, &property_path, new_value);
    }
}

fn draw_inspector_simple_property_bool(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_string().unwrap();
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
    property_inherited: bool,
) {
    use nexdb::*;

    draw_inspector_simple_property(
        ui,
        ui_state,
        db,
        object_id,
        property_path,
        property_name,
        schema,
        property_inherited,
        |ui, value| {
            let mut v = value.as_object_ref().unwrap();
            let property_im_str = im_str!("{}", &property_name);
            let mut value = im_str!("{}", v.as_uuid());
            imgui::InputText::new(ui, &property_im_str, &mut value).read_only(true).build();

            if let Some(payload) = crate::ui::asset_browser_grid_drag_drop::asset_browser_grid_drag_target(ui, &ui_state.asset_browser_state.grid_state) {
                match payload {
                    AssetBrowserGridPayload::Single(object_id) => {
                        *is_editing_complete = true;
                        Some(Value::ObjectRef(object_id))
                    }
                    AssetBrowserGridPayload::AllSelected => {
                        None
                    }
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
    //     db,
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
    db: &mut Database,
    is_editing: &mut bool,
    is_editing_complete: &mut bool,
    object_id: nexdb::ObjectId,
    property_path: &str,
    property_name: &str,
    schema: &nexdb::Schema,
) {
    use nexdb::*;

    match schema {
        Schema::Nullable(inner_schema) => {
            let property_inherited = !db.get_null_override(object_id, &property_path).is_some();
            let mut is_nulled = db
                .resolve_is_null(object_id, &property_path)
                .unwrap_or(true);

            if imgui::CollapsingHeader::new(&im_str!("{}", property_name)).build(ui) {
                draw_property_style(ui, property_inherited, false, |ui| {
                    ui.text(property_path);
                });

                ui.indent();

                if is_nulled {
                    if ui.button(im_str!("Set Non-Null")) {
                        db.set_null_override(object_id, property_path, NullOverride::SetNonNull);
                        is_nulled = false;
                    }
                } else {
                    if ui.button(im_str!("Set Null")) {
                        db.set_null_override(object_id, property_path, NullOverride::SetNull);
                        is_nulled = true;
                    }
                }

                ui.same_line();
                if ui.button(im_str!("Inherit Null Status")) {
                    db.remove_null_override(object_id, property_path);
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
                        db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_bool(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_i32(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_i64(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_u32(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_u64(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_f32(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_f64(
                    ui,
                    ui_state,
                    db,
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
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_simple_property_string(
                    ui,
                    ui_state,
                    db,
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
            let resolve = db.resolve_dynamic_array(object_id, &property_path);
            let overrides: Vec<_> = db
                .get_dynamic_array_overrides(object_id, &property_path).map(|x| x.cloned().collect()).unwrap_or_default();

            ui.text(im_str!("{}", property_name));
            if imgui::CollapsingHeader::new(&im_str!("elements")).build(ui) {
                ui.indent();
                let mut index = 0;
                for id in &resolve[0..(resolve.len() - overrides.len())] {
                    // inherited
                    let field_path = format!("{}.{}", property_path, id);
                    let id_token = ui.push_id(&field_path);
                    draw_inspector_nexdb_property(
                        ui,
                        ui_state,
                        db,
                        is_editing,
                        is_editing_complete,
                        object_id,
                        &field_path,
                        &id.to_string(),
                        array.item_type(),
                    );
                    id_token.pop();
                    index += 1;
                }

                for id in overrides {
                    let field_path = format!("{}.{}", property_path, id);
                    let id_token = ui.push_id(&field_path);
                    draw_inspector_nexdb_property(
                        ui,
                        ui_state,
                        db,
                        is_editing,
                        is_editing_complete,
                        object_id,
                        &field_path,
                        &id.to_string(),
                        array.item_type(),
                    );
                    id_token.pop();
                    index += 1;
                }
                ui.unindent();
            }
        }
        Schema::Map(_) => {}
        //Schema::RecordRef(_) => {}

        Schema::ObjectRef(named_type_fingerprint) => {
            let property_inherited = !db.has_property_override(object_id, &property_path);
            draw_property_style(ui, property_inherited, false, |ui| {
                draw_inspector_object_ref(
                    ui,
                    ui_state,
                    db,
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
            let named_type = db
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
                                db,
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
          //             draw_inspector_nexdb_property(ui, db, object_id, &field_path, field.name(), field.field_schema());
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
    object_id: nexdb::ObjectId,
) {
    let ui_state = &mut app_state.ui_state;
    app_state.db_state.db.with_undo_context("PropertyInspector", |db| {
        let schema = db.object_schema(object_id).clone();
        let mut is_editing = false;
        let mut is_editing_complete = false;

        if let Some(schema) = schema {
            draw_inspector_nexdb_property(
                ui,
                ui_state,
                db,
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

        is_editing && !is_editing_complete
    });
}

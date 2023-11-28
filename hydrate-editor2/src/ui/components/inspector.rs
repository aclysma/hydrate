use hydrate_model::{AssetId, HashSet, Schema, SchemaNamedType, SchemaSet};
use crate::ui_state::EditorModelUiState;

fn draw_inspector_property(
    ui: &mut egui::Ui,
    asset_id: AssetId,
    property_path: &str,
    property_name: &str,
    schema: &hydrate_model::Schema,
    read_only: bool,
    schema_set: &SchemaSet,
) {
    match schema {
        // Schema::Nullable(_) => unimplemented!(),
        // Schema::Boolean => unimplemented!(),
        // Schema::I32 => unimplemented!(),
        // Schema::I64 => unimplemented!(),
        // Schema::U32 => unimplemented!(),
        // Schema::U64 => unimplemented!(),
        Schema::F32 => {
            //let value =
            //egui::DragValue::new()
        },
        // Schema::F64 => unimplemented!(),
        // Schema::Bytes => unimplemented!(),
        // Schema::String => unimplemented!(),
        // Schema::StaticArray(_) => unimplemented!(),
        // Schema::DynamicArray(_) => unimplemented!(),
        // Schema::Map(_) => unimplemented!(),
        // Schema::AssetRef(_) => unimplemented!(),
        Schema::NamedType(schema_fingerprint) => {
            let schema = schema_set.find_named_type_by_fingerprint(*schema_fingerprint).unwrap();
            match schema {
                SchemaNamedType::Record(record_schema) => {
                    for field in record_schema.fields() {
                        let field_path = if !property_path.is_empty() {
                            format!("{}.{}", property_path, field.name())
                        } else {
                            field.name().to_string()
                        };

                        draw_inspector_property(
                            ui,
                            asset_id,
                            &field_path,
                            field.name(),
                            field.field_schema(),
                            read_only,
                            schema_set
                        );
                    }
                }
                // SchemaNamedType::Enum(_) => unimplemented!(),
                // SchemaNamedType::Fixed(_) => unimplemented!(),
                _ => { ui.label(format!("unimplemented {:?} {}", schema, property_name)); },
            }
        }
        _ => { ui.label(format!("unimplemented {:?} {}", schema, property_name)); },
    }
}

pub fn draw_inspector(
    ui: &mut egui::Ui,
    editor_model_ui_state: &EditorModelUiState,
    asset_id: AssetId,
    schema_set: &SchemaSet,
) {
    draw_inspector_property(
        ui,
        asset_id,
        "",
        "",
        &Schema::NamedType(editor_model_ui_state.all_asset_info.get(&asset_id).unwrap().schema.fingerprint()),
        false,
        schema_set,
    )
}


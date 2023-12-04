use demo_plugins::generated::*;
use hydrate::editor::egui::Ui;
use hydrate::editor::inspector_system::*;
use hydrate::model::{Record, Schema, SchemaDefRecordFieldMarkup, SchemaRecord, SchemaSet};

struct Vec3RecordInspector;

impl RecordInspector for Vec3RecordInspector {
    fn can_draw_as_single_value(&self) -> bool {
        true
    }

    fn draw_inspector_value(
        &self,
        ui: &mut Ui,
        ctx: InspectorContext,
    ) {
        ui.label("X");
        let field_path = ctx.property_path.push("x");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "x",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );
        ui.label("Y");
        let field_path = ctx.property_path.push("y");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "y",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );
        ui.label("Z");
        let field_path = ctx.property_path.push("z");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "z",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );
    }

    fn draw_inspector_rows(
        &self,
        table_body: &mut hydrate::editor::egui_extras::TableBody,
        ctx: InspectorContext,
        record: &SchemaRecord,
        indent_level: u32,
    ) {
        table_body.row(20.0, |mut row| {
            row.col(|mut ui| {
                draw_indented_label(ui, indent_level, ctx.display_name(), Some("test"));
            });
            row.col(|mut ui| {
                self.draw_inspector_value(ui, ctx);
            });
        });
    }
}

struct Vec4RecordInspector;

impl RecordInspector for Vec4RecordInspector {
    fn can_draw_as_single_value(&self) -> bool {
        true
    }

    fn draw_inspector_value(
        &self,
        ui: &mut Ui,
        ctx: InspectorContext,
    ) {
        ui.label("X");
        let field_path = ctx.property_path.push("x");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "x",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );
        ui.label("Y");
        let field_path = ctx.property_path.push("y");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "y",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );
        ui.label("Z");
        let field_path = ctx.property_path.push("z");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "z",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );

        ui.label("W");
        let field_path = ctx.property_path.push("w");
        draw_inspector_value(
            ui,
            InspectorContext {
                property_default_display_name: "w",
                property_path: &field_path,
                schema: &Schema::F32,
                ..ctx
            },
        );
    }

    fn draw_inspector_rows(
        &self,
        table_body: &mut hydrate::editor::egui_extras::TableBody,
        ctx: InspectorContext,
        record: &SchemaRecord,
        indent_level: u32,
    ) {
        table_body.row(20.0, |mut row| {
            row.col(|mut ui| {
                draw_indented_label(ui, indent_level, ctx.display_name(), Some("test"));
            });
            row.col(|mut ui| {
                self.draw_inspector_value(ui, ctx);
            });
        });
    }
}

pub fn create_registry(schema_set: &SchemaSet) -> InspectorRegistry {
    let mut inspector_registry = InspectorRegistry::default();
    inspector_registry.register_inspector::<Vec3Record>(schema_set, Vec3RecordInspector);
    inspector_registry.register_inspector::<Vec4Record>(schema_set, Vec4RecordInspector);
    inspector_registry
}

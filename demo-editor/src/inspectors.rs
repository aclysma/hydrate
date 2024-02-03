use demo_plugins::generated::*;
use hydrate::editor::action_queue::UIAction;
use hydrate::editor::egui::Ui;
use hydrate::editor::inspector_system::*;
use hydrate::model::{EndContextBehavior, Schema, SchemaSet, Value};

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
}

struct ColorRgbaU8RecordInspector;

impl RecordInspector for ColorRgbaU8RecordInspector {
    fn can_draw_as_single_value(&self) -> bool {
        true
    }

    fn draw_inspector_value(
        &self,
        ui: &mut Ui,
        ctx: InspectorContext,
    ) {
        //
        // Get the current values and put them in an egui color
        //
        let r_field_path = ctx.property_path.push("r");
        let g_field_path = ctx.property_path.push("g");
        let b_field_path = ctx.property_path.push("b");
        let a_field_path = ctx.property_path.push("a");

        let r = ctx
            .editor_model
            .root_edit_context()
            .resolve_property(ctx.primary_asset_id, r_field_path.path())
            .unwrap()
            .as_u32()
            .unwrap();
        let g = ctx
            .editor_model
            .root_edit_context()
            .resolve_property(ctx.primary_asset_id, g_field_path.path())
            .unwrap()
            .as_u32()
            .unwrap();
        let b = ctx
            .editor_model
            .root_edit_context()
            .resolve_property(ctx.primary_asset_id, b_field_path.path())
            .unwrap()
            .as_u32()
            .unwrap();
        let a = ctx
            .editor_model
            .root_edit_context()
            .resolve_property(ctx.primary_asset_id, a_field_path.path())
            .unwrap()
            .as_u32()
            .unwrap();

        let mut color = egui::Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a as u8);

        //
        // Draw the egui widget
        //
        let popup_id = ui.auto_id_with("popup");
        let was_open = ui.memory(|mem| mem.is_popup_open(popup_id));
        //ui.allocate_space(egui::vec2(5.0, 5.0));
        let response = ui.color_edit_button_srgba(&mut color);
        let is_open = ui.memory(|mem| mem.is_popup_open(popup_id));

        //
        // On change set the properties
        //
        if response.changed() {
            ctx.action_sender.queue_action(UIAction::SetProperty(
                ctx.selected_assets.iter().copied().collect(),
                r_field_path,
                Some(Value::U32(color.r() as u32)),
                EndContextBehavior::AllowResume,
            ));
            ctx.action_sender.queue_action(UIAction::SetProperty(
                ctx.selected_assets.iter().copied().collect(),
                g_field_path,
                Some(Value::U32(color.g() as u32)),
                EndContextBehavior::AllowResume,
            ));
            ctx.action_sender.queue_action(UIAction::SetProperty(
                ctx.selected_assets.iter().copied().collect(),
                b_field_path,
                Some(Value::U32(color.b() as u32)),
                EndContextBehavior::AllowResume,
            ));
            ctx.action_sender.queue_action(UIAction::SetProperty(
                ctx.selected_assets.iter().copied().collect(),
                a_field_path,
                Some(Value::U32(color.a() as u32)),
                EndContextBehavior::AllowResume,
            ));
        }

        //
        // Committing any property will commit the above changes, whether they were made on this frame
        // or a previous frame. Does nothing if we didn't send a property change notification
        //
        if was_open && !is_open {
            ctx.action_sender
                .queue_action(UIAction::CommitPendingUndoContext);
        }
    }
}

pub fn register_inspectors(
    schema_set: &SchemaSet,
    inspector_registry: &mut InspectorRegistry,
) {
    inspector_registry.register_inspector::<Vec3Record>(schema_set, Vec3RecordInspector);
    inspector_registry.register_inspector::<Vec4Record>(schema_set, Vec4RecordInspector);
    inspector_registry
        .register_inspector::<ColorRgbaU8Record>(schema_set, ColorRgbaU8RecordInspector);
}

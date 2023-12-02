#[derive(Default)]
pub struct EguiDebugUiState {
    pub show_settings_ui: bool,
    pub show_memory_ui: bool,
    pub show_style_ui: bool,
    pub show_inspection_ui: bool,
    pub show_texture_ui: bool,
}

pub fn show_egui_debug_ui(
    ctx: &egui::Context,
    egui_debug_ui_state: &EguiDebugUiState,
) {
    if egui_debug_ui_state.show_settings_ui {
        egui::Window::new("Egui Settings").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ctx.settings_ui(ui);
            });
        });
    }

    if egui_debug_ui_state.show_memory_ui {
        egui::Window::new("Egui Memory").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ctx.memory_ui(ui);
            });
        });
    }

    if egui_debug_ui_state.show_style_ui {
        egui::Window::new("Egui Style UI").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ctx.style_ui(ui);
            });
        });
    }

    if egui_debug_ui_state.show_inspection_ui {
        egui::Window::new("Egui Inspection UI").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ctx.inspection_ui(ui);
            });
        });
    }

    if egui_debug_ui_state.show_texture_ui {
        egui::Window::new("Egui Texture UI").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ctx.texture_ui(ui);
            });
        });
    }
}

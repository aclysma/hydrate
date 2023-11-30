use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::egui_debug_ui::EguiDebugUiState;

pub fn draw_main_menu_bar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    egui_debug_ui_state: &mut EguiDebugUiState,
    action_sender: &UIActionQueueSender,
) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Save All").clicked() {
                action_sender.queue_action(UIAction::SaveAll);
                ui.close_menu();
            }
            if ui.button("Revert All").clicked() {
                action_sender.queue_action(UIAction::RevertAll);
                ui.close_menu();
            }
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });


        ui.menu_button("Edit", |ui| {
            if ui.button("Undo").clicked() {
                action_sender.queue_action(UIAction::Undo);
                ui.close_menu();
            }
            if ui.button("Redo").clicked() {
                action_sender.queue_action(UIAction::Redo);
                ui.close_menu();
            }
        });

        ui.menu_button("Egui Debug", |ui| {
            if ui.checkbox(&mut egui_debug_ui_state.show_settings_ui, "Settings UI").changed() {
                ui.close_menu();
            }
            if ui.checkbox(&mut egui_debug_ui_state.show_memory_ui, "Memory UI").changed() {
                ui.close_menu();
            }
            if ui.checkbox(&mut egui_debug_ui_state.show_style_ui, "Style UI").changed() {
                ui.close_menu();
            }
            if ui.checkbox(&mut egui_debug_ui_state.show_inspection_ui, "Inspection UI").changed() {
                ui.close_menu();
            }
            if ui.checkbox(&mut egui_debug_ui_state.show_texture_ui, "Texture UI").changed() {
                ui.close_menu();
            }
        });

        ui.add_space(16.0);

        egui::widgets::global_dark_light_mode_buttons(ui);
    });
}
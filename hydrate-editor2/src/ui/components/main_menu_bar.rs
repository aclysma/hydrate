use crate::action_queue::{UIAction, UIActionQueueSender};

pub fn draw_main_menu_bar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
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

        ui.add_space(16.0);

        egui::widgets::global_dark_light_mode_buttons(ui);
    });
}
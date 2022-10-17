
use imgui::im_str;
use crate::{AppState, QueuedActions};

pub fn draw_menu_bar(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), || {
            //imgui::MenuItem::new(im_str!("New")).build(ui);
            //imgui::MenuItem::new(im_str!("Open")).build(ui);


            if imgui::MenuItem::new(im_str!("Save All")).build(ui) {
                app_state.action_queue.queue_action(QueuedActions::SaveAll);
            }

            if imgui::MenuItem::new(im_str!("Revert All")).build(ui) {
                app_state.action_queue.queue_action(QueuedActions::RevertAll);
            }
        });
        ui.menu(im_str!("Edit"), || {
            if imgui::MenuItem::new(im_str!("Undo")).build(ui) {
                app_state.db_state.editor_model.undo(); //undo_stack.undo(&mut app_state.db_state.db);
                app_state.action_queue.queue_action(QueuedActions::Undo);
            }
            if imgui::MenuItem::new(im_str!("Redo")).build(ui) {
                //app_state.db_state.editor_model.redo(); //undo_stack.undo(&mut app_state.db_state.db);
                app_state.action_queue.queue_action(QueuedActions::Redo);
            }
        });
        ui.menu(im_str!("Windows"), || {
            if imgui::MenuItem::new(im_str!("Toggle ImGui Demo Window")).build(ui) {
                app_state.ui_state.show_imgui_demo_window =
                    !app_state.ui_state.show_imgui_demo_window;
            }
            ui.separator();

            if imgui::MenuItem::new(im_str!("Reset Window Layout")).build(ui) {
                app_state.ui_state.redock_windows = true;
            }
        })
    });
}
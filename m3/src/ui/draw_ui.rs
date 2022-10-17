use crate::app_state::{ActionQueueSender, AppState, ModalAction, ModalActionControlFlow, QueuedActions, UiState};
use crate::imgui_support::ImguiManager;
use imgui::{im_str, PopupModal, Ui};
use imnodes::Context;
use crate::db_state::DbState;

#[derive(Default)]
struct TestModalAction1 {
    finished_first_draw: bool
}

impl ModalAction for TestModalAction1 {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("Test Popup 1"));
        }

        let result = PopupModal::new(imgui::im_str!("Test Popup 1")).build(ui, || {
            if ui.button(imgui::im_str!("close")) {
                println!("close");
                ui.close_current_popup();

                action_queue.queue_action(QueuedActions::TryBeginModalAction(Box::new(TestModalAction2::default())));

                return ModalActionControlFlow::End;
            }

            ModalActionControlFlow::Continue
        });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

#[derive(Default)]
struct TestModalAction2 {
    finished_first_draw: bool
}

impl ModalAction for TestModalAction2 {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("Test Popup 2"));
        }

        let result = PopupModal::new(imgui::im_str!("Test Popup 2")).build(ui, || {
            if ui.button(imgui::im_str!("close")) {
                println!("close");
                ui.close_current_popup();
                return ModalActionControlFlow::End;
            }

            ModalActionControlFlow::Continue
        });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

fn draw_menu_bar(
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

            if imgui::MenuItem::new(im_str!("Test Modal State")).build(ui) {
                app_state.action_queue.queue_action(QueuedActions::TryBeginModalAction(Box::new(TestModalAction1::default())))
            }

            ui.separator();

            if imgui::MenuItem::new(im_str!("Reset Window Layout")).build(ui) {
                app_state.ui_state.redock_windows = true;
            }
        })
    });
}

pub fn draw_imgui(
    imgui_manager: &ImguiManager,
    imnodes_editor: &mut imnodes::EditorContext,
    app_state: &mut AppState,
) {
    //
    //Draw an inspect window for the example struct
    //
    {
        imgui_manager.with_ui(|ui, imnodes_context| {
            crate::ui::views::draw_editor_view::draw_dockspace(ui, imnodes_editor, app_state);
            //crate::ui::views::draw_assets_view::draw_view(ui, app_state);
            draw_menu_bar(ui, app_state);
            //crate::ui::views::draw_2_pane_view::draw_2_pane_view(ui, app_state);
            //crate::ui::views::draw_3_pane_view::draw_3_pane_view(ui, app_state);

            if app_state.ui_state.show_imgui_demo_window {
                unsafe {
                    imgui::sys::igShowDemoWindow(&mut app_state.ui_state.show_imgui_demo_window);
                }
            }

            if let Some(modal_action) = &mut app_state.modal_action {
                let control_flow = modal_action.draw_imgui(
                    ui,
                    imnodes_context,
                    &mut app_state.db_state,
                    &mut app_state.ui_state,
                    app_state.action_queue.sender()
                );
                if control_flow == ModalActionControlFlow::End {
                    app_state.modal_action = None;
                }
            }
        });
    }
}

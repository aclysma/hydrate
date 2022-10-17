use std::path::PathBuf;
use imgui::PopupModal;
use imgui::sys::ImVec2;
use nexdb::{ObjectLocation, ObjectPath};
use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;

pub struct NewObjectModal {
    finished_first_draw: bool,
    create_location: ObjectPath,
}

impl NewObjectModal {
    pub fn new(create_location: ObjectPath) -> Self {
        NewObjectModal {
            finished_first_draw: false,
            create_location,
        }
    }
}

impl ModalAction for NewObjectModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("Create New Object"));
        }

        unsafe {
            imgui::sys::igSetNextWindowSize(ImVec2::new(600.0, 400.0), imgui::sys::ImGuiCond__ImGuiCond_Appearing as _);
        }

        let result = PopupModal::new(imgui::im_str!("Create New Object"))
            .build(ui, || {
                ui.text("Type of object to create");

                imgui::ChildWindow::new("child1")
                    .size([0.0, 100.0])
                    .build(ui, || {
                        for i in 0..20 {
                            ui.text(&format!("type {}", i))
                        }

                    });

                if ui.button(imgui::im_str!("Cancel")) {
                    ui.close_current_popup();

                    return ModalActionControlFlow::End;
                }

                unsafe { imgui::sys::igBeginDisabled(true); }

                ui.same_line();
                if ui.button(imgui::im_str!("TODO NOT IMPLEMENTED Create")) {
                    ui.close_current_popup();

                    return ModalActionControlFlow::End;
                }


                unsafe { imgui::sys::igEndDisabled(); }

                ModalActionControlFlow::Continue
            });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

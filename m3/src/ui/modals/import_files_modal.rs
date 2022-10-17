use std::path::PathBuf;
use imgui::PopupModal;
use imgui::sys::ImVec2;
use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;

pub struct ImportFilesModal {
    finished_first_draw: bool,
    files_to_import: Vec<PathBuf>
}

impl ImportFilesModal {
    pub fn new(files_to_import: Vec<PathBuf>) -> Self {
        ImportFilesModal {
            finished_first_draw: false,
            files_to_import
        }
    }
}

impl ModalAction for ImportFilesModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("Import Files"));
        }

        unsafe {
            imgui::sys::igSetNextWindowSize(ImVec2::new(600.0, 400.0), imgui::sys::ImGuiCond__ImGuiCond_Appearing as _);
        }

        let result = PopupModal::new(imgui::im_str!("Import Files"))
            .build(ui, || {
                ui.text("Files to be imported:");

                imgui::ChildWindow::new("child1")
                    .size([0.0, 100.0])
                    .build(ui, || {
                        for file in &self.files_to_import {
                            ui.text(file.to_str().unwrap());
                        }

                    });

                ui.separator();
                ui.text("Where to import the files");

                imgui::ChildWindow::new("child2")
                    .size([0.0, 180.0])
                    .build(ui, || {
                        for i in 0..20 {
                            ui.text("where to import");
                        }

                    });

                if ui.button(imgui::im_str!("Cancel")) {
                    ui.close_current_popup();

                    return ModalActionControlFlow::End;
                }

                unsafe { imgui::sys::igBeginDisabled(true); }

                ui.same_line();
                if ui.button(imgui::im_str!("TODO NOT IMPLEMENTED Import")) {
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

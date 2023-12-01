use egui::Ui;
use hydrate_model::pipeline::AssetEngine;
use crate::action_queue::UIActionQueueSender;
use crate::DbState;
use crate::modal_action::{default_modal_window, ModalAction, ModalActionControlFlow, ModalContext};
use crate::ui_state::EditorModelUiState;

#[derive(Default)]
pub struct TestModal {

}

impl ModalAction for TestModal {
    fn draw(&mut self, context: ModalContext) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Test Modal", context, |context, ui| {
            if ui.button("close").clicked() {
                control_flow = ModalActionControlFlow::End;
            }
        });

        control_flow
    }
}
use crate::action_queue::UIActionQueueSender;
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use crate::ui_state::EditorModelUiState;
use crate::DbState;
use egui::Ui;
use hydrate_model::pipeline::AssetEngine;

#[derive(Default)]
pub struct TestModal {}

impl ModalAction for TestModal {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Test Modal", context, |context, ui| {
            if ui.button("close").clicked() {
                control_flow = ModalActionControlFlow::End;
            }
        });

        control_flow
    }
}

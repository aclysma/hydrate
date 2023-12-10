use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};

#[derive(Default)]
pub struct TestModal {}

impl ModalAction for TestModal {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Test Modal", context, |_context, ui| {
            if ui.button("close").clicked() {
                control_flow = ModalActionControlFlow::End;
            }
        });

        control_flow
    }
}

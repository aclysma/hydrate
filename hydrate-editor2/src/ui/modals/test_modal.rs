use egui::Ui;
use hydrate_model::pipeline::AssetEngine;
use crate::action_queue::UIActionQueueSender;
use crate::modal_action::{ModalAction, ModalActionControlFlow};
use crate::ui_state::EditorModelUiState;

#[derive(Default)]
pub struct TestModal {

}

impl ModalAction for TestModal {
    fn draw(&mut self, ctx: &egui::Context, ui_state: &EditorModelUiState, asset_engine: &mut AssetEngine, action_queue: &UIActionQueueSender) -> ModalActionControlFlow {
        let mut close_clicked = false;
        egui::Window::new("Test Modal")
            .movable(false)
            .collapsible(false)
            .pivot(egui::Align2::CENTER_CENTER).current_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
            if ui.button("close").clicked() {
                close_clicked = true;
            }
        });

        if close_clicked {
            ModalActionControlFlow::End
        } else {
            ModalActionControlFlow::Continue
        }
    }
}
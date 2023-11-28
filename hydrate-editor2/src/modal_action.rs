use std::fmt::Formatter;
use hydrate_model::pipeline::AssetEngine;
use crate::action_queue::UIActionQueueSender;
use crate::ui_state::EditorModelUiState;

#[derive(PartialEq)]
pub enum ModalActionControlFlow {
    Continue,
    End,
}

pub trait ModalAction {
    fn draw(
        &mut self,
        ctx: &egui::Context,
        ui_state: &EditorModelUiState,
        asset_engine: &mut AssetEngine,
        action_queue: &UIActionQueueSender,
    ) -> ModalActionControlFlow;
}

impl std::fmt::Debug for dyn ModalAction {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ModalAction").finish()
    }
}

use crate::action_queue::UIActionQueueSender;
use crate::ui_state::EditorModelUiState;
use crate::DbState;
use hydrate_model::pipeline::AssetEngine;
use std::fmt::Formatter;

pub struct ModalContext<'a> {
    pub egui_ctx: &'a egui::Context,
    pub ui_state: &'a EditorModelUiState,
    pub asset_engine: &'a mut AssetEngine,
    pub db_state: &'a DbState,
    pub action_queue: &'a UIActionQueueSender,
}

pub fn default_modal_window<'a, F: FnOnce(ModalContext<'a>, &mut egui::Ui)>(
    title_text: &str,
    context: ModalContext<'a>,
    f: F,
) {
    egui::Window::new(title_text)
        .movable(false)
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(context.egui_ctx.screen_rect().center())
        .default_width(300.0)
        .show(context.egui_ctx, |ui| (f)(context, ui));
}

#[derive(PartialEq)]
pub enum ModalActionControlFlow {
    Continue,
    End,
}

pub trait ModalAction {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow;
}

impl std::fmt::Debug for dyn ModalAction {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish()
    }
}

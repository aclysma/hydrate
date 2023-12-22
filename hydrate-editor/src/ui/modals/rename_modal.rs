use crate::action_queue::UIAction;
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use crate::ui::components::draw_location_selector;
use hydrate_model::{AssetId, AssetLocation, AssetName};

pub struct MoveOrRenameAssetModal {
    asset_id: AssetId,
    new_name: String,
    new_location: Option<AssetLocation>,
}

impl MoveOrRenameAssetModal {
    pub fn new(asset_id: AssetId, new_name: String, new_location: Option<AssetLocation>) -> Self {
        MoveOrRenameAssetModal {
            asset_id,
            new_name,
            new_location,
        }
    }
}

impl ModalAction for MoveOrRenameAssetModal {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Rename/Move", context, |context, ui| {
            ui.label("Asset Name:");
            egui::TextEdit::singleline(&mut self.new_name).show(ui);

            ui.separator();
            ui.label("New Location");
            egui::ScrollArea::vertical()
                .id_source("locations")
                .auto_shrink([false, false])
                .max_height(200.0)
                .show(ui, |ui| {
                    draw_location_selector(
                        ui,
                        &context.db_state.editor_model,
                        context.action_queue,
                        context.ui_state,
                        &mut self.new_location,
                    );
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    control_flow = ModalActionControlFlow::End;
                }

                let is_enabled = !self.new_name.is_empty()
                    && self.new_location.is_some();
                if ui
                    .add_enabled(is_enabled, egui::Button::new("Move/Rename"))
                    .clicked()
                {
                    context.action_queue.queue_action(UIAction::MoveOrRename(
                        self.asset_id,
                        AssetName::new(&self.new_name),
                        self.new_location.clone().unwrap(),
                    ));
                    control_flow = ModalActionControlFlow::End;
                }
            });
        });

        control_flow
    }
}

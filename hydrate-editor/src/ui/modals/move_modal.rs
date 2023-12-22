use crate::action_queue::UIAction;
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use crate::ui::components::draw_location_selector;
use hydrate_model::{AssetId, AssetLocation, AssetName};

pub struct MoveAssetsModal {
    asset_ids: Vec<AssetId>,
    new_name: Option<String>,
    new_location: Option<AssetLocation>,
}

impl MoveAssetsModal {
    pub fn new_single_asset(asset_id: AssetId, new_name: String, new_location: Option<AssetLocation>) -> Self {
        MoveAssetsModal {
            asset_ids: vec![asset_id],
            new_name: Some(new_name),
            new_location,
        }
    }

    pub fn new_multiple_assets(asset_ids: Vec<AssetId>, new_location: Option<AssetLocation>) -> Self {
        MoveAssetsModal {
            asset_ids,
            new_name: None,
            new_location,
        }
    }
}

impl ModalAction for MoveAssetsModal {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Rename/Move", context, |context, ui| {

            if let Some(new_name) = &mut self.new_name {
                ui.label("Asset Name:");
                egui::TextEdit::singleline(new_name).show(ui);
            } else {
                ui.label("Multiple asset selected, cannot rename.");
            };

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

                let mut is_enabled = self.new_location.is_some();
                if let Some(name) = &self.new_name {
                    if name.is_empty() {
                        is_enabled = false;
                    }
                }

                if ui
                    .add_enabled(is_enabled, egui::Button::new("Move/Rename"))
                    .clicked()
                {
                    context.action_queue.queue_action(UIAction::MoveOrRename(
                        self.asset_ids.clone(),
                        self.new_name.clone().map(|x| AssetName::new(x)),
                        self.new_location.clone().unwrap(),
                    ));
                    control_flow = ModalActionControlFlow::End;
                }
            });
        });

        control_flow
    }
}

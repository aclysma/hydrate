use crate::action_queue::UIAction;
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use crate::ui::components::draw_location_selector;
use hydrate_model::{AssetId, AssetLocation, AssetName};

pub struct NewAssetModal {
    create_location: Option<AssetLocation>,
    prototype: Option<AssetId>,
    asset_name: String,
    schema_name: String,
}

impl NewAssetModal {
    pub fn new(create_location: Option<AssetLocation>) -> Self {
        NewAssetModal {
            create_location,
            prototype: None,
            asset_name: Default::default(),
            schema_name: Default::default(),
        }
    }

    pub fn new_with_prototype(
        create_location: Option<AssetLocation>,
        prototype: AssetId,
    ) -> Self {
        NewAssetModal {
            create_location,
            prototype: Some(prototype),
            asset_name: Default::default(),
            schema_name: Default::default(),
        }
    }
}

impl ModalAction for NewAssetModal {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Create New Asset", context, |context, ui| {
            ui.label("Asset Name:");
            egui::TextEdit::singleline(&mut self.asset_name).show(ui);

            let named_schema_type = if let Some(prototype) = self.prototype {
                context
                    .db_state
                    .editor_model
                    .root_edit_context()
                    .asset_schema(prototype)
                    .cloned()
            } else {
                ui.label("Schema Type:");
                crate::ui::components::schema_record_selector(
                    ui,
                    &mut self.schema_name,
                    context.db_state.editor_model.schema_set(),
                )
                .inner
            };

            ui.separator();
            ui.label("Where to create the asset");
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
                        &mut self.create_location,
                    );
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    control_flow = ModalActionControlFlow::End;
                }

                let is_enabled = !self.asset_name.is_empty()
                    && named_schema_type.is_some()
                    && self.create_location.is_some();
                if ui
                    .add_enabled(is_enabled, egui::Button::new("Create"))
                    .clicked()
                {
                    context.action_queue.queue_action(UIAction::NewAsset(
                        AssetName::new(&self.asset_name),
                        self.create_location.clone().unwrap(),
                        named_schema_type.unwrap(),
                        self.prototype,
                    ));
                    control_flow = ModalActionControlFlow::End;
                }
            });
        });

        control_flow
    }
}

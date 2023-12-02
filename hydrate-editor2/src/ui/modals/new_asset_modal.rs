use crate::action_queue::UIAction;
use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use hydrate_model::{AssetLocation, AssetName, EndContextBehavior, SchemaFingerprint};

pub struct NewAssetModal {
    create_location: AssetLocation,
    asset_name: String,
    schema_name: String,
}

impl NewAssetModal {
    pub fn new(create_location: AssetLocation) -> Self {
        NewAssetModal {
            create_location,
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
            ui.label("Schema Type:");
            let named_schema_type = crate::ui::components::schema_record_selector(
                ui,
                "schema_name",
                &mut self.schema_name,
                context.db_state.editor_model.schema_set(),
            )
            .inner;
            ui.label("TODO: Location selector");

            if ui.button("Cancel").clicked() {
                control_flow = ModalActionControlFlow::End;
            }

            let is_enabled = !self.asset_name.is_empty() && named_schema_type.is_some();
            if ui
                .add_enabled(is_enabled, egui::Button::new("create"))
                .clicked()
            {
                context.action_queue.queue_action(UIAction::NewAsset(
                    AssetName::new(&self.asset_name),
                    self.create_location.clone(),
                    named_schema_type.unwrap(),
                ));
                control_flow = ModalActionControlFlow::End;
            }
        });

        control_flow
    }
}

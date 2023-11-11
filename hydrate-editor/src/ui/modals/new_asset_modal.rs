use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;
use hydrate_model::pipeline::AssetEngine;
use hydrate_model::{HashSet, AssetLocation, AssetName, SchemaFingerprint};
use imgui::sys::ImVec2;
use imgui::{im_str, ImString, PopupModal};

pub struct NewAssetModal {
    finished_first_draw: bool,
    create_location: AssetLocation,
    asset_name: ImString,
    selected_type: Option<SchemaFingerprint>,
}

impl NewAssetModal {
    pub fn new(create_location: AssetLocation) -> Self {
        NewAssetModal {
            finished_first_draw: false,
            create_location,
            asset_name: Default::default(),
            selected_type: None,
        }
    }
}

impl ModalAction for NewAssetModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        _imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        _asset_engine: &mut AssetEngine,
        _action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(im_str!("Create New Asset"));
        }

        unsafe {
            imgui::sys::igSetNextWindowSize(
                ImVec2::new(600.0, 400.0),
                imgui::sys::ImGuiCond__ImGuiCond_Appearing as _,
            );
        }

        let result = PopupModal::new(im_str!("Create New Asset")).build(ui, || {
            ui.text(format!("Creating asset at: {:?}", self.create_location));

            println!("selected: {:?}", self.selected_type);

            imgui::InputText::new(ui, im_str!("Asset Name"), &mut self.asset_name)
                .chars_noblank(true)
                .resize_buffer(true)
                .build();

            ui.text("Type of asset to create");
            imgui::ListBox::new(im_str!("type_selection"))
                .size([0.0, 100.0])
                .build(ui, || {
                    for (fingerprint, named_type) in db_state.editor_model.schema_set().schemas() {
                        if named_type.as_record().is_some() {
                            let is_selected = self.selected_type == Some(*fingerprint);
                            println!("{:?} is selected: {:?}", fingerprint, is_selected);
                            if imgui::Selectable::new(&im_str!("{}", named_type.name()))
                                .selected(is_selected)
                                .build(ui)
                            {
                                self.selected_type = Some(*fingerprint);
                            }
                        }
                    }
                });

            if ui.button(im_str!("Cancel")) {
                ui.close_current_popup();
                return ModalActionControlFlow::End;
            }

            unsafe {
                imgui::sys::igBeginDisabled(self.selected_type.is_none());
            }

            ui.same_line();
            if ui.button(im_str!("Create")) {
                let asset_name = AssetName::new(self.asset_name.to_string());
                let schema = db_state
                    .editor_model
                    .schema_set()
                    .find_named_type_by_fingerprint(self.selected_type.unwrap())
                    .unwrap()
                    .as_record()
                    .unwrap()
                    .clone();
                let new_asset_id = db_state.editor_model.root_edit_context_mut().new_asset(
                    &asset_name,
                    &self.create_location,
                    &schema,
                );
                let mut selected_items = HashSet::default();
                selected_items.insert(new_asset_id);
                ui_state.asset_browser_state.grid_state.selected_items = selected_items;

                ui.close_current_popup();
                return ModalActionControlFlow::End;
            }

            unsafe {
                imgui::sys::igEndDisabled();
            }

            ModalActionControlFlow::Continue
        });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

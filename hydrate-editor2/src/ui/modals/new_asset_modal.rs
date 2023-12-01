use hydrate_model::{AssetLocation, AssetName, EndContextBehavior, SchemaFingerprint};
use crate::action_queue::UIAction;
use crate::modal_action::{default_modal_window, ModalAction, ModalActionControlFlow, ModalContext};

pub struct NewAssetModal {
    finished_first_draw: bool,
    create_location: AssetLocation,
    asset_name: String,
    schema_name: String,
    //selected_type: Option<SchemaFingerprint>,
}

impl NewAssetModal {
    pub fn new(create_location: AssetLocation) -> Self {
        NewAssetModal {
            finished_first_draw: false,
            create_location,
            asset_name: Default::default(),
            schema_name: Default::default(),
            //selected_type: None,
        }
    }
}

impl ModalAction for NewAssetModal {
    fn draw(&mut self, context: ModalContext) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Create New Asset", context, |context, ui| {
            ui.label("Asset Name:");
            egui::TextEdit::singleline(&mut self.asset_name).show(ui);
            ui.label("Schema Type:");
            let named_schema_type = crate::ui::components::schema_record_selector(ui, "schema_name", &mut self.schema_name, context.db_state.editor_model.schema_set()).inner;
            ui.label("TODO: Location selector");

            if ui.button("Cancel").clicked() {
                control_flow = ModalActionControlFlow::End;
            }

            //ui.set_enabled(!self.asset_name.is_empty() && schema.is_some());
            if let Some(named_schema_type) = named_schema_type {
                if ui.button("Create").clicked() {
                    control_flow = ModalActionControlFlow::End;
                    context.action_queue.queue_action(UIAction::NewAsset(AssetName::new(&self.asset_name), self.create_location.clone(), named_schema_type));
                    control_flow = ModalActionControlFlow::End;
                }
            }



            // egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(200.0).show(ui, |ui| {
            //     for (fingerprint, named_type) in context.db_state.editor_model.schema_set().schemas() {
            //         if named_type.try_as_record().is_some() {
            //             // let is_selected = self.selected_type == Some(*fingerprint);
            //             // println!("{:?} is selected: {:?}", fingerprint, is_selected);
            //             // if imgui::Selectable::new(&im_str!("{}", named_type.name()))
            //             //     .selected(is_selected)
            //             //     .build(ui)
            //             // {
            //             //     self.selected_type = Some(*fingerprint);
            //             // }
            //             let selected = false;
            //             if ui.selectable_label(selected, named_type.name()).clicked() {
            //
            //             }
            //         }
            //     }
            // });



            // if ui.button("close").clicked() {
            //     control_flow = ModalActionControlFlow::End;
            // }

            //             ui.text(format!("Creating asset at: {:?}", self.create_location));
//
//             println!("selected: {:?}", self.selected_type);
//
//             imgui::InputText::new(ui, im_str!("Asset Name"), &mut self.asset_name)
//                 .chars_noblank(true)
//                 .resize_buffer(true)
//                 .build();
//
//             ui.text("Type of asset to create");
//             imgui::ListBox::new(im_str!("type_selection"))
//                 .size([0.0, 100.0])
//                 .build(ui, || {
//                     for (fingerprint, named_type) in db_state.editor_model.schema_set().schemas() {
//                         if named_type.try_as_record().is_some() {
//                             let is_selected = self.selected_type == Some(*fingerprint);
//                             println!("{:?} is selected: {:?}", fingerprint, is_selected);
//                             if imgui::Selectable::new(&im_str!("{}", named_type.name()))
//                                 .selected(is_selected)
//                                 .build(ui)
//                             {
//                                 self.selected_type = Some(*fingerprint);
//                             }
//                         }
//                     }
//                 });
//
//             if ui.button(im_str!("Cancel")) {
//                 ui.close_current_popup();
//                 return ModalActionControlFlow::End;
//             }
//
//             unsafe {
//                 imgui::sys::igBeginDisabled(self.selected_type.is_none());
//             }
//
//             ui.same_line();
//             if ui.button(im_str!("Create")) {
//                 let asset_name = AssetName::new(self.asset_name.to_string());
//                 let schema = db_state
//                     .editor_model
//                     .schema_set()
//                     .find_named_type_by_fingerprint(self.selected_type.unwrap())
//                     .unwrap()
//                     .as_record()
//                     .unwrap()
//                     .clone();
//                 let new_asset_id = db_state.editor_model.root_edit_context_mut().new_asset(
//                     &asset_name,
//                     &self.create_location,
//                     &schema,
//                 );
//                 let mut selected_items = HashSet::default();
//                 selected_items.insert(new_asset_id);
//                 ui_state.asset_browser_state.grid_state.selected_items = selected_items;
//
//                 ui.close_current_popup();
//                 return ModalActionControlFlow::End;
//             }
//
//             unsafe {
//                 imgui::sys::igEndDisabled();
//             }
        });

        control_flow
    }
}


// use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
// use crate::db_state::DbState;
// use crate::ui_state::UiState;
// use hydrate_model::pipeline::AssetEngine;
// use hydrate_model::{AssetLocation, AssetName, HashSet, SchemaFingerprint};
// use imgui::sys::ImVec2;
// use imgui::{im_str, ImString, PopupModal};
//
// pub struct NewAssetModal {
//     finished_first_draw: bool,
//     create_location: AssetLocation,
//     asset_name: ImString,
//     selected_type: Option<SchemaFingerprint>,
// }
//
// impl NewAssetModal {
//     pub fn new(create_location: AssetLocation) -> Self {
//         NewAssetModal {
//             finished_first_draw: false,
//             create_location,
//             asset_name: Default::default(),
//             selected_type: None,
//         }
//     }
// }
//
// impl ModalAction for NewAssetModal {
//     fn draw_imgui(
//         &mut self,
//         ui: &mut imgui::Ui,
//         _imnodes_context: &mut imnodes::Context,
//         db_state: &mut DbState,
//         ui_state: &mut UiState,
//         _asset_engine: &mut AssetEngine,
//         _action_queue: ActionQueueSender,
//     ) -> ModalActionControlFlow {
//         if !self.finished_first_draw {
//             ui.open_popup(im_str!("Create New Asset"));
//         }
//
//         unsafe {
//             imgui::sys::igSetNextWindowSize(
//                 ImVec2::new(600.0, 400.0),
//                 imgui::sys::ImGuiCond__ImGuiCond_Appearing as _,
//             );
//         }
//
//         let result = PopupModal::new(im_str!("Create New Asset")).build(ui, || {
//             ui.text(format!("Creating asset at: {:?}", self.create_location));
//
//             println!("selected: {:?}", self.selected_type);
//
//             imgui::InputText::new(ui, im_str!("Asset Name"), &mut self.asset_name)
//                 .chars_noblank(true)
//                 .resize_buffer(true)
//                 .build();
//
//             ui.text("Type of asset to create");
//             imgui::ListBox::new(im_str!("type_selection"))
//                 .size([0.0, 100.0])
//                 .build(ui, || {
//                     for (fingerprint, named_type) in db_state.editor_model.schema_set().schemas() {
//                         if named_type.try_as_record().is_some() {
//                             let is_selected = self.selected_type == Some(*fingerprint);
//                             println!("{:?} is selected: {:?}", fingerprint, is_selected);
//                             if imgui::Selectable::new(&im_str!("{}", named_type.name()))
//                                 .selected(is_selected)
//                                 .build(ui)
//                             {
//                                 self.selected_type = Some(*fingerprint);
//                             }
//                         }
//                     }
//                 });
//
//             if ui.button(im_str!("Cancel")) {
//                 ui.close_current_popup();
//                 return ModalActionControlFlow::End;
//             }
//
//             unsafe {
//                 imgui::sys::igBeginDisabled(self.selected_type.is_none());
//             }
//
//             ui.same_line();
//             if ui.button(im_str!("Create")) {
//                 let asset_name = AssetName::new(self.asset_name.to_string());
//                 let schema = db_state
//                     .editor_model
//                     .schema_set()
//                     .find_named_type_by_fingerprint(self.selected_type.unwrap())
//                     .unwrap()
//                     .as_record()
//                     .unwrap()
//                     .clone();
//                 let new_asset_id = db_state.editor_model.root_edit_context_mut().new_asset(
//                     &asset_name,
//                     &self.create_location,
//                     &schema,
//                 );
//                 let mut selected_items = HashSet::default();
//                 selected_items.insert(new_asset_id);
//                 ui_state.asset_browser_state.grid_state.selected_items = selected_items;
//
//                 ui.close_current_popup();
//                 return ModalActionControlFlow::End;
//             }
//
//             unsafe {
//                 imgui::sys::igEndDisabled();
//             }
//
//             ModalActionControlFlow::Continue
//         });
//
//         self.finished_first_draw = true;
//         result.unwrap_or(ModalActionControlFlow::End)
//     }
// }

use crate::app_state::{ActionQueueSender, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;
use imgui::sys::ImVec2;
use imgui::{im_str, ImString, PopupModal};
use nexdb::{HashSet, ObjectLocation, ObjectName, SchemaFingerprint};
use std::path::PathBuf;
use crate::importers::{ImporterRegistry, ImportJobs};

pub struct NewObjectModal {
    finished_first_draw: bool,
    create_location: ObjectLocation,
    object_name: ImString,
    selected_type: Option<SchemaFingerprint>,
}

impl NewObjectModal {
    pub fn new(create_location: ObjectLocation) -> Self {
        NewObjectModal {
            finished_first_draw: false,
            create_location,
            object_name: Default::default(),
            selected_type: None,
        }
    }
}

impl ModalAction for NewObjectModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        importer_registry: &ImporterRegistry,
        import_jobs: &mut ImportJobs,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(im_str!("Create New Object"));
        }

        unsafe {
            imgui::sys::igSetNextWindowSize(
                ImVec2::new(600.0, 400.0),
                imgui::sys::ImGuiCond__ImGuiCond_Appearing as _,
            );
        }

        let result = PopupModal::new(im_str!("Create New Object")).build(ui, || {
            ui.text(format!("Creating object at: {:?}", self.create_location));

            println!("selected: {:?}", self.selected_type);

            imgui::InputText::new(ui, im_str!("Object Name"), &mut self.object_name)
                .chars_noblank(true)
                .resize_buffer(true)
                .build();

            ui.text("Type of object to create");
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
                let object_name = ObjectName::new(self.object_name.to_string());
                let schema = db_state
                    .editor_model
                    .schema_set()
                    .find_named_type_by_fingerprint(self.selected_type.unwrap())
                    .unwrap()
                    .as_record()
                    .unwrap()
                    .clone();
                let new_object_id = db_state.editor_model.root_edit_context_mut().new_object(
                    &object_name,
                    &self.create_location,
                    &schema,
                );
                let mut selected_items = HashSet::default();
                selected_items.insert(new_object_id);
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

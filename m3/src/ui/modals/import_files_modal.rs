use crate::app_state::{ActionQueueSender, AppState, ModalAction, ModalActionControlFlow};
use crate::db_state::DbState;
use crate::ui_state::UiState;
use imgui::sys::ImVec2;
use imgui::{im_str, PopupModal, StyleColor, TreeNodeFlags};
use std::path::PathBuf;
use nexdb::{LocationTreeNode, ObjectLocation};

pub struct ImportFilesModal {
    finished_first_draw: bool,
    files_to_import: Vec<PathBuf>,
    selected_import_location: ObjectLocation,
}

impl ImportFilesModal {
    pub fn new(files_to_import: Vec<PathBuf>) -> Self {
        ImportFilesModal {
            finished_first_draw: false,
            files_to_import,
            selected_import_location: ObjectLocation::null()
        }
    }
}

fn default_flags() -> imgui::TreeNodeFlags {
    imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK
        | imgui::TreeNodeFlags::OPEN_ON_ARROW
        | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
}

fn leaf_flags() -> imgui::TreeNodeFlags {
    imgui::TreeNodeFlags::LEAF | default_flags()
}

pub fn path_tree_node(
    ui: &imgui::Ui,
    db_state: &DbState,
    ui_state: &mut UiState,
    child_name: &str,
    tree_node: &LocationTreeNode,
    selected_import_location: &mut ObjectLocation,
) {
    let id = im_str!("{}", tree_node.location.path_node_id().as_uuid());
    let is_selected = *selected_import_location == tree_node.location;
    let is_modified = tree_node.has_changes;

    let label = if is_modified {
        im_str!("{}*", child_name)
    } else {
        im_str!("{}", child_name)
    };

    let color = if is_modified {
        [1.0, 1.0, 0.0, 1.0]
    } else {
        [1.0, 1.0, 1.0, 1.0]
    };

    let mut flags = if tree_node.children.is_empty() {
        leaf_flags()
    } else {
        default_flags()
    };

    if is_selected {
        flags |= TreeNodeFlags::SELECTED;
    }

    let style = ui.push_style_color(StyleColor::Text, color);
    let ds_tree_node = imgui::TreeNode::new(&id).label(&label).flags(flags);
    let token = ds_tree_node.push(ui);
    style.pop();

    //try_select_tree_node(ui, ui_state, &tree_node.location);
    if ui.is_item_clicked() && !ui.is_item_toggled_open() {
        *selected_import_location = tree_node.location.clone();
    }

    if let Some(token) = token {
        // Draw nodes with children first
        for (child_name, child) in &tree_node.children {
            if !child.children.is_empty() {
                path_tree_node(
                    ui,
                    db_state,
                    ui_state,
                    child_name.name(),
                    child,
                    selected_import_location
                );
            }
        }

        // Then draw nodes without children
        for (child_name, child) in &tree_node.children {
            if child.children.is_empty() {
                path_tree_node(
                    ui,
                    db_state,
                    ui_state,
                    child_name.name(),
                    child,
                    selected_import_location
                );
            }
        }
    }
}


pub fn path_tree(
    ui: &imgui::Ui,
    db_state: &mut DbState,
    ui_state: &mut UiState,
    selected_import_location: &mut ObjectLocation,
) {
    db_state.editor_model.refresh_tree_node_cache();
    let tree = db_state.editor_model.cached_location_tree();

    let show_root = true;
    if show_root {
        path_tree_node(
            ui,
            db_state,
            ui_state,
            "db:/",
            &tree.root_node,
            selected_import_location
        );
    } else {
        // Draw nodes with children first
        for (child_name, child) in &tree.root_node.children {
            if !child.children.is_empty() {
                path_tree_node(
                    ui,
                    db_state,
                    ui_state,
                    child_name.name(),
                    child,
                    selected_import_location
                );
            }
        }

        // Then draw nodes without children
        for (child_name, child) in &tree.root_node.children {
            if child.children.is_empty() {
                path_tree_node(
                    ui,
                    db_state,
                    ui_state,
                    child_name.name(),
                    child,
                    selected_import_location
                );
            }
        }
    }
}

impl ModalAction for ImportFilesModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("Import Files"));
        }

        unsafe {
            imgui::sys::igSetNextWindowSize(
                ImVec2::new(600.0, 400.0),
                imgui::sys::ImGuiCond__ImGuiCond_Appearing as _,
            );
        }

        let result = PopupModal::new(imgui::im_str!("Import Files")).build(ui, || {
            ui.text("Files to be imported:");

            imgui::ChildWindow::new("child1")
                .size([0.0, 100.0])
                .build(ui, || {
                    for file in &self.files_to_import {
                        ui.text(file.to_str().unwrap());
                    }
                });

            ui.separator();
            ui.text("Where to import the files");

            imgui::ChildWindow::new("child2")
                .size([0.0, 180.0])
                .build(ui, || {
                    path_tree(ui, db_state, ui_state, &mut self.selected_import_location);
                });

            if ui.button(imgui::im_str!("Cancel")) {
                ui.close_current_popup();

                return ModalActionControlFlow::End;
            }

            unsafe {
                imgui::sys::igBeginDisabled(true);
            }

            ui.same_line();
            if ui.button(imgui::im_str!("TODO NOT IMPLEMENTED Import")) {
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

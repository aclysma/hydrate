use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui::modals::NewAssetModal;
use crate::ui_state::EditorModelUiState;
use egui::{InnerResponse, Response, Ui};
use hydrate_model::{AssetId, AssetLocation, EditorModel, LocationTreeNode};

fn draw_tree_node(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    selected_asset_location: &mut Option<AssetLocation>,
    tree_node: &LocationTreeNode,
) {
    let path_node_asset_id = tree_node.location.path_node_id();
    let name = editor_model
        .root_edit_context()
        .asset_name(tree_node.location.path_node_id());
    let name = name
        .map(|x| {
            x.as_string()
                .cloned()
                .unwrap_or_else(|| tree_node.location.path_node_id().to_string())
        })
        .unwrap();

    let mut is_selected = false;
    if let Some(selected_asset_location) = selected_asset_location {
        is_selected = selected_asset_location.path_node_id() == path_node_asset_id;
    }

    let response = if tree_node.children.len() > 0 {
        let id = ui.make_persistent_id(tree_node.location.path_node_id());
        let mut collapsing_header =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

        if let Some(selected_location) = selected_asset_location {
            let location_chain = editor_model
                .root_edit_context()
                .asset_location_chain(selected_location.path_node_id())
                .unwrap();
            if location_chain.contains(&tree_node.location) {
                collapsing_header.set_open(true);
            }
        }

        let (toggle_button_response, header_response, body_response) = collapsing_header
            .show_header(ui, |ui| ui.toggle_value(&mut is_selected, &name))
            .body_unindented(|ui| {
                ui.horizontal(|ui| {
                    crate::ui::add_indent_spacing(ui);
                    ui.vertical(|ui| {
                        for (key, child_tree_node) in &tree_node.children {
                            draw_tree_node(
                                ui,
                                editor_model,
                                action_sender,
                                selected_asset_location,
                                child_tree_node,
                            );
                        }
                    });
                });
            });

        header_response.inner
    } else {
        ui.horizontal(|ui| {
            let prev_item_spacing = ui.spacing_mut().item_spacing;
            ui.spacing_mut().item_spacing.x = 0.0; // the toggler button uses the full indent width
                                                   // empty space where the collapsing header's icon would be
            ui.allocate_space(egui::vec2(ui.spacing().indent, ui.spacing().icon_width));
            ui.spacing_mut().item_spacing = prev_item_spacing;

            ui.selectable_label(is_selected, &name)
        })
        .inner
    };

    if response.clicked() {
        *selected_asset_location = Some(tree_node.location);
    }
}

pub fn draw_location_selector(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    selected_asset_location: &mut Option<AssetLocation>,
) {
    ui.push_id("asset tree", |ui| {
        ui.style_mut().visuals.indent_has_left_vline = false;

        for (_, tree_node) in &editor_model_ui_state.location_tree.root_nodes {
            draw_tree_node(
                ui,
                editor_model,
                action_sender,
                selected_asset_location,
                tree_node,
            );
        }
    });
}

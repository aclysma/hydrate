use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui::modals::NewAssetModal;
use crate::ui_state::EditorModelUiState;
use egui::{InnerResponse, Response, Ui};
use hydrate_model::{AssetId, EditorModel, LocationTreeNode};

#[derive(Default)]
pub struct AssetTreeUiState {
    selected_tree_node: Option<AssetId>,
}

fn draw_tree_node(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    asset_tree_ui_state: &mut AssetTreeUiState,
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

    let mut is_selected = asset_tree_ui_state.selected_tree_node == Some(path_node_asset_id);

    crate::ui::drag_drop::drag_source(
        ui,
        egui::Id::new(path_node_asset_id),
        DragDropPayload::AssetReference(path_node_asset_id),
        |ui| {
            //if tree_node.children.len() > 0 {
            //ui.push_id(tree_node.location.path_node_id(), |ui| {

            //Reject drop if asset is dropped on itself
            //TODO: Make this also reject if dragged is already a child of this node
            let can_accept = match crate::ui::drag_drop::peek_payload() {
                None => false,
                Some(DragDropPayload::AssetReference(payload_asset_id)) => {
                    payload_asset_id != path_node_asset_id
                }
            };

            let response = if tree_node.children.len() > 0 {
                let id = ui.make_persistent_id(tree_node.location.path_node_id());
                let (toggle_button_response, header_response, body_response) =
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        false,
                    )
                    .show_header(ui, |ui| {
                        let response = crate::ui::drag_drop::drop_target(ui, can_accept, |ui| {
                            ui.toggle_value(&mut is_selected, &name)
                        });

                        handle_drop_on_asset_tree_node(ui, &response, action_sender, tree_node);

                        response.inner
                    })
                    .body(|ui| {
                        for (key, child_tree_node) in &tree_node.children {
                            draw_tree_node(
                                ui,
                                editor_model,
                                action_sender,
                                asset_tree_ui_state,
                                child_tree_node,
                            );
                        }
                    });

                header_response.inner
            } else {
                ui.horizontal(|ui| {
                    let prev_item_spacing = ui.spacing_mut().item_spacing;
                    ui.spacing_mut().item_spacing.x = 0.0; // the toggler button uses the full indent width
                                                           // empty space where the collapsing header's icon would be
                    ui.allocate_space(egui::vec2(ui.spacing().indent, ui.spacing().icon_width));
                    ui.spacing_mut().item_spacing = prev_item_spacing;

                    let response = crate::ui::drag_drop::drop_target(ui, can_accept, |ui| {
                        ui.selectable_label(is_selected, &name)
                    });

                    handle_drop_on_asset_tree_node(ui, &response, action_sender, tree_node);

                    response.inner
                })
                .inner
            };

            if response.clicked() {
                asset_tree_ui_state.selected_tree_node = Some(path_node_asset_id);
            }

            response.context_menu(|ui| {
                ui.label("hi");
                if ui.button("New Asset").clicked() {
                    action_sender.try_set_modal_action(NewAssetModal::new(tree_node.location));
                    ui.close_menu();
                }
            })

            //});
        },
    );
}

fn handle_drop_on_asset_tree_node(
    ui: &mut Ui,
    response: &InnerResponse<Response>,
    action_sender: &UIActionQueueSender,
    dropped_on_tree_node: &LocationTreeNode,
) {
    if let Some(payload) = crate::ui::drag_drop::try_take_dropped_payload(ui, &response.response) {
        match payload {
            DragDropPayload::AssetReference(payload_asset_id) => {
                println!(
                    "DROPPED ASSET {:?} ON {:?}",
                    payload_asset_id,
                    dropped_on_tree_node.location.path_node_id()
                );
                action_sender.queue_action(UIAction::MoveAsset(
                    payload_asset_id,
                    dropped_on_tree_node.location,
                ));
            }
            _ => unimplemented!(),
        }
    }
}

pub fn draw_asset_tree(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    asset_tree_ui_state: &mut AssetTreeUiState,
) {
    ui.label("ASSET TREE");
    ui.push_id("asset tree", |ui| {
        ui.style_mut().visuals.indent_has_left_vline = false;

        for (_, tree_node) in &editor_model_ui_state.location_tree.root_nodes {
            draw_tree_node(
                ui,
                editor_model,
                action_sender,
                asset_tree_ui_state,
                tree_node,
            );
        }
    });
}

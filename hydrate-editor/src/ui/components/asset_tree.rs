use crate::action_queue::{UIAction, UIActionQueueSender};
use crate::ui::drag_drop::DragDropPayload;
use crate::ui::modals::NewAssetModal;
use crate::ui_state::EditorModelUiState;
use egui::{InnerResponse, Response, Ui};
use hydrate_model::{AssetLocation, EditorModel, LocationTreeNode};

#[derive(Default)]
pub struct AssetTreeUiState {
    pub selected_tree_node: Option<AssetLocation>,
}

fn draw_tree_node(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    asset_tree_ui_state: &mut AssetTreeUiState,
    tree_node: &LocationTreeNode,
    indent_count: u32,
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

    let mut is_selected = asset_tree_ui_state.selected_tree_node == Some(tree_node.location);

    crate::ui::drag_drop::drag_source(
        ui,
        egui::Id::new(path_node_asset_id),
        DragDropPayload::AssetReference(path_node_asset_id),
        |ui| {
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

                let mut collapsing_header_state =
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        false,
                    );

                if let Some(selected_tree_node) = asset_tree_ui_state.selected_tree_node {
                    let location_chain = editor_model
                        .root_edit_context()
                        .asset_location_chain(selected_tree_node.path_node_id())
                        .unwrap();
                    if location_chain.contains(&tree_node.location) {
                        collapsing_header_state.set_open(true);
                    }
                }

                let inner_response = ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let header_response = collapsing_header_state.show_header(ui, |ui| {
                            let response =
                                crate::ui::drag_drop::drop_target(ui, can_accept, |ui| {
                                    ui.toggle_value(&mut is_selected, &name)
                                });

                            handle_drop_on_asset_tree_node(ui, &response, action_sender, tree_node);

                            response.inner
                        });

                        let (_, header_response, _) = header_response.body_unindented(|ui| {
                            ui.horizontal(|ui| {
                                crate::ui::add_indent_spacing(ui);
                                ui.vertical(|ui| {
                                    for (_, child_tree_node) in &tree_node.children {
                                        draw_tree_node(
                                            ui,
                                            editor_model,
                                            action_sender,
                                            asset_tree_ui_state,
                                            child_tree_node,
                                            indent_count + 1,
                                        );
                                    }
                                });
                            });
                        });

                        header_response.inner
                    })
                    .inner
                });

                inner_response.inner
            } else {
                ui.horizontal(|ui| {
                    crate::ui::add_icon_spacing(ui);

                    let response = crate::ui::drag_drop::drop_target(ui, can_accept, |ui| {
                        ui.selectable_label(is_selected, &name)
                    });

                    handle_drop_on_asset_tree_node(ui, &response, action_sender, tree_node);

                    response.inner
                })
                .inner
            };

            if response.clicked() {
                asset_tree_ui_state.selected_tree_node = Some(tree_node.location);
            }

            response.context_menu(|ui| {
                tree_node_context_menu(action_sender, tree_node, ui);
            })

            //});
        },
    );
}

fn tree_node_context_menu(
    action_sender: &UIActionQueueSender,
    tree_node: &LocationTreeNode,
    ui: &mut Ui,
) {
    if ui.button("New Asset").clicked() {
        action_sender.try_set_modal_action(NewAssetModal::new(Some(tree_node.location)));
        ui.close_menu();
    }
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
    egui::ScrollArea::vertical()
        .max_width(f32::INFINITY)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label("ASSET TREE");
            ui.push_id("asset tree", |ui| {
                ui.style_mut().visuals.indent_has_left_vline = false;
                ui.style_mut().spacing.item_spacing = egui::vec2(2.0, 2.0);

                if ui
                    .selectable_label(asset_tree_ui_state.selected_tree_node.is_none(), "project")
                    .clicked()
                {
                    asset_tree_ui_state.selected_tree_node = None;
                }

                for (_, tree_node) in &editor_model_ui_state.location_tree.root_nodes {
                    draw_tree_node(
                        ui,
                        editor_model,
                        action_sender,
                        asset_tree_ui_state,
                        tree_node,
                        0,
                    );
                }
            });
        });
}

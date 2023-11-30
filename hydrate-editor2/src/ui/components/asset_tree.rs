use hydrate_model::{AssetId, EditorModel, LocationTreeNode};
use crate::action_queue::UIActionQueueSender;
use crate::ui::drag_drop::DragDropPayload;
use crate::ui_state::EditorModelUiState;

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
    let name = editor_model.root_edit_context().asset_name(tree_node.location.path_node_id());
    let name = name.map(|x| x.as_string().cloned().unwrap_or_else(|| tree_node.location.path_node_id().to_string())).unwrap();

    let mut is_selected = asset_tree_ui_state.selected_tree_node == Some(path_node_asset_id);

    //if tree_node.children.len() > 0 {
    ui.push_id(tree_node.location.path_node_id(), |ui| {
        let response = if tree_node.children.len() > 0 {
            let id = ui.make_persistent_id(tree_node.location.path_node_id());
            let (toggle_button_response, header_response, body_response) =
                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    ui.push_id("collapsing header inner", |ui| {
                        ui.toggle_value(&mut is_selected, &name)//.context_menu(|ui| {ui.label("test2");})
                    }).inner
                }).body(|ui| {
                for (key, child_tree_node) in &tree_node.children {
                    draw_tree_node(ui, editor_model, action_sender, asset_tree_ui_state, child_tree_node);
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

                let response = ui.selectable_label(is_selected, &name);
                response
            }).inner
        };

        if response.clicked() {
            asset_tree_ui_state.selected_tree_node = Some(path_node_asset_id);
        }

        // crate::ui::drag_drop::drag_source(ui, path_node_asset_id, DragDropPayload::AssetReference(path_node_asset_id), |ui| {
        //
        // })

        response.context_menu(|ui| {
            ui.label("hi");
        });
    });
}

pub fn draw_asset_tree(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    action_sender: &UIActionQueueSender,
    editor_model_ui_state: &EditorModelUiState,
    asset_tree_ui_state: &mut AssetTreeUiState,
) {
    ui.label("ASSET TREE");

    ui.style_mut().visuals.indent_has_left_vline = false;

    for (_, tree_node) in &editor_model_ui_state.location_tree.root_nodes {
        draw_tree_node(
            ui,
            editor_model,
            action_sender,
            asset_tree_ui_state,
            tree_node
        );
    }
}
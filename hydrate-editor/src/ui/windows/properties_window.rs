use crate::ui_state::ActiveToolRegion;
use crate::AppState;
use hydrate_model::ObjectId;
use imgui::im_str;

pub fn draw_properties_window_single_select(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    object_id: ObjectId,
) {
    ui.text(format!("Object: {}", object_id.as_uuid()));

    let edit_context = app_state.db_state.editor_model.root_edit_context();

    let name = edit_context.object_name(object_id);
    let location = edit_context.object_location(object_id).unwrap();

    ui.text(im_str!(
        "Name: {}",
        name.as_string().cloned().unwrap_or_default()
    ));
    let import_info = edit_context.import_info(object_id);
    if let Some(import_info) = import_info {
        ui.text(im_str!("Imported From: {}", import_info.source_file_path().to_string_lossy()));
        if !import_info.importable_name().is_empty() {
            ui.text(im_str!("Importable Name: {}", import_info.importable_name()));
        }
    }

    ui.text(im_str!(
        "Path Node: {}",
        app_state
            .db_state
            .editor_model
            .object_display_name_long(location.path_node_id())
    ));

    if ui.button(im_str!("Force Rebuild")) {
        app_state.asset_engine.queue_build_operation(object_id);
    }

    if let Some(prototype) = edit_context.object_prototype(object_id) {
        if ui.button(im_str!(">>")) {
            let mut grid_state = &mut app_state.ui_state.asset_browser_state.grid_state;
            grid_state.first_selected = Some(prototype);
            grid_state.last_selected = Some(prototype);
            grid_state.selected_items.clear();
            grid_state.selected_items.insert(prototype);
        }
        ui.same_line();

        let prototype_display_name = app_state
            .db_state
            .editor_model
            .object_display_name_long(prototype);

        ui.text(format!("Prototype: {}", prototype_display_name));
    }

    crate::ui::components::draw_ui_inspector::draw_inspector_nexdb(ui, app_state, object_id);
}

pub fn draw_properties_window(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    if app_state.ui_state.active_tool_region == Some(ActiveToolRegion::AssetBrowserGrid)
        || app_state.ui_state.active_tool_region == Some(ActiveToolRegion::AssetBrowserTree)
    {
        let selected_items = &app_state
            .ui_state
            .asset_browser_state
            .grid_state
            .selected_items;
        if selected_items.len() > 1 {
            ui.text("multiple selected");
        } else if selected_items.len() == 1 {
            let item = selected_items.iter().next().unwrap();

            if app_state
                .db_state
                .editor_model
                .root_edit_context()
                .has_object(*item)
            {
                draw_properties_window_single_select(ui, app_state, *item);
            }
        } else {
            ui.text("no selection");
        }
    }
}

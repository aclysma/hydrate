use crate::app_state::QueuedActions;
use crate::ui_state::ActiveToolRegion;
use crate::AppState;
use hydrate_model::AssetId;
use imgui::im_str;

pub fn draw_properties_window_single_select(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    asset_id: AssetId,
) {
    ui.text(format!("Asset: {}", asset_id.as_uuid()));

    let edit_context = app_state.db_state.editor_model.root_edit_context();

    let name = edit_context.asset_name(asset_id);
    let location = edit_context.asset_location(asset_id).unwrap();

    ui.text(im_str!(
        "Name: {}",
        name.unwrap().as_string().cloned().unwrap_or_default()
    ));
    let import_info = edit_context.import_info(asset_id);
    if let Some(import_info) = import_info {
        ui.text(im_str!(
            "Imported From: {}",
            import_info.source_file_path().to_string_lossy()
        ));
        if let Some(importable_name) = import_info.importable_name() {
            ui.text(im_str!("Importable Name: {}", importable_name));
        }
    }

    let is_generated = app_state.db_state.editor_model.is_generated_asset(asset_id);
    if is_generated {
        ui.text(im_str!("This asset is generated from a source file and can't be modified unless it is persisted to disk. A new asset file will be created and source file changes will no longer affect it."));
    }

    if is_generated {
        if ui.button(im_str!("Persist Asset")) {
            app_state
                .action_queue
                .queue_action(QueuedActions::PersistAssets(vec![asset_id]));
        }
    }

    ui.text(im_str!(
        "Path Node: {}",
        app_state
            .db_state
            .editor_model
            .asset_display_name_long(location.path_node_id())
    ));

    if ui.button(im_str!("Force Rebuild")) {
        app_state.asset_engine.queue_build_operation(asset_id);
    }

    if let Some(prototype) = edit_context.asset_prototype(asset_id) {
        if ui.button(im_str!(">>")) {
            let grid_state = &mut app_state.ui_state.asset_browser_state.grid_state;
            grid_state.first_selected = Some(prototype);
            grid_state.last_selected = Some(prototype);
            grid_state.selected_items.clear();
            grid_state.selected_items.insert(prototype);
        }
        ui.same_line();

        let prototype_display_name = app_state
            .db_state
            .editor_model
            .asset_display_name_long(prototype);

        ui.text(format!("Prototype: {}", prototype_display_name));
    }

    // unsafe {
    //     is::igPushItemFlag(is::ImGuiItemFlags__ImGuiItemFlags_Disabled as _, true);
    // }

    let read_only = is_generated;
    crate::ui::components::draw_ui_inspector::draw_inspector_nexdb(
        ui, app_state, asset_id, read_only,
    );

    // if is_generated {
    //     unsafe {
    //         is::igPopItemFlag();
    //     }
    // }
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
                .has_asset(*item)
            {
                draw_properties_window_single_select(ui, app_state, *item);
            }
        } else {
            ui.text("no selection");
        }
    }
}

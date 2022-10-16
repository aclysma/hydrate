use imgui::im_str;
use nexdb::ObjectId;
use crate::app_state::ActiveToolRegion;
use crate::AppState;


pub fn draw_properties_window_single_select(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    object_id: ObjectId
) {
    ui.text(format!("Object: {}", object_id.as_uuid()));
    if let Some(prototype) = app_state.db_state.editor_model.root_context().object_prototype(object_id) {
        if ui.button(im_str!(">>")) {
            let mut grid_state = &mut app_state.ui_state.asset_browser_state.grid_state;
            grid_state.first_selected = Some(prototype);
            grid_state.last_selected = Some(prototype);
            grid_state.selected_items.clear();
            grid_state.selected_items.insert(prototype);
        }
        ui.same_line();
        ui.text(format!("Prototype: {}", prototype.as_uuid()));
    }



    crate::ui::components::draw_ui_inspector::draw_inspector_nexdb(ui, app_state, object_id);
}

pub fn draw_properties_window(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    if app_state.ui_state.active_tool_region == Some(ActiveToolRegion::AssetBrowserGrid) || app_state.ui_state.active_tool_region == Some(ActiveToolRegion::AssetBrowserTree){
        let selected_items = &app_state.ui_state.asset_browser_state.grid_state.selected_items;
        if selected_items.len() > 1 {
            ui.text("multiple selected");

        } else if selected_items.len() == 1 {
            let item = selected_items.iter().next().unwrap();
            draw_properties_window_single_select(ui, app_state, *item);

        } else {
            ui.text("no selection");

        }
    }
}
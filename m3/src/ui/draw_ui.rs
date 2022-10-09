use crate::ui::components::draw_ui_inspector::*;
use crate::app::AppState;
use crate::imgui_support::ImguiManager;
use imgui::im_str;
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImVec2,
};
use std::convert::TryInto;
use crate::ui::views::draw_editor_view::draw_dockspace;

fn draw_menu_bar(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), || {
            imgui::MenuItem::new(im_str!("New")).build(ui);
            imgui::MenuItem::new(im_str!("Open")).build(ui);
            imgui::MenuItem::new(im_str!("Save")).build(ui);
            if imgui::MenuItem::new(im_str!("Reset Window Layout")).build(ui) {
                app_state.redock_windows = true;
            }
        });
    });
}

pub fn draw_imgui(
    imgui_manager: &ImguiManager,
    app_state: &mut AppState,
) {
    //
    //Draw an inspect window for the example struct
    //
    {
        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            crate::ui::views::draw_editor_view::draw_dockspace(ui, app_state);
            //crate::ui::views::draw_assets_view::draw_view(ui, app_state);
            draw_menu_bar(ui, app_state);
            //crate::ui::views::draw_2_pane_view::draw_2_pane_view(ui, app_state);
            //crate::ui::views::draw_3_pane_view::draw_3_pane_view(ui, app_state);
        });
    }
}

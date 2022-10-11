use crate::ui::components::draw_ui_inspector::*;
use crate::app::AppState;
use crate::imgui_support::ImguiManager;
use imgui::{im_str, ImStr, ImString};
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImVec2,
};
use std::convert::TryInto;
use std::ffi::CString;
use imgui::sys as is;

const WINDOW_NAME_PROPERTIES: &str = "PropertiesWindow";
const WINDOW_NAME_ASSETS: &str = "AssetsWindow";
const WINDOW_NAME_ASSETS_LEFT: &str = "AssetsWindowLeft";
const WINDOW_NAME_ASSETS_RIGHT: &str = "AssetsWindowRight";


fn draw_root_dockspace(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let root_dockspace_id = is::igDockSpaceOverViewport(is::igGetMainViewport(), 0, std::ptr::null());
        if app_state.ui_state.redock_windows {
            let work_size = (*is::igGetMainViewport()).WorkSize;

            // Setup root node
            let mut dockspace_main = root_dockspace_id;
            is::igDockBuilderAddNode(dockspace_main, is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace);

            // Create sub-nodes
            is::igDockBuilderSetNodeSize(root_dockspace_id, work_size);
            let dockspace_properties = is::igDockBuilderSplitNode(dockspace_main, is::ImGuiDir_Right, 0.2, std::ptr::null_mut(), &mut dockspace_main);

            // Dock the windows
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_PROPERTIES).as_ptr(), dockspace_properties);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS).as_ptr(), dockspace_main);

            is::igDockBuilderFinish(root_dockspace_id);
        }
    }
}


pub fn draw_view(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    draw_root_dockspace(ui, app_state);

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_PROPERTIES))
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("properties"));
        window_token.end();
    }

    crate::ui::windows::assets_window::draw_assets_dockspace_and_window(ui, app_state);

    app_state.ui_state.redock_windows = false;

}
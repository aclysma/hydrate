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

const WINDOW_NAME_DOC_OUTLINE: &str = "DocumentOutlineWindow";
const WINDOW_NAME_DOC_CONTENTS: &str = "DocumentContents";
const WINDOW_NAME_PROPERTIES: &str = "PropertiesWindow";
const WINDOW_NAME_ASSETS: &str = "AssetsWindow";
const WINDOW_NAME_ASSETS_LEFT: &str = "AssetsWindowLeft";
const WINDOW_NAME_ASSETS_RIGHT: &str = "AssetsWindowRight";

pub fn draw_assets_dockspace(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let window = imgui::sys::igGetCurrentWindow();
        let mut work_size = (*window).Size;
        // Subtract 22 to account for tabs above. This gets rid of the scroll bar.
        work_size.y -= 22.0;

        // Get the ID for WINDOW_NAME_ASSETS
        let assets_window_id = (*is::igGetCurrentWindow()).ID;
        let root_assets_dockspace_id = is::igDockSpace(assets_window_id, work_size, 0, std::ptr::null_mut());

        // The first time through, set up the left/right panes of the asset browser so that they are pinned inside the asset browser window
        // and nothing can dock inside them
        if app_state.redock_windows {
            let mut assets_main = root_assets_dockspace_id;
            is::igDockBuilderAddNode(
                assets_main,
                is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace |
                    imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking |
                    imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe);

            // We hardcode a 1.0 size ratio here because the first run, width will be the window's init size rather than the actual size.
            // The window width is 300, so ratio = 1 results in left pane being 300 pixels wide.
            is::igDockBuilderSetNodeSize(root_assets_dockspace_id, work_size);
            let assets_left = is::igDockBuilderSplitNode(assets_main, is::ImGuiDir_Left, 1.0, std::ptr::null_mut(), &mut assets_main);

            // Assign windows to dock nodes
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS_LEFT).as_ptr(), assets_left);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS_RIGHT).as_ptr(), assets_main);

            // Don't draw tab bars on the left/right panes
            (*imgui::sys::igDockBuilderGetNode(assets_left)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;
            (*imgui::sys::igDockBuilderGetNode(assets_main)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;

            is::igDockBuilderFinish(root_assets_dockspace_id);
        }
    }
}

pub fn draw_dockspace(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let root_dockspace_id = is::igDockSpaceOverViewport(is::igGetMainViewport(), 0, std::ptr::null());
        if app_state.redock_windows {
            let work_size = (*is::igGetMainViewport()).WorkSize;

            // Setup root node
            let mut dockspace_main = root_dockspace_id;
            is::igDockBuilderAddNode(dockspace_main, is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace);

            // Create sub-nodes
            is::igDockBuilderSetNodeSize(root_dockspace_id, work_size);
            let dockspace_properties = is::igDockBuilderSplitNode(dockspace_main, is::ImGuiDir_Right, 0.2, std::ptr::null_mut(), &mut dockspace_main);
            let dockspace_assets = is::igDockBuilderSplitNode(dockspace_main, is::ImGuiDir_Down, 0.3, std::ptr::null_mut(), &mut dockspace_main);
            let dockspace_outline = is::igDockBuilderSplitNode(dockspace_main, is::ImGuiDir_Left, 0.2, std::ptr::null_mut(), &mut dockspace_main);

            // Dock the windows
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_DOC_CONTENTS).as_ptr(), dockspace_main);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_PROPERTIES).as_ptr(), dockspace_properties);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS).as_ptr(), dockspace_assets);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_DOC_OUTLINE).as_ptr(), dockspace_outline);

            is::igDockBuilderFinish(root_dockspace_id);
        }
    }


    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_DOC_CONTENTS))
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("document contents"));
        window_token.end();
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_PROPERTIES))
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("properties"));
        window_token.end();
    }

    // We set padding to zero when creating the assets window so that the vertical splitter bar
    // will go from top to bottom of the window
    unsafe {
        is::igPushStyleVar_Vec2(
            is::ImGuiStyleVar_WindowPadding as _,
            ImVec2::new(0.0, 0.0),
        );
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS))
        // The width of this matters, it sets the initial width of the left column
        .size([300.0, 400.0], imgui::Condition::Once)
        .flags(imgui::WindowFlags::NO_COLLAPSE)
        .begin(ui);

    unsafe {
        is::igPopStyleVar(1);
    }

    if let Some(window_token) = window_token {
        draw_assets_dockspace(ui, app_state);

        let inner_window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS_LEFT))
            .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            ui.text(im_str!("assets left"));
            inner_window_token.end();
        }


        let inner_window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS_RIGHT))
            .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            ui.text(im_str!("assets right"));
            inner_window_token.end();
        }

        window_token.end();
    } else {
        //TODO: keepalive the assets dockspace
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_DOC_OUTLINE))
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("outline"));
        window_token.end();
    }

    app_state.redock_windows = false;

}
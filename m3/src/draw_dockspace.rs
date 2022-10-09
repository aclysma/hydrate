use super::draw_ui_inspector::*;
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
        let mut work_size = ImVec2::new(1416.0, 285.0);
        //is::igGetContentRegionAvail(&mut work_size);
        //is::igGetWindowContentRegionMax(&mut work_size);
        //is::igGetWindowSize(&mut work_size);
        println!("work size {:?}", work_size);

        // let window = imgui::sys::igGetCurrentWindow();
        // let work_pos = (*window).WorkRect.Min.clone();
        // let mut work_size = (*window).WorkRect.Max.clone();
        // work_size.x -= work_pos.x;
        // work_size.y -= work_pos.y;

        let start = WINDOW_NAME_ASSETS.as_ptr() as *const std::os::raw::c_char;
        let end = start.add(WINDOW_NAME_ASSETS.len());
        let assets_window_id = imgui::sys::igGetIDStrStr(start, end);
        let root_assets_dockspace_id = is::igDockSpace(assets_window_id, work_size, 0, std::ptr::null_mut());

        if app_state.redock_windows {
            let mut assets_main = root_assets_dockspace_id;
            is::igDockBuilderAddNode(assets_main, is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking| imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe);

            is::igDockBuilderSetNodeSize(root_assets_dockspace_id, work_size);
            let assets_left = is::igDockBuilderSplitNode(assets_main, is::ImGuiDir_Left, 0.2, std::ptr::null_mut(), &mut assets_main);

            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS_LEFT).as_ptr(), assets_left);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS_RIGHT).as_ptr(), assets_main);

            //(*imgui::sys::igDockBuilderGetNode(root_assets_dockspace_id)).SharedFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;// | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;
            (*imgui::sys::igDockBuilderGetNode(assets_left)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;// | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe| imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;
            (*imgui::sys::igDockBuilderGetNode(assets_main)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;// | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;

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
        //.position([550.0, 100.0], imgui::Condition::Once)
        //.size([300.0, 400.0], imgui::Condition::Once)
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("document contents"));
        window_token.end();
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_PROPERTIES))
        //.position([550.0, 100.0], imgui::Condition::Once)
        //.size([300.0, 400.0], imgui::Condition::Once)
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("properties"));
        window_token.end();
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS))
        //.position([550.0, 100.0], imgui::Condition::Once)
        //.size([300.0, 400.0], imgui::Condition::Once)
        .begin(ui);

    if let Some(window_token) = window_token {
        //ui.text(im_str!("assets"));
        draw_assets_dockspace(ui, app_state);




        let inner_window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS_LEFT))
            //.position([550.0, 100.0], imgui::Condition::Once)
            //.size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            ui.text(im_str!("assets left"));
            inner_window_token.end();
        }


        let inner_window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS_RIGHT))
            //.position([550.0, 100.0], imgui::Condition::Once)
            //.size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            ui.text(im_str!("assets right"));
            inner_window_token.end();
        }



        window_token.end();
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_DOC_OUTLINE))
        //.position([550.0, 100.0], imgui::Condition::Once)
        //.size([300.0, 400.0], imgui::Condition::Once)
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("outline"));
        window_token.end();
    }


    app_state.redock_windows = false;

}
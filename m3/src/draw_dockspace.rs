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
        //let mut work_size = ImVec2::new(1416.0, 285.0);
        //let mut work_size = ImVec2::new(0.0, 0.0);
        //is::igGetContentRegionAvail(&mut work_size);
        //is::igGetWindowContentRegionMax(&mut work_size);
        //is::igGetWindowSize(&mut work_size);



        let window = imgui::sys::igGetCurrentWindow();
        let mut work_size = (*window).Size;
        //work_size.x = 400.0;
        work_size.y -= 22.0;
        //let work_pos = (*window).WorkRect.Min.clone();
        //let mut work_size = (*window).WorkRect.Max.clone();
        //work_size.x -= work_pos.x;
        //work_size.y -= work_pos.y;

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
            let assets_left = is::igDockBuilderSplitNode(assets_main, is::ImGuiDir_Left, 1.0, std::ptr::null_mut(), &mut assets_main);

            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS_LEFT).as_ptr(), assets_left);
            is::igDockBuilderDockWindow(ImString::new(WINDOW_NAME_ASSETS_RIGHT).as_ptr(), assets_main);

            //(*imgui::sys::igDockBuilderGetNode(root_assets_dockspace_id)).SharedFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;// | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;
            (*imgui::sys::igDockBuilderGetNode(assets_left)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;// | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe| imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;
            (*imgui::sys::igDockBuilderGetNode(assets_main)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;// | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;

            is::igDockBuilderFinish(root_assets_dockspace_id);
        }
    }
}

// bool Splitter(bool split_vertically, float thickness, float* size1, float* size2, float min_size1, float min_size2, float splitter_long_axis_size = -1.0f)
// {
// using namespace ImGui;
// ImGuiContext& g = *GImGui;
// ImGuiWindow* window = g.CurrentWindow;
// ImGuiID id = window->GetID("##Splitter");
// ImRect bb;
// bb.Min = window->DC.CursorPos + (split_vertically ? ImVec2(*size1, 0.0f) : ImVec2(0.0f, *size1));
// bb.Max = bb.Min + CalcItemSize(split_vertically ? ImVec2(thickness, splitter_long_axis_size) : ImVec2(splitter_long_axis_size, thickness), 0.0f, 0.0f);
// return SplitterBehavior(id, bb, split_vertically ? ImGuiAxis_X : ImGuiAxis_Y, size1, size2, min_size1, min_size2, 0.0f);
// }

fn splitter(vertical: bool, thickness: f32, size1: &mut f32, size2: &mut f32, min_size1: f32, min_size2: f32) {
    unsafe {
        let splitter_long_axis_size = -1.0;
        let window = is::igGetCurrentWindow();
        let id = is::igGetID_Str(CString::new("##Splitter").unwrap().as_ptr());
        let mut bb = is::ImRect {
            Min: ImVec2::zero(),
            Max: ImVec2::zero()
        };
        let add_to_min = if vertical {
            is::ImVec2::new(*size1, 0.0)
        } else {
            is::ImVec2::new(0.0, *size1)
        };

        let calc_item_size_in = if vertical {
            is::ImVec2::new(thickness, splitter_long_axis_size)
        } else {
            is::ImVec2::new(splitter_long_axis_size, thickness)
        };
        let mut calc_item_size_out = ImVec2::zero();
        is::igCalcItemSize(&mut calc_item_size_out as _, calc_item_size_in, 0.0, 0.0);

        let cursor_pos = (*window).DC.CursorPos;
        bb.Min = ImVec2::new(cursor_pos.x + add_to_min.x, cursor_pos.y + add_to_min.y);
        bb.Max = ImVec2::new(bb.Min.x + calc_item_size_out.x, bb.Min.y + calc_item_size_out.y);

        let axis = if vertical {
            is::ImGuiAxis_ImGuiAxis_X
        } else {
            is::ImGuiAxis_ImGuiAxis_Y
        };

        is::igSplitterBehavior(bb, id, axis, size1, size2, min_size1, min_size2, 0.0, 0.0);


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

    unsafe {
        //imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        //imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        is::igPushStyleVar_Vec2(
            is::ImGuiStyleVar_WindowPadding as _,
            ImVec2::new(0.0, 0.0),
        );
    }

    let window_token = imgui::Window::new(&ImString::new(WINDOW_NAME_ASSETS))
        //.position([550.0, 100.0], imgui::Condition::Once)
        .size([300.0, 400.0], imgui::Condition::Once)
        .flags(imgui::WindowFlags::NO_COLLAPSE)
        .begin(ui);

    unsafe {
        is::igPopStyleVar(1);
    }

    if let Some(window_token) = window_token {
        //ui.text(im_str!("assets"));

        //
        // SPLITTER APPROACH
        //
        //let mut size1 = 200.0;
        //let mut size2 = 200.0;
        //let mut min_size1 = 10.0;
        //let mut min_size2 = 10.0;
        //splitter(true, 1.0, &mut app_state.splitter_size1, &mut app_state.splitter_size2, min_size1, min_size2);



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
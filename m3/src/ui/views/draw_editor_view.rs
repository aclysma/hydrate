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

fn draw_root_dockspace(
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
            //is::igDockBuilderDockWindow(ImString::new(crate::ui::WINDOW_NAME_DOC_CONTENTS).as_ptr(), dockspace_main);
            is::igDockBuilderDockWindow(ImString::new(crate::ui::WINDOW_NAME_PROPERTIES).as_ptr(), dockspace_properties);
            is::igDockBuilderDockWindow(ImString::new(crate::ui::WINDOW_NAME_ASSETS).as_ptr(), dockspace_assets);
            is::igDockBuilderDockWindow(ImString::new(crate::ui::WINDOW_NAME_DOC_OUTLINE).as_ptr(), dockspace_outline);

            is::igDockBuilderFinish(root_dockspace_id);
        }
    }
}

pub fn draw_dockspace(
    ui: &imgui::Ui,
    imnodes_editor: &mut imnodes::EditorContext,
    app_state: &mut AppState,
) {
    draw_root_dockspace(ui, app_state);


    let window_token = imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_DOC_CONTENTS))
        .begin(ui);

    if let Some(window_token) = window_token {
        let mut id_gen = imnodes_editor.new_identifier_generator();
        imnodes::editor(imnodes_editor, |mut editor| {
            let node_id1 = id_gen.next_node();
            node_id1.set_position(0.0, 0.0, imnodes::CoordinateSystem::GridSpace);
            let out_pin1 = id_gen.next_output_pin();
            editor.add_node(node_id1, |mut node| {
                node.add_titlebar(|| {
                    ui.text(im_str!("simple node"));
                });

                node.add_input(id_gen.next_input_pin(), imnodes::PinShape::Circle, || {
                    ui.text(im_str!("input"));
                });

                node.add_output(out_pin1, imnodes::PinShape::QuadFilled, || {
                    ui.text(im_str!("output"));
                });
            });

            let node_id2 = id_gen.next_node();
            node_id2.set_position(200.0, 0.0, imnodes::CoordinateSystem::GridSpace);
            let in_pin1 = id_gen.next_input_pin();
            editor.add_node(node_id2, |mut node| {
                node.add_titlebar(|| {
                    ui.text(im_str!("another node"));
                });

                node.add_input(in_pin1, imnodes::PinShape::Circle, || {
                    ui.text(im_str!("input"));
                });

                node.add_output(id_gen.next_output_pin(), imnodes::PinShape::QuadFilled, || {
                    ui.text(im_str!("output"));
                });
            });

            let link = id_gen.next_link();
            editor.add_link(link, in_pin1, out_pin1);


        });



        window_token.end();
    }

    let window_token = imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_PROPERTIES))
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("properties"));
        window_token.end();
    }

    crate::ui::windows::assets_window::draw_assets_dockspace_and_window(ui, app_state);

    let window_token = imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_DOC_OUTLINE))
        .begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("outline"));
        window_token.end();
    }

    app_state.redock_windows = false;
}
use crate::ui::components::draw_ui_inspector::*;

use crate::app::AppState;
use crate::imgui_support::ImguiManager;
use imgui::im_str;
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImVec2,
};
use std::convert::TryInto;

pub fn draw_2_pane_view(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let main_viewport = imgui::sys::igGetMainViewport();
        let work_pos = (*main_viewport).WorkPos.clone();
        let work_size = (*main_viewport).WorkSize.clone();

        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(
            imgui::sys::ImGuiStyleVar_WindowPadding as _,
            ImVec2::new(0.0, 0.0),
        );

        let root_window_token = imgui::Window::new(im_str!("Root Window"))
            .position([work_pos.x, work_pos.y], imgui::Condition::Always)
            .size([work_size.x, work_size.y], imgui::Condition::Always)
            .flags(
                imgui::WindowFlags::NO_TITLE_BAR
                    | imgui::WindowFlags::NO_COLLAPSE
                    | imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_DOCKING
                    | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                    | imgui::WindowFlags::NO_NAV_FOCUS,
            )
            .draw_background(false)
            .resizable(false)
            .begin(ui);

        if let Some(root_window_token) = root_window_token {
            let id = imgui::Id::from("RootDockspace");
            let root_dockspace_id = unsafe {
                match id {
                    imgui::Id::Int(i) => imgui::sys::igGetIDPtr(i as *const std::os::raw::c_void),
                    imgui::Id::Ptr(p) => imgui::sys::igGetIDPtr(p),
                    imgui::Id::Str(s) => {
                        let start = s.as_ptr() as *const std::os::raw::c_char;
                        let end = start.add(s.len());
                        imgui::sys::igGetIDStrStr(start, end)
                    }
                }
            };

            if imgui::sys::igDockBuilderGetNode(root_dockspace_id) == std::ptr::null_mut() {
                //println!("SET UP DOCK");
                imgui::sys::igDockBuilderRemoveNode(root_dockspace_id);
                imgui::sys::igDockBuilderAddNode(
                    root_dockspace_id,
                    imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace,
                );
                imgui::sys::igDockBuilderSetNodeSize(root_dockspace_id, (*main_viewport).WorkSize);

                let mut right_dockspace_id = 0;
                let mut left_dockspace_id = 0;
                imgui::sys::igDockBuilderSplitNode(
                    root_dockspace_id,
                    imgui::sys::ImGuiDir_Left,
                    0.5,
                    &mut left_dockspace_id as _,
                    &mut right_dockspace_id as _,
                );

                imgui::sys::igDockBuilderDockWindow(
                    im_str!("Prototype").as_ptr(),
                    left_dockspace_id,
                );
                imgui::sys::igDockBuilderDockWindow(
                    im_str!("Instance").as_ptr(),
                    right_dockspace_id,
                );
                imgui::sys::igDockBuilderFinish(root_dockspace_id);
            }

            imgui::sys::igDockSpace(
                root_dockspace_id,
                ImVec2::new(0.0, 0.0),
                0,
                std::ptr::null(),
            );
            root_window_token.end();
        }

        imgui::sys::igPopStyleVar(3);

        //TODO: Uncomment to bring asset browser back
        //draw_asset_browser_dock_space(ui, &mut app_state.test_data_nexdb.db);
        //draw_asset_browser_dock_space_windows(ui, &mut app_state.test_data_nexdb.db);

        let window_token = imgui::Window::new(im_str!("Prototype"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            //draw_inspector_refdb(ui, &mut app_state.test_data_refdb.db, app_state.test_data_refdb.prototype_obj);
            draw_inspector_nexdb(
                ui,
                &mut app_state.test_data_nexdb.db,
                app_state.test_data_nexdb.prototype_obj,
            );
            window_token.end();
        }

        let window_token = imgui::Window::new(im_str!("Instance"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            //draw_inspector_refdb(ui, &mut app_state.test_data_refdb.db, app_state.test_data_refdb.instance_obj);
            draw_inspector_nexdb(
                ui,
                &mut app_state.test_data_nexdb.db,
                app_state.test_data_nexdb.instance_obj,
            );
            window_token.end();
        }
    }
}

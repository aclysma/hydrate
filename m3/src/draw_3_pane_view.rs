use super::draw_ui_inspector::*;
use crate::app::AppState;
use crate::imgui_support::ImguiManager;
use imgui::im_str;
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImVec2,
};
use std::convert::TryInto;


fn draw_asset_browser_dock_space(
    ui: &imgui::Ui,
    db: &mut nexdb::Database,
) -> bool {
    unsafe {
        //let main_viewport = imgui::sys::igGetMainViewport();
        //let work_pos = (*main_viewport).WorkPos.clone();
        //let work_size = (*main_viewport).WorkSize.clone();

        //imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        //imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(
            imgui::sys::ImGuiStyleVar_WindowPadding as _,
            ImVec2::new(0.0, 0.0),
        );

        let asset_browser_window_token = imgui::Window::new(im_str!("Asset Browser"))
            //.position([work_pos.x, work_pos.y], imgui::Condition::Always)
            //.size([work_size.x, work_size.y], imgui::Condition::Always)
            .position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .flags(imgui::WindowFlags::NO_COLLAPSE)
            //.flags(imgui::WindowFlags::NO_TITLE_BAR | imgui::WindowFlags::NO_COLLAPSE | imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_DOCKING | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS)
            //.draw_background(false)
            //.resizable(false)
            .begin(ui);

        // let is_visible = imgui::sys::igIsItemVisible();
        // println!("visible");

        let mut should_show = false;

        let id = imgui::Id::from("AssetBrowserDockSpace");
        let asset_browser_dockspace_id = unsafe {
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

        if let Some(asset_browser_window_token) = asset_browser_window_token {
            should_show = true;

            let window = imgui::sys::igGetCurrentWindow();
            let work_pos = (*window).WorkRect.Min.clone();
            let mut work_size = (*window).WorkRect.Max.clone();
            work_size.x -= work_pos.x;
            work_size.y -= work_pos.y;

            if imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id) == std::ptr::null_mut()
            {
                //println!("SET UP DOCK");
                imgui::sys::igDockBuilderRemoveNode(asset_browser_dockspace_id);
                imgui::sys::igDockBuilderAddNode(
                    asset_browser_dockspace_id,
                    imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking| imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe, // imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace |
                    //      imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoWindowMenuButton |
                    //      imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoCloseButton |
                    //     imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking |
                    //     imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe |
                    //     imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar
                );
                imgui::sys::igDockBuilderSetNodeSize(asset_browser_dockspace_id, work_size);

                let mut right_dockspace_id = 0;
                let mut left_dockspace_id = 0;
                imgui::sys::igDockBuilderSplitNode(
                    asset_browser_dockspace_id,
                    imgui::sys::ImGuiDir_Left,
                    0.5,
                    &mut left_dockspace_id as _,
                    &mut right_dockspace_id as _,
                );

                imgui::sys::igDockBuilderDockWindow(
                    im_str!("AssetTree").as_ptr(),
                    left_dockspace_id,
                );
                imgui::sys::igDockBuilderDockWindow(
                    im_str!("AssetPane").as_ptr(),
                    right_dockspace_id,
                );

                (*imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id)).SharedFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;
                (*imgui::sys::igDockBuilderGetNode(left_dockspace_id)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe| imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;
                (*imgui::sys::igDockBuilderGetNode(right_dockspace_id)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;

                imgui::sys::igDockBuilderFinish(asset_browser_dockspace_id);
            }

            let current_window = imgui::sys::igGetCurrentWindow();
            // println!(
            //     "About to set up dock node {:?}",
            //     (*current_window).DockNodeAsHost
            // );
            //imgui::sys::igDockSpace(asset_browser_dockspace_id, ImVec2::new(0.0, 0.0), imgui::sys::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _, std::ptr::null());
            imgui::sys::igDockSpace(
                asset_browser_dockspace_id,
                ImVec2::new(0.0, 0.0),
                0,
                std::ptr::null(),
            );
            let current_window = imgui::sys::igGetCurrentWindow();
            //println!("Set up dock node {:?}", (*current_window).DockNodeAsHost);
            //asset_browser_window_token.end();
        } else {
            // imgui::sys::igDockSpace(
            //     asset_browser_dockspace_id,
            //     ImVec2::new(0.0, 0.0),
            //     imgui::sys::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _,
            //     std::ptr::null(),
            // );
        }

        //imgui::sys::igPopStyleVar(3);
        imgui::sys::igPopStyleVar(1);

        should_show
    }
}

fn draw_asset_browser_dock_space_windows(
    ui: &imgui::Ui,
    db: &mut nexdb::Database,
) {
    unsafe {
        let window_token = imgui::Window::new(im_str!("AssetTree"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            //draw_inspector(ui, &mut app_state.db, app_state.prototype_obj);
            window_token.end();
        }

        let window_token = imgui::Window::new(im_str!("AssetPane"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            //draw_inspector(ui, &mut app_state.db, app_state.instance_obj);
            window_token.end();
        }
    }
}

fn to_c_id(id: imgui::Id) -> imgui::sys::ImGuiID {
    unsafe {
        match id {
            imgui::Id::Int(i) => imgui::sys::igGetIDPtr(i as *const std::os::raw::c_void),
            imgui::Id::Ptr(p) => imgui::sys::igGetIDPtr(p),
            imgui::Id::Str(s) => {
                let start = s.as_ptr() as *const std::os::raw::c_char;
                let end = start.add(s.len());
                imgui::sys::igGetIDStrStr(start, end)
            }
        }
    }
}

pub fn draw_3_pane_view(
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

                let mut center_dockspace_id = root_dockspace_id;
                let mut left_dockspace_id = 0;
                imgui::sys::igDockBuilderSplitNode(
                    center_dockspace_id,
                    imgui::sys::ImGuiDir_Left,
                    0.2,
                    &mut left_dockspace_id as _,
                    &mut center_dockspace_id as _,
                );

                let mut bottom_dockspace_id = 0u32;
                imgui::sys::igDockBuilderSplitNode(
                    center_dockspace_id,
                    imgui::sys::ImGuiDir_Down,
                    0.2,
                    &mut bottom_dockspace_id as *mut _,
                    &mut center_dockspace_id as *mut _,
                );

                imgui::sys::igDockBuilderDockWindow(
                    im_str!("Demo Window 1").as_ptr(),
                    center_dockspace_id,
                );
                imgui::sys::igDockBuilderDockWindow(
                    im_str!("Demo Window 2").as_ptr(),
                    left_dockspace_id,
                );
                // imgui::sys::igDockBuilderDockWindow(
                //     im_str!("Asset Browser").as_ptr(),
                //     bottom_dockspace_id,
                // );
                imgui::sys::igDockBuilderDockWindow(
                    im_str!("Demo Window 3").as_ptr(),
                    bottom_dockspace_id,
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

        if draw_asset_browser_dock_space(ui, &mut app_state.test_data_nexdb.db) {
            draw_asset_browser_dock_space_windows(ui, &mut app_state.test_data_nexdb.db);
        }


        imgui::Window::new(im_str!("Demo Window 2"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {});

        imgui::Window::new(im_str!("Demo Window 3"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {});

        imgui::Window::new(im_str!("Demo Window 1"))
            //.position([150.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {});
    }
}

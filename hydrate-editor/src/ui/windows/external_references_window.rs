use crate::app_state::AppState;
use imgui::sys as is;
use imgui::sys::ImVec2;
use imgui::ImString;

pub fn _draw_external_references_dockspace(
    _ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let window = imgui::sys::igGetCurrentWindow();
        let mut work_size = (*window).Size;
        // Subtract 22 to account for tabs above. This gets rid of the scroll bar.
        work_size.y -= 22.0;

        // Get the ID for WINDOW_NAME_ASSETS
        let assets_window_id = (*is::igGetCurrentWindow()).ID;
        let root_assets_dockspace_id = is::igDockSpace(
            assets_window_id,
            work_size,
            imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe,
            std::ptr::null_mut(),
        );

        // The first time through, set up the left/right panes of the asset browser so that they are pinned inside the asset browser window
        // and nothing can dock inside them
        if app_state.ui_state.redock_windows {
            let mut assets_main = root_assets_dockspace_id;
            is::igDockBuilderAddNode(
                assets_main,
                is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingOverMe,
            );

            // We hardcode a 1.0 size ratio here because the first run, width will be the window's init size rather than the actual size.
            // The window width is 300, so ratio = 1 results in left pane being 300 pixels wide.
            is::igDockBuilderSetNodeSize(root_assets_dockspace_id, work_size);
            let assets_left = is::igDockBuilderSplitNode(
                assets_main,
                is::ImGuiDir_Left,
                1.0,
                std::ptr::null_mut(),
                &mut assets_main,
            );

            // Assign windows to dock nodes
            is::igDockBuilderDockWindow(
                ImString::new(crate::ui::_WINDOW_NAME_EXTERNAL_REFERENCES_LEFT).as_ptr(),
                assets_left,
            );
            is::igDockBuilderDockWindow(
                ImString::new(crate::ui::_WINDOW_NAME_EXTERNAL_REFERENCES_RIGHT).as_ptr(),
                assets_main,
            );

            // Don't draw tab bars on the left/right panes
            (*imgui::sys::igDockBuilderGetNode(assets_left)).LocalFlags |=
                imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;
            (*imgui::sys::igDockBuilderGetNode(assets_main)).LocalFlags |=
                imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;

            is::igDockBuilderFinish(root_assets_dockspace_id);
        }
    }
}

pub fn _draw_external_references_dockspace_and_window(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    // We set padding to zero when creating the assets window so that the vertical splitter bar
    // will go from top to bottom of the window
    unsafe {
        is::igPushStyleVar_Vec2(is::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));
    }

    let window_token =
        imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_EXTERNAL_REFERENCES))
            // The width of this matters, it sets the initial width of the left column
            .size([300.0, 400.0], imgui::Condition::Once)
            .flags(imgui::WindowFlags::NO_COLLAPSE)
            .begin(ui);

    unsafe {
        is::igPopStyleVar(1);
    }

    if let Some(window_token) = window_token {
        _draw_external_references_dockspace(ui, app_state);

        let inner_window_token = imgui::Window::new(&ImString::new(
            crate::ui::_WINDOW_NAME_EXTERNAL_REFERENCES_LEFT,
        ))
        .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            //assets_window_left(ui, app_state);
            inner_window_token.end();
        }

        let inner_window_token = imgui::Window::new(&ImString::new(
            crate::ui::_WINDOW_NAME_EXTERNAL_REFERENCES_RIGHT,
        ))
        .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            //assets_window_right(ui, app_state);
            inner_window_token.end();
        }

        window_token.end();
    } else {
        //TODO: keepalive the assets dockspace
        println!("KEEPALIVE EXTERNAL ASSETS");
        //unsafe {
        //let id = imgui::Id::from(crate::ui::WINDOW_NAME_EXTERNAL_REFERENCES);
        //let id = is::igGetIDStr(imgui::im_str!("{}", crate::ui::WINDOW_NAME_EXTERNAL_REFERENCES).as_ptr());
        //is::igDockSpace(id, ImVec2::new(100.0, 100.0), is::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _, std::ptr::null_mut());
        //}
    }
}

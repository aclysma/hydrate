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
            is::igDockBuilderDockWindow(ImString::new(crate::ui::WINDOW_NAME_ASSETS_LEFT).as_ptr(), assets_left);
            is::igDockBuilderDockWindow(ImString::new(crate::ui::WINDOW_NAME_ASSETS_RIGHT).as_ptr(), assets_main);

            // Don't draw tab bars on the left/right panes
            (*imgui::sys::igDockBuilderGetNode(assets_left)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;
            (*imgui::sys::igDockBuilderGetNode(assets_main)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;

            is::igDockBuilderFinish(root_assets_dockspace_id);
        }
    }
}

fn default_flags() -> imgui::TreeNodeFlags {
    imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK | imgui::TreeNodeFlags::OPEN_ON_ARROW
}

fn leaf_flags() -> imgui::TreeNodeFlags {
    imgui::TreeNodeFlags::LEAF | default_flags()
}

fn context_menu<F: FnOnce(&imgui::Ui)>(ui: &imgui::Ui, f: F) {
    unsafe {
        if imgui::sys::igBeginPopupContextItem(
            std::ptr::null(),
            imgui::sys::ImGuiPopupFlags_MouseButtonRight as _,
        ) {
            (f)(ui);
            imgui::sys::igEndPopup();
        }
    }
}

pub fn assets_tree_file_system_data_source_loaded(
    ui: &imgui::Ui,
    app_state: &AppState,
    ds: &crate::data_source::FileSystemDataSource,
    loaded_state: &crate::data_source::FileSystemLoadedState
) {
    for file in loaded_state.files() {
        //let id = ImString::new(file.path().file_name().unwrap().to_string_lossy());
        let id = im_str!("\u{e872} {}", file.path().file_name().unwrap().to_string_lossy());
        imgui::TreeNode::new(&id).flags(leaf_flags()).build(ui, || {
            // A single file
        });

        //
        // context_menu(ui, |ui| {
        //
        // });
    }
}

pub fn assets_tree_file_system_data_source(
    ui: &imgui::Ui,
    app_state: &AppState,
    ds: &crate::data_source::FileSystemDataSource
) {
    if let Some(loaded_state) = ds.loaded_state() {
        assets_tree_file_system_data_source_loaded(ui, app_state, ds, loaded_state);
    }
}

pub fn assets_tree(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    //assets_tree_file_system_data_source(ui, app_state, &app_state.file_system_ds);

    let root_path = app_state.file_system_ds.root_path();
    //ui.button(im_str!("\u{fd74} Add"));
    //ui.button(im_str!("\u{e8b1} Add"));
    //ui.button(im_str!("\u{e8b1} Add"));

    let id = im_str!("\u{e916} {}", root_path.to_string_lossy());

    let ds_tree_node = imgui::TreeNode::new(&id).flags(default_flags());
    let token = ds_tree_node.push(ui);

    context_menu(ui, |ui| {
        if app_state.file_system_ds.loaded_state().is_some() {
            if imgui::MenuItem::new(im_str!("Unload")).build(ui) {
                //TODO: Unload
            }
        } else {
            if imgui::MenuItem::new(im_str!("Load")).build(ui) {
                //TODO: Load
            }
        }
    });

    if let Some(token) = token {
        assets_tree_file_system_data_source(ui, app_state, &app_state.file_system_ds);
    }
    //
    //
    // //let id = ImString::new(root_path.to_string_lossy());
    // imgui::TreeNode::new(&id).flags(default_flags()).build(ui, || {
    //
    //     // The data source
    //     assets_tree_file_system_data_source(ui, app_state, &app_state.file_system_ds);
    // });
}

pub fn assets_window_left(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    assets_tree(ui, app_state);
}

pub fn draw_assets_dockspace_and_window(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    // We set padding to zero when creating the assets window so that the vertical splitter bar
    // will go from top to bottom of the window
    unsafe {
        is::igPushStyleVar_Vec2(
            is::ImGuiStyleVar_WindowPadding as _,
            ImVec2::new(0.0, 0.0),
        );
    }

    let window_token = imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_ASSETS))
        // The width of this matters, it sets the initial width of the left column
        .size([300.0, 400.0], imgui::Condition::Once)
        .flags(imgui::WindowFlags::NO_COLLAPSE)
        .begin(ui);

    unsafe {
        is::igPopStyleVar(1);
    }

    if let Some(window_token) = window_token {
        draw_assets_dockspace(ui, app_state);

        let inner_window_token = imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_ASSETS_LEFT))
            .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            assets_window_left(ui, app_state);
            inner_window_token.end();
        }


        let inner_window_token = imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_ASSETS_RIGHT))
            .begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            ui.text(im_str!("assets right"));
            inner_window_token.end();
        }

        window_token.end();
    } else {
        //TODO: keepalive the assets dockspace
    }
}
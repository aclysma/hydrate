use crate::ui::components::draw_ui_inspector::*;
use crate::app::AppState;
use crate::imgui_support::ImguiManager;
use imgui::{im_str, ImStr, ImString};
use imgui::sys::{igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double, ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX, ImVec2};
use std::convert::TryInto;
use std::ffi::CString;
use imgui::sys as is;


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
}

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

fn size_of_button(text: &imgui::ImStr, size: ImVec2) -> ImVec2 {
    unsafe {
        //let style = &(*is::igGetCurrentContext()).Style;
        let style = &(*is::igGetStyle());


        let mut text_size = ImVec2::zero();
        is::igCalcTextSize(&mut text_size, text.as_ptr(), std::ptr::null(), true, 0.0);
        let mut item_size = ImVec2::zero();
        is::igCalcItemSize(&mut item_size, size, text_size.x + style.FramePadding.x * 2.0, text_size.y + style.FramePadding.y * 2.0);
        item_size
    }
}


fn text_centered(text: &imgui::ImStr) {
    unsafe {
        let mut available = ImVec2::zero();
        is::igGetContentRegionAvail(&mut available);
        let mut text_size = ImVec2::zero();
        is::igCalcTextSize(&mut text_size, text.as_ptr(), std::ptr::null(), true, available.x);
        is::igSetCursorPosX(is::igGetCursorPosX() + ((available.x - text_size.x) * 0.5));
        is::igTextWrapped(text.as_ptr());
    }

}

pub fn assets_window_left(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    assets_tree(ui, app_state);
}

pub fn assets_window_right(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    let mut content_available_region = ImVec2::zero();
    unsafe {
        //
        // Draw the top bar
        //
        is::igGetContentRegionAvail(&mut content_available_region);
        is::igBeginChild_Str(im_str!("##AssetBrowserContents").as_ptr(), content_available_region, false, 0);

        is::igGetContentRegionAvail(&mut content_available_region);
        let padding = (*is::igGetStyle()).CellPadding;
        let scroll_bar_width = (*is::igGetStyle()).ScrollbarSize;
        let item_size = 128;
        let mut columns = ((content_available_region.x - scroll_bar_width) as i32 / (item_size + (2.0 * padding.x) as i32));
        columns = columns.max(1);

        ui.button(im_str!("asd1"));
        ui.same_line();
        ui.button(im_str!("asd2"));;
        ui.same_line();
        ui.button(im_str!("asd3"));

        // Determine size of the buttons and spacing between them
        let mut b1 = size_of_button(im_str!("ButtonRight 1"), ImVec2::zero());
        let mut b2 = size_of_button(im_str!("ButtonRight 2"), ImVec2::zero());
        let mut b3 = size_of_button(im_str!("ButtonRight 3"), ImVec2::zero());
        let spacing = (*is::igGetStyle()).ItemSpacing;
        let required_space_for_rhs_buttons = (b1.x + b2.x + b3.x) + (2.0 * spacing.x);

        // Call same_line here so that we can get remaining x space on this line
        ui.same_line();
        is::igGetContentRegionAvail(&mut content_available_region);

        // If there's enough space, draw, otherwise draw a dummy object
        if content_available_region.x > required_space_for_rhs_buttons {
            //is::igSetCursorPosX(is::igGetCursorPosX() + (content_available_region.x - required_space_for_rhs_buttons));
            ui.same_line_with_pos(is::igGetCursorPosX() + (content_available_region.x - required_space_for_rhs_buttons));
            ui.button(im_str!("ButtonRight 1"));
            ui.same_line();
            ui.button(im_str!("ButtonRight 2"));;
            ui.same_line();
            ui.button(im_str!("ButtonRight 3"));
        } else {
            // We called same line above, but there isn't enough room to draw anything. So draw a 0x0 to consume the same_line call
            ui.dummy([0.0, 0.0]);
        }

        //
        // Separator for top menu and grid of assets
        //
        ui.separator();

        //
        // Grid of assets
        //
        is::igGetContentRegionAvail(&mut content_available_region);
        is::igBeginChild_Str(im_str!("##AssetBrowserContentsTable").as_ptr(), content_available_region, false, 0);
        let outer_size = ImVec2::zero();
        let width = 0.0;
        if is::igBeginTable(im_str!("contents").as_ptr(), columns, is::ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX as _, ImVec2::zero(), 0.0) {

            for i in 0..columns {
                is::igTableSetupColumn(im_str!("").as_ptr(), is::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as _, item_size as _, 0);
            }

            for i in 0..200 {
                is::igTableNextColumn();

                is::igGetContentRegionAvail(&mut content_available_region);
                is::igInvisibleButton(im_str!("SomeAsset {}", i).as_ptr(), ImVec2::new(item_size as _, item_size as _), 0 as _);
                let mut min = ImVec2::zero();
                let mut max = ImVec2::zero();
                is::igGetItemRectMin(&mut min);
                is::igGetItemRectMax(&mut max);
                //(*is::igGetWindowDrawList()).
                is::ImDrawList_AddRect(is::igGetWindowDrawList(), min, max, 0xFF333333, 0.0, 0, 2.0);

                text_centered(&im_str!("very_long_file_{}.txt", i));
            }

            is::igEndTable();
        }

        is::igEndChild();
        is::igEndChild();
    }
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
            assets_window_right(ui, app_state);
            inner_window_token.end();
        }

        window_token.end();
    } else {
        //TODO: keepalive the assets dockspace
    }
}
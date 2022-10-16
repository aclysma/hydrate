use crate::app_state::{
    ActiveToolRegion, AppState, AssetBrowserGridState, AssetBrowserState, UiState,
};
use crate::imgui_support::ImguiManager;
use imgui::sys as is;
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX,
    ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_Selected, ImVec2,
};
use imgui::{im_str, ImStr, ImString, TreeNodeFlags};
use nexdb::{HashSet, ObjectId, ObjectLocation, ObjectPath};
use std::convert::TryInto;
use std::ffi::CString;
use std::path::PathBuf;
use uuid::Uuid;

fn default_flags() -> imgui::TreeNodeFlags {
    imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK | imgui::TreeNodeFlags::OPEN_ON_ARROW
}

fn leaf_flags() -> imgui::TreeNodeFlags {
    imgui::TreeNodeFlags::LEAF | default_flags()
}

fn context_menu<F: FnOnce(&imgui::Ui)>(
    ui: &imgui::Ui,
    str_id: Option<&ImStr>,
    f: F,
) {
    let id = if let Some(str_id) = str_id {
        str_id.as_ptr()
    } else {
        std::ptr::null()
    };
    unsafe {
        if imgui::sys::igBeginPopupContextItem(
            id,
            imgui::sys::ImGuiPopupFlags_MouseButtonRight as _,
        ) {
            (f)(ui);
            imgui::sys::igEndPopup();
        }
    }
}

fn try_select_tree_node(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    id: &ImStr,
) {
    if ui.is_item_clicked() && !ui.is_item_toggled_open() {
        ui_state.active_tool_region = Some(ActiveToolRegion::AssetBrowserTree);
        if !ui.io().key_super {
            println!("clear selection");
            ui_state
                .asset_browser_state
                .tree_state
                .selected_items
                .clear();
        }

        ui_state
            .asset_browser_state
            .tree_state
            .selected_items
            .insert(id.to_string());
    }
}

fn try_select_grid_item(
    ui: &imgui::Ui,
    ui_state: &mut UiState,
    items: &[(ObjectId, ObjectLocation)],
    index: usize,
    id: ObjectId,
) {
    let mut grid_state = &mut ui_state.asset_browser_state.grid_state;

    //let is_selected = if grid_state.selected_items.contains(&id) {
    // If the item is already selected, we may be dragging. So more complex logic to determine if user
    // is single-clicking or dragging
    let drag_delta = ui.mouse_drag_delta();
    let is_selected = ui.is_item_hovered()
        && ui.is_mouse_released(imgui::MouseButton::Left)
        && drag_delta[0] < 1.0
        && drag_delta[1] < 1.0;
    //} else {
    // It's not selected, so user isn't dragging. Just look for mouse down
    //    ui.is_item_clicked()
    //};

    if is_selected {
        ui_state.active_tool_region = Some(ActiveToolRegion::AssetBrowserGrid);

        if grid_state.first_selected.is_none() {}

        if ui.io().key_super {
            if grid_state.first_selected.is_none() {
                grid_state.first_selected = Some(id);
            }
            grid_state.last_selected = Some(id);
            grid_state.selected_items.insert(id);
        } else if ui.io().key_shift {
            if grid_state.first_selected.is_none() {
                grid_state.first_selected = Some(id);
            }
            grid_state.last_selected = Some(id);

            let mut index_of_first = items
                .iter()
                .position(|x| Some(x.0) == grid_state.first_selected)
                .unwrap();
            let mut index_of_last = items
                .iter()
                .position(|x| Some(x.0) == grid_state.last_selected)
                .unwrap();

            if index_of_first > index_of_last {
                std::mem::swap(&mut index_of_first, &mut index_of_last);
            }

            grid_state.selected_items.clear();
            for i in index_of_first..=index_of_last {
                grid_state.selected_items.insert(items[i].0);
            }

            //TODO: we need to find range between first/last
            grid_state.selected_items.insert(id);
        } else {
            // clear selection and single-select
            grid_state.first_selected = Some(id);
            grid_state.last_selected = Some(id);
            grid_state.selected_items.clear();
            grid_state.selected_items.insert(id);
        }
    }
}

pub fn assets_tree_file_system_data_source_loaded(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    file_system_package_index: usize,
) {
    /*
    let package = &mut app_state.file_system_packages[file_system_package_index];
    if let Some(data_source) = package.data_source() {
        for (path, file_state) in data_source.file_states() {
            //let id = ImString::new(file.path().file_name().unwrap().to_string_lossy());
            let id = im_str!("\u{e872} {}", path.file_name().unwrap().to_string_lossy());
            // imgui::TreeNode::new(&id).flags(leaf_flags()).build(ui, || {
            //     // A single file
            // });

            let mut flags = leaf_flags();

            let is_selected = app_state.ui_state.asset_browser_state.tree_state.selected_items.contains(&id.to_string());
            if is_selected {
                flags |= TreeNodeFlags::SELECTED;
            }

            let doc_tree_node = imgui::TreeNode::new(&id).flags(flags);
            let token = doc_tree_node.push(ui);

            try_select_tree_node(ui, &mut app_state.ui_state, &id);

            crate::ui::asset_browser_grid_drag_drop::asset_browser_grid_drag_target(ui, &app_state.ui_state.asset_browser_state.grid_state);


            context_menu(ui, Some(&id), |ui| {
                if imgui::MenuItem::new(im_str!("Save")).build(ui) {
                    log::info!("Save {:?}", path);
                }
            });

            // no contents to draw
            drop(token);
        }
    }
    */
}

pub fn assets_tree_file_system_data_source(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    file_system_package_index: usize,
) {
    /*
    let package = &app_state.file_system_packages[file_system_package_index];
    let root_path = package.root_path();

    let id = im_str!("\u{e916} {}", root_path.to_string_lossy());

    let mut flags = default_flags();

    let is_selected = app_state.ui_state.asset_browser_state.tree_state.selected_items.contains(&id.to_string());
    if is_selected {
        flags |= TreeNodeFlags::SELECTED;
    }

    let ds_tree_node = imgui::TreeNode::new(&id).flags(flags);
    let token = ds_tree_node.push(ui);

    try_select_tree_node(ui, &mut app_state.ui_state, &id);
    context_menu(ui, Some(&id), |ui| {
        if package.data_source().is_some() {
            if imgui::MenuItem::new(im_str!("Unload")).build(ui) {
                //TODO: Unload
                log::info!("unload {}", root_path.to_string_lossy());
            }
        } else {
            if imgui::MenuItem::new(im_str!("Load")).build(ui) {
                //TODO: Load
                log::info!("load {}", root_path.to_string_lossy());
            }
        }
    });

    if let Some(token) = token {
        let loaded = package.data_source().is_some();
        if loaded {
            assets_tree_file_system_data_source_loaded(ui, app_state, file_system_package_index);
        }
    }
    */
}

/*
pub fn assets_tree(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    //assets_tree_file_system_data_source(ui, app_state, &app_state.file_system_ds);
    for file_system_package_index in 0..app_state.file_system_packages.len() {
        assets_tree_file_system_data_source(ui, app_state, file_system_package_index);
    }
}

 */

pub fn assets_tree(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    let mut paths: HashSet<ObjectPath> = Default::default();
    for (object_id, object_info) in app_state.db_state.editor_model.root_edit_context().objects() {
        paths.insert(object_info.object_location().path().clone());
    }

    let mut paths: Vec<ObjectPath> = paths.into_iter().collect();
    paths.sort_by(|lhs, rhs| lhs.as_string().cmp(rhs.as_string()));

    for path in paths {
        ui.text(path.as_string());
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
        let root_assets_dockspace_id =
            is::igDockSpace(assets_window_id, work_size, 0, std::ptr::null_mut());

        // The first time through, set up the left/right panes of the asset browser so that they are pinned inside the asset browser window
        // and nothing can dock inside them
        if app_state.ui_state.redock_windows {
            let mut assets_main = root_assets_dockspace_id;
            is::igDockBuilderAddNode(
                assets_main,
                is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking
                    | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe,
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
                ImString::new(crate::ui::WINDOW_NAME_ASSETS_LEFT).as_ptr(),
                assets_left,
            );
            is::igDockBuilderDockWindow(
                ImString::new(crate::ui::WINDOW_NAME_ASSETS_RIGHT).as_ptr(),
                assets_main,
            );

            // Don't draw tab bars on the left/right panes
            (*imgui::sys::igDockBuilderGetNode(assets_left)).LocalFlags |=
                imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;
            (*imgui::sys::igDockBuilderGetNode(assets_main)).LocalFlags |=
                imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar;

            is::igDockBuilderFinish(root_assets_dockspace_id);
        }
    }
}

fn size_of_button(
    text: &imgui::ImStr,
    size: ImVec2,
) -> ImVec2 {
    unsafe {
        //let style = &(*is::igGetCurrentContext()).Style;
        let style = &(*is::igGetStyle());

        let mut text_size = ImVec2::zero();
        is::igCalcTextSize(&mut text_size, text.as_ptr(), std::ptr::null(), true, 0.0);
        let mut item_size = ImVec2::zero();
        is::igCalcItemSize(
            &mut item_size,
            size,
            text_size.x + style.FramePadding.x * 2.0,
            text_size.y + style.FramePadding.y * 2.0,
        );
        item_size
    }
}

fn text_centered(text: &imgui::ImStr) {
    unsafe {
        let mut available = ImVec2::zero();
        is::igGetContentRegionAvail(&mut available);
        let mut text_size = ImVec2::zero();
        is::igCalcTextSize(
            &mut text_size,
            text.as_ptr(),
            std::ptr::null(),
            true,
            available.x,
        );
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

pub fn draw_asset(
    ui: &imgui::Ui,
    app_state: &mut AppState,
    items: &[(ObjectId, ObjectLocation)],
    //name: &ImStr,
    index: usize,
    item_size: u32,
) {
    let id = items[index].0;
    let name = im_str!("{}", items[index].0.as_uuid());

    let stack_token = ui.push_id(&name);

    // Non-active tool
    // let selected_color = if app_state.ui_state.active_tool_region == Some(ActiveToolRegion::AssetBrowserGrid) {
    //     ui.style_color(imgui::StyleColor::ScrollbarGrabActive)
    // } else {
    //     ui.style_color(imgui::StyleColor::ScrollbarGrab)
    // };
    let selected_color = ui.style_color(imgui::StyleColor::Header);

    let draw_list = ui.get_window_draw_list();
    draw_list.channels_split(2, |split| {
        // Draw foreground
        split.set_current(1);

        ui.group(|| {
            let mut content_available_region = ImVec2::zero();
            let content_available_region = ui.content_region_avail();
            ui.invisible_button(&name, [item_size as _, item_size as _]);
            let min = ui.item_rect_min();
            let max = ui.item_rect_max();
            draw_list
                .add_rect(min, max, imgui::ImColor32::from_rgb_f32s(0.2, 0.2, 0.2))
                .build();
            crate::ui::asset_browser_grid_drag_drop::asset_browser_grid_drag_source(
                ui,
                &app_state.ui_state.asset_browser_state.grid_state,
                id,
            );

            text_centered(&name);
        });

        if app_state
            .ui_state
            .asset_browser_state
            .grid_state
            .selected_items
            .contains(&id)
        {
            // Draw background
            split.set_current(0);
            let min = ui.item_rect_min();
            let max = ui.item_rect_max();
            draw_list.add_rect_filled_multicolor(
                min,
                max,
                selected_color,
                selected_color,
                selected_color,
                selected_color,
            );
        }
    });

    try_select_grid_item(ui, &mut app_state.ui_state, items, index, id);

    // if ui.is_item_clicked() {
    //     println!("asset {:?} clicked", name);
    //     app_state.ui_state.active_tool_region = Some(ActiveToolRegion::AssetBrowserGrid);
    // }

    stack_token.end();

    context_menu(ui, Some(&name), |ui| {
        if imgui::MenuItem::new(&im_str!("Save {}", name)).build(ui) {
            log::info!("safe asset {}", &name);
        }
    });
}

pub fn assets_window_right_header(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    ui.button(im_str!("asd1"));
    ui.same_line();
    ui.button(im_str!("asd2"));
    ui.same_line();
    ui.button(im_str!("asd3"));

    // Determine size of the buttons and spacing between them
    let b1 = size_of_button(im_str!("ButtonRight 1"), ImVec2::zero());
    let b2 = size_of_button(im_str!("ButtonRight 2"), ImVec2::zero());
    let b3 = size_of_button(im_str!("ButtonRight 3"), ImVec2::zero());
    let spacing = unsafe { (*is::igGetStyle()).ItemSpacing };
    let required_space_for_rhs_buttons = (b1.x + b2.x + b3.x) + (2.0 * spacing.x);

    // Call same_line here so that we can get remaining x space on this line
    ui.same_line();

    let content_available_region = ui.content_region_avail();

    // If there's enough space, draw, otherwise draw a dummy object
    if content_available_region[0] > required_space_for_rhs_buttons {
        ui.same_line_with_pos(
            ui.cursor_pos()[0] + (content_available_region[0] - required_space_for_rhs_buttons),
        );
        ui.button(im_str!("ButtonRight 1"));
        ui.same_line();
        ui.button(im_str!("ButtonRight 2"));
        ui.same_line();
        ui.button(im_str!("ButtonRight 3"));
    } else {
        // We called same line above, but there isn't enough room to draw anything. So draw a 0x0 to consume the same_line call
        ui.dummy([0.0, 0.0]);
    }
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
        is::igBeginChild_Str(
            im_str!("##AssetBrowserContents").as_ptr(),
            content_available_region,
            false,
            0,
        );

        assets_window_right_header(ui, app_state);

        //
        // Separator for top menu and grid of assets
        //
        ui.separator();

        //
        // Grid of assets
        //

        // Determine number of columns
        is::igGetContentRegionAvail(&mut content_available_region);
        let padding = (*is::igGetStyle()).CellPadding;
        let scroll_bar_width = (*is::igGetStyle()).ScrollbarSize;
        let item_size = 128u32;
        let mut columns = ((content_available_region.x - scroll_bar_width) as i32
            / (item_size as i32 + (2.0 * padding.x) as i32));
        columns = columns.max(1);

        // Set up the table
        is::igGetContentRegionAvail(&mut content_available_region);
        is::igBeginChild_Str(
            im_str!("##AssetBrowserContentsTable").as_ptr(),
            content_available_region,
            false,
            0,
        );
        let outer_size = ImVec2::zero();
        let width = 0.0;
        if is::igBeginTable(
            im_str!("contents").as_ptr(),
            columns,
            is::ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX as _,
            ImVec2::zero(),
            0.0,
        ) {
            for _ in 0..columns {
                is::igTableSetupColumn(
                    im_str!("").as_ptr(),
                    is::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as _,
                    item_size as _,
                    0,
                );
            }

            let mut filtered_objects = Vec::default();

            // mock placeholder
            // for i in 0..200 {
            //     filtered_objects.push((ObjectId(i), PathBuf::from("testpath")));
            // }

            // for file_system_package in &app_state.db_statefile_system_packages {
            //     if let Some(data_source) = file_system_package.data_source() {
            //         for kvp in data_source.object_locations() {
            //             filtered_objects.push((*kvp.0, kvp.1.to_path_buf()));
            //         }
            //     }
            // }

            for (k, v) in app_state.db_state.editor_model.root_edit_context().objects() {
                filtered_objects.push((*k, v.object_location().clone()));
            }

            for i in 0..filtered_objects.len() {
                is::igTableNextColumn();

                let name = im_str!(
                    "{} {}",
                    filtered_objects[i].1.path().as_string(),
                    filtered_objects[i].0.as_uuid()
                );
                draw_asset(ui, app_state, &filtered_objects, i, item_size);
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
        is::igPushStyleVar_Vec2(is::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));
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

        let inner_window_token =
            imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_ASSETS_LEFT)).begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            assets_window_left(ui, app_state);
            inner_window_token.end();
        }

        let inner_window_token =
            imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_ASSETS_RIGHT)).begin(ui);

        if let Some(inner_window_token) = inner_window_token {
            assets_window_right(ui, app_state);
            inner_window_token.end();
        }

        window_token.end();
    } else {
        //TODO: keepalive the assets dockspace
    }
}

use refdb::*;
use crate::app::AppState;
use imgui::sys::ImVec2;
use crate::imgui_support::ImguiManager;
use imgui::im_str;

fn draw_menu_bar(
    ui: &imgui::Ui
) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), || {
            imgui::MenuItem::new(im_str!("New")).build(ui);
            imgui::MenuItem::new(im_str!("Open")).build(ui);
            imgui::MenuItem::new(im_str!("Save")).build(ui);
        });
    });
}

fn draw_property_style<F: FnOnce(&imgui::Ui)>(
    ui: &imgui::Ui,
    property_inherited: bool,
    property_overridden: bool,
    f: F
) {
    let inherited_style_token = if property_inherited {
        Some(ui.push_style_color(imgui::StyleColor::Text, [0.2, 0.2, 0.2, 1.0]))
    } else {
        None
    };

    let overridden_style_token = if property_overridden {
        Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
    } else {
        None
    };

    (f)(ui);
}

fn draw_inspector(
    ui: &imgui::Ui,
    db: &mut ObjectDb,
    object_id: ObjectId,
) {
    ui.text(im_str!("Start"));

    // let header_text = &im_str!("header_text");
    // let content_region = ui.window_content_region_max();
    // let id_token = ui.push_id("some_id");
    // let draw_children = unsafe {
    //     imgui::sys::igCollapsingHeaderTreeNodeFlags(
    //         header_text.as_ptr(),
    //         imgui::sys::ImGuiTreeNodeFlags_DefaultOpen as i32
    //             | imgui::sys::ImGuiTreeNodeFlags_AllowItemOverlap as i32,
    //     )
    // };
    //
    // ui.same_line_with_pos(content_region[0] - 50.0);
    //
    // if ui.small_button(im_str!("Delete")) {
    //     // delete
    // } else if draw_children {
    //     ui.indent();
    //
    //     ui.text(im_str!("child"));
    //
    //     ui.unindent();
    // }
    //
    // id_token.pop();

    let type_id = db.type_id_of_object(object_id);
    let ty = db.object_type(type_id);
    let property_count = ty.properties.len();

    for i in 0..property_count {
        let property_index = PropertyIndex::from_index(i);

        let property_inherited = db.is_property_inherited(object_id, property_index);
        // let style_token = if property_overridden {
        //     Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        // } else {
        //     None
        // };

        let p = &db.object_type(type_id).properties[i];
        let property_im_str = im_str!("{}", &p.name);
        match p.property_type {
            PropertyType::U64 => {
                draw_property_style(ui, property_inherited, false, |ui| {
                    let mut v = db.get_u64(object_id, property_index).unwrap();
                    let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);
                    if modified {
                        db.set_u64(object_id, property_index, v);
                    }
                });
            }
            PropertyType::F32 => {
                draw_property_style(ui, property_inherited, false, |ui| {
                    let mut v = db.get_f32(object_id, property_index).unwrap();
                    let modified = imgui::Drag::new(&property_im_str).build(ui, &mut v);



                    unsafe {
                        if imgui::sys::igBeginPopupContextItem(std::ptr::null(), imgui::sys::ImGuiPopupFlags_MouseButtonRight as _) {
                            if (imgui::MenuItem::new(im_str!("Clear Override")).build(ui)) {
                                db.clear_property_override(object_id, property_index);
                            }

                            if (imgui::MenuItem::new(im_str!("Apply Override")).build(ui)) {
                                db.apply_property_override_to_prototype(object_id, property_index);
                            }

                            imgui::sys::igEndPopup();
                        }
                    }

                    if modified {
                        db.set_f32(object_id, property_index, v);
                    }
                });
            }
            PropertyType::Subobject(_) => {
                ui.text(im_str!("{}", p.name));
                ui.indent();
                let id_token = ui.push_id(&p.name);
                let subobject = db.get_subobject(object_id, property_index).unwrap();
                draw_inspector(ui, db, subobject);
                id_token.pop();
                ui.unindent();
            },
            PropertyType::SubobjectSet(_) => {
                ui.text(im_str!("UNHANDLED SET {}", p.name));
            }
        }
        //drop(style_token);
    }




    // let mut value = 0.0;
    // ui.input_float(im_str!("Value 1"), &mut value).build();
    // ui.input_float(im_str!("Value 2"), &mut value).build();
    // ui.input_float(im_str!("Value 3"), &mut value).build();
    //
    // imgui::Slider::new(im_str!("slider na asdfasdf asdf asdf asd fasdf asdf asdf"))
    //     .range(std::ops::RangeInclusive::new(0.0, 100.0))
    //     .build(ui, &mut value);
    //
    //
    // ui.indent();
    // let g = ui.begin_group();
    // ui.input_float(im_str!("Value 4"), &mut value).build();
    // ui.input_float(im_str!("Value 5"), &mut value).build();
    // g.end();
    // ui.unindent();
    //
    // ui.text(im_str!("End"));
}

fn splitter(vertical: bool, size1: &mut f32, size2: &mut f32) {
    let thickness = 4.0;
    let min_size1 = 10.0;
    let min_size2 = 10.0;
    let long_axis_size = -1.0;

    unsafe {
        let id = imgui::Id::from("splitter");
        let mut bb = imgui::sys::ImRect {
            Min: ImVec2::new(0.0, 0.0),
            Max: ImVec2::new(0.0, 0.0),
        };

        bb.Min = if vertical {
            ImVec2::new(*size1, 0.0)
        } else {
            ImVec2::new(0.0, *size1)
        };

        let window = imgui::sys::igGetCurrentWindow();
        if !window.is_null() {
            bb.Min.x += (*window).DC.CursorPos.x;
            bb.Min.y += (*window).DC.CursorPos.y;
        }

        let x = if vertical {
            ImVec2::new(thickness, long_axis_size)
        } else {
            ImVec2::new(long_axis_size, thickness)
        };
        let mut max_add = ImVec2::zero();
        imgui::sys::igCalcItemSize(&mut max_add as *mut _, x, 0.0, 0.0);
        bb.Max = bb.Min;
        bb.Max.x += max_add.x;
        bb.Max.y += max_add.y;

        let axis = if vertical {
            imgui::sys::ImGuiAxis_ImGuiAxis_X
        } else {
            imgui::sys::ImGuiAxis_ImGuiAxis_Y
        };
        imgui::sys::igSplitterBehavior(bb, to_c_id(id), axis, size1, size2, min_size1, min_size2, 0.0, 0.0);
    }
}

fn draw_asset_browser_splitter(
    ui: &imgui::Ui,
    db: &mut ObjectDb,
) {
    unsafe {
        imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));
    }

    let window_token = imgui::Window::new(im_str!("Asset Browser"))
        .position([550.0, 100.0], imgui::Condition::Once)
        .size([300.0, 400.0], imgui::Condition::Once)
        .begin(ui);

    if let Some(window_token) = window_token {
        unsafe {
            let id = imgui::Id::from(im_str!("splitter"));
            let width = imgui::sys::igGetWindowWidth();
            let mut size1 = 100.0;
            let mut size2 = width - 100.0;
            splitter(true, &mut size1, &mut size2);

            unsafe {
                imgui::sys::igPopStyleVar(1);
            }

            let child1_id = imgui::Id::from(im_str!("Child1"));
            let child1_cid = to_c_id(child1_id);
            imgui::sys::igBeginChildID(
                child1_cid,
                imgui::sys::ImVec2::new(size1, -1.0),
                true,
                0
            );

            ui.text("child1");
            ui.text("child2");
            ui.text("child3");

            imgui::sys::igEndChild();

            ui.same_line();

            let child1_id = imgui::Id::from(im_str!("Child2"));
            let child1_cid = to_c_id(child1_id);
            imgui::sys::igBeginChildID(
                child1_cid,
                imgui::sys::ImVec2::new(size2, -1.0),
                true,
                0
            );

            ui.text("child2");
            ui.text("child3");

            imgui::sys::igEndChild();
        }


        window_token.end();
    }

}

fn draw_asset_browser_dock_space(
    ui: &imgui::Ui,
    db: &mut ObjectDb,
) {
    unsafe {
        //let main_viewport = imgui::sys::igGetMainViewport();
        //let work_pos = (*main_viewport).WorkPos.clone();
        //let work_size = (*main_viewport).WorkSize.clone();

        //imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        //imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));

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
            let window = imgui::sys::igGetCurrentWindow();
            let work_pos = (*window).WorkRect.Min.clone();
            let mut work_size = (*window).WorkRect.Max.clone();
            work_size.x -= work_pos.x;
            work_size.y -= work_pos.y;

            if imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id) == std::ptr::null_mut() {
                println!("SET UP DOCK");
                imgui::sys::igDockBuilderRemoveNode(asset_browser_dockspace_id);
                imgui::sys::igDockBuilderAddNode(
                    asset_browser_dockspace_id,
                    0
                    // imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace |
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
                    &mut right_dockspace_id as _
                );

                imgui::sys::igDockBuilderDockWindow(im_str!("AssetTree").as_ptr(), left_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(im_str!("AssetPane").as_ptr(), right_dockspace_id);

                (*imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id)).SharedFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;
                (*imgui::sys::igDockBuilderGetNode(left_dockspace_id)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe| imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;
                (*imgui::sys::igDockBuilderGetNode(right_dockspace_id)).LocalFlags |= imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe | imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitOther;

                imgui::sys::igDockBuilderFinish(asset_browser_dockspace_id);
            }

            let current_window = imgui::sys::igGetCurrentWindow();
            println!("About to set up dock node {:?}", (*current_window).DockNodeAsHost);
            //imgui::sys::igDockSpace(asset_browser_dockspace_id, ImVec2::new(0.0, 0.0), imgui::sys::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _, std::ptr::null());
            imgui::sys::igDockSpace(asset_browser_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());
            let current_window = imgui::sys::igGetCurrentWindow();
            println!("Set up dock node {:?}", (*current_window).DockNodeAsHost);
            //asset_browser_window_token.end();
        } else {
            imgui::sys::igDockSpace(asset_browser_dockspace_id, ImVec2::new(0.0, 0.0), imgui::sys::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _, std::ptr::null());
        }

        //imgui::sys::igPopStyleVar(3);
        imgui::sys::igPopStyleVar(1);
    }
}

fn draw_asset_browser_dock_space_windows(
    ui: &imgui::Ui,
    db: &mut ObjectDb,
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

fn draw_2_pane_view(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let main_viewport = imgui::sys::igGetMainViewport();
        let work_pos = (*main_viewport).WorkPos.clone();
        let work_size = (*main_viewport).WorkSize.clone();

        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));

        let root_window_token = imgui::Window::new(im_str!("Root Window"))
            .position([work_pos.x, work_pos.y], imgui::Condition::Always)
            .size([work_size.x, work_size.y], imgui::Condition::Always)
            .flags(imgui::WindowFlags::NO_TITLE_BAR | imgui::WindowFlags::NO_COLLAPSE | imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_DOCKING | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS)
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
                println!("SET UP DOCK");
                imgui::sys::igDockBuilderRemoveNode(root_dockspace_id);
                imgui::sys::igDockBuilderAddNode(root_dockspace_id, imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace);
                imgui::sys::igDockBuilderSetNodeSize(root_dockspace_id, (*main_viewport).WorkSize);

                let mut right_dockspace_id = 0;
                let mut left_dockspace_id = 0;
                imgui::sys::igDockBuilderSplitNode(
                    root_dockspace_id,
                    imgui::sys::ImGuiDir_Left,
                    0.5,
                    &mut left_dockspace_id as _,
                    &mut right_dockspace_id as _
                );

                imgui::sys::igDockBuilderDockWindow(im_str!("Prototype").as_ptr(), left_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(im_str!("Instance").as_ptr(), right_dockspace_id);
                imgui::sys::igDockBuilderFinish(root_dockspace_id);
            }

            imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());
            root_window_token.end();
        }

        imgui::sys::igPopStyleVar(3);


        draw_asset_browser_dock_space(ui, &mut app_state.db);


        draw_asset_browser_dock_space_windows(ui, &mut app_state.db);

        let window_token = imgui::Window::new(im_str!("Prototype"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            draw_inspector(ui, &mut app_state.db, app_state.prototype_obj);
            window_token.end();
        }

        let window_token = imgui::Window::new(im_str!("Instance"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            draw_inspector(ui, &mut app_state.db, app_state.instance_obj);
            window_token.end();
        }

    }
}

fn draw_3_pane_view(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let main_viewport = imgui::sys::igGetMainViewport();
        let work_pos = (*main_viewport).WorkPos.clone();
        let work_size = (*main_viewport).WorkSize.clone();

        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));
        let root_window_token = imgui::Window::new(im_str!("Root Window"))
            .position([work_pos.x, work_pos.y], imgui::Condition::Always)
            .size([work_size.x, work_size.y], imgui::Condition::Always)
            .flags(imgui::WindowFlags::NO_TITLE_BAR | imgui::WindowFlags::NO_COLLAPSE | imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_DOCKING | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS)
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
                println!("SET UP DOCK");
                imgui::sys::igDockBuilderRemoveNode(root_dockspace_id);
                imgui::sys::igDockBuilderAddNode(root_dockspace_id, imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace);
                imgui::sys::igDockBuilderSetNodeSize(root_dockspace_id, (*main_viewport).WorkSize);

                let mut center_dockspace_id = root_dockspace_id;
                let mut left_dockspace_id = 0;
                imgui::sys::igDockBuilderSplitNode(
                    center_dockspace_id,
                    imgui::sys::ImGuiDir_Left,
                    0.2,
                    &mut left_dockspace_id as _,
                    &mut center_dockspace_id as _
                );

                let mut bottom_dockspace_id = 0u32;
                imgui::sys::igDockBuilderSplitNode(
                    center_dockspace_id,
                    imgui::sys::ImGuiDir_Down,
                    0.2,
                    &mut bottom_dockspace_id as *mut _,
                    &mut center_dockspace_id as *mut _,
                );

                imgui::sys::igDockBuilderDockWindow(im_str!("Demo Window 1").as_ptr(), center_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(im_str!("Demo Window 2").as_ptr(), left_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(im_str!("Demo Window 3").as_ptr(), bottom_dockspace_id);
                imgui::sys::igDockBuilderFinish(root_dockspace_id);
            }

            imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());

            root_window_token.end();
        }

        imgui::sys::igPopStyleVar(3);


        imgui::Window::new(im_str!("Demo Window 2"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {

            });

        imgui::Window::new(im_str!("Demo Window 3"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {

            });

        imgui::Window::new(im_str!("Demo Window 1"))
            //.position([150.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {

            });
    }
}

pub fn draw_imgui(
    imgui_manager: &ImguiManager,
    app_state: &mut AppState,
) {
    //
    //Draw an inspect window for the example struct
    //
    {
        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            draw_menu_bar(ui);
            //draw_2_pane_view(ui, app_state);
            draw_2_pane_view(ui, app_state);
        });
    }
}

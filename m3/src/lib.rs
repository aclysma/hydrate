mod renderer;
use renderer::Renderer;

mod imgui_support;
use imgui_support::ImguiManager;

mod imgui_themes;

use imgui::sys::ImVec2;

mod test_data;

use refdb::*;
use crate::test_data::TestData;

// This struct is a simple example of something that can be inspected
pub struct AppState {
    pub db: ObjectDb,
    pub prototype_obj: ObjectId,
    pub instance_obj: ObjectId,
}

fn draw_menu_bar(
    ui: &mut imgui::Ui
) {
    ui.main_menu_bar(|| {
        ui.menu(imgui::im_str!("File"), || {
            imgui::MenuItem::new(imgui::im_str!("New")).build(ui);
            imgui::MenuItem::new(imgui::im_str!("Open")).build(ui);
            imgui::MenuItem::new(imgui::im_str!("Save")).build(ui);
        });
    });
}

fn draw_inspector(
    ui: &mut imgui::Ui,
    db: &mut ObjectDb,
    object_id: ObjectId,
) {
    ui.text(imgui::im_str!("Start"));

    // let header_text = &imgui::im_str!("header_text");
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
    // if ui.small_button(imgui::im_str!("Delete")) {
    //     // delete
    // } else if draw_children {
    //     ui.indent();
    //
    //     ui.text(imgui::im_str!("child"));
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
        let p = &db.object_type(type_id).properties[i];
        match p.property_type {
            PropertyType::U64 => {
                let mut v = db.get_u64(object_id, property_index).unwrap() as i32;
                ui.input_int(&imgui::im_str!("{}", &p.name), &mut v).build();
            }
            PropertyType::F32 => {
                let mut v = db.get_f32(object_id, property_index).unwrap();
                ui.input_float(&imgui::im_str!("{}", &p.name), &mut v).build();
            }
            PropertyType::Subobject(_) => {
                ui.text(imgui::im_str!("{}", p.name));
                ui.indent();
                let subobject = db.get_subobject(object_id, property_index).unwrap();
                draw_inspector(ui, db, subobject);
                ui.unindent();
            }
        }
    }




    // let mut value = 0.0;
    // ui.input_float(imgui::im_str!("Value 1"), &mut value).build();
    // ui.input_float(imgui::im_str!("Value 2"), &mut value).build();
    // ui.input_float(imgui::im_str!("Value 3"), &mut value).build();
    //
    // imgui::Slider::new(imgui::im_str!("slider na asdfasdf asdf asdf asd fasdf asdf asdf"))
    //     .range(std::ops::RangeInclusive::new(0.0, 100.0))
    //     .build(ui, &mut value);
    //
    //
    // ui.indent();
    // let g = ui.begin_group();
    // ui.input_float(imgui::im_str!("Value 4"), &mut value).build();
    // ui.input_float(imgui::im_str!("Value 5"), &mut value).build();
    // g.end();
    // ui.unindent();
    //
    // ui.text(imgui::im_str!("End"));
}

fn draw_2_pane_view(
    ui: &mut imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let main_viewport = imgui::sys::igGetMainViewport();
        let work_pos = (*main_viewport).WorkPos.clone();
        let work_size = (*main_viewport).WorkSize.clone();

        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));

        let root_window_token = imgui::Window::new(imgui::im_str!("Root Window"))
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

                imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Prototype").as_ptr(), left_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Instance").as_ptr(), right_dockspace_id);
                imgui::sys::igDockBuilderFinish(root_dockspace_id);
            }

            imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());
            root_window_token.end();
        }

        imgui::sys::igPopStyleVar(3);

        let window_token = imgui::Window::new(imgui::im_str!("Prototype"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .begin(ui);

        if let Some(window_token) = window_token {
            draw_inspector(ui, &mut app_state.db, app_state.prototype_obj);
            window_token.end();
        }

        let window_token = imgui::Window::new(imgui::im_str!("Instance"))
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
    ui: &mut imgui::Ui,
    app_state: &mut AppState,
) {
    unsafe {
        let main_viewport = imgui::sys::igGetMainViewport();
        let work_pos = (*main_viewport).WorkPos.clone();
        let work_size = (*main_viewport).WorkSize.clone();

        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
        imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
        imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));
        let root_window_token = imgui::Window::new(imgui::im_str!("Root Window"))
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

                imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Demo Window 1").as_ptr(), center_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Demo Window 2").as_ptr(), left_dockspace_id);
                imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Demo Window 3").as_ptr(), bottom_dockspace_id);
                imgui::sys::igDockBuilderFinish(root_dockspace_id);
            }

            imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());

            root_window_token.end();
        }

        imgui::sys::igPopStyleVar(3);


        imgui::Window::new(imgui::im_str!("Demo Window 2"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {

            });

        imgui::Window::new(imgui::im_str!("Demo Window 3"))
            //.position([550.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {

            });

        imgui::Window::new(imgui::im_str!("Demo Window 1"))
            //.position([150.0, 100.0], imgui::Condition::Once)
            .size([300.0, 400.0], imgui::Condition::Once)
            .build(ui, || {

            });
    }
}

fn draw_imgui(
    imgui_manager: &ImguiManager,
    app_state: &mut AppState,
) {
    //
    //Draw an inspect window for the example struct
    //
    {
        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            draw_menu_bar(ui);
            draw_2_pane_view(ui, app_state);
        });
    }
}


// Creates a window and runs the event loop.
pub fn run() {
    let test_data = test_data::setup_test_data();

    let mut app_state = AppState {
        db: test_data.db,
        prototype_obj: test_data.prototype_obj,
        instance_obj: test_data.instance_obj
    };

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = winit::dpi::LogicalSize::new(900.0, 600.0);

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("M3")
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    // Initialize imgui
    let imgui_manager = imgui_support::init_imgui_manager(&window);

    // Create the renderer, which will draw to the window
    let renderer = Renderer::new(&window, imgui_manager.font_atlas_texture());

    // Check if there were errors setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    // Start the window event loop. Winit will not return once run is called. We will get notified
    // when important events happen.
    event_loop.run(move |event, _window_target, control_flow| {
        imgui_manager.handle_event(&window, &event);

        match event {
            //
            // Halt if the user requests to close the window
            //
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            //
            // Close if the escape key is hit
            //
            winit::event::Event::WindowEvent {
                event:
                winit::event::WindowEvent::KeyboardInput {
                    input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                },
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            //
            // Request a redraw any time we finish processing events
            //
            winit::event::Event::MainEventsCleared => {
                // Queue a RedrawRequested event.
                window.request_redraw();
            }

            //
            // Redraw
            //
            winit::event::Event::RedrawRequested(_window_id) => {
                imgui_manager.begin_frame(&window);
                draw_imgui(&imgui_manager, &mut app_state);
                imgui_manager.render(&window);
                if let Err(e) =
                renderer.draw(&window, imgui_manager.draw_data(), &app_state)
                {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }

            //
            // Ignore all other events
            //
            _ => {}
        }
    });
}

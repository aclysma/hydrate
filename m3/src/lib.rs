mod renderer;
use renderer::Renderer;

mod imgui_support;
use imgui_support::ImguiManager;
use imgui::sys::ImVec2;

// This struct is a simple example of something that can be inspected
pub struct ExampleInspectTarget {
    x_position: f32,
    y_position: f32,
    radius: f32,
    // String is supported as well
    text: String,
    first_call: bool
}

impl Default for ExampleInspectTarget {
    fn default() -> Self {
        ExampleInspectTarget {
            x_position: 300.0,
            y_position: 250.0,
            radius: 50.0,
            text: "".to_string(),
            first_call: true
        }
    }
}

fn draw_imgui(
    imgui_manager: &ImguiManager,
    example_inspect_target: &mut ExampleInspectTarget,
) {
    //
    //Draw an inspect window for the example struct
    //
    {
        imgui_manager.with_ui(|ui: &mut imgui::Ui| {

            // ui.main_menu_bar(|| {
            //     ui.menu(imgui::im_str!("File"), || {
            //         imgui::MenuItem::new(imgui::im_str!("New")).build(ui);
            //         imgui::MenuItem::new(imgui::im_str!("Open")).build(ui);
            //         imgui::MenuItem::new(imgui::im_str!("Save")).build(ui);
            //     });
            // });

            unsafe {
                // let main_viewport = imgui::sys::igGetMainViewport();
                // let root_dockspace_id = imgui::sys::igDockSpaceOverViewport(main_viewport, 0, std::ptr::null());
                // dbg!(root_dockspace_id);
                //
                // let mut left_dockspace_id = 0;
                // let mut center_dockspace_id = root_dockspace_id;
                //
                // //if example_inspect_target.first_call {
                // if imgui::sys::igDockBuilderGetNode(root_dockspace_id) != std::ptr::null_mut() {
                //     example_inspect_target.first_call = false;
                //
                //     println!("SET UP DOCK");
                //     imgui::sys::igDockBuilderRemoveNode(root_dockspace_id);
                //     imgui::sys::igDockBuilderAddNode(root_dockspace_id, imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace);
                //     imgui::sys::igDockBuilderSetNodeSize(root_dockspace_id, (*main_viewport).WorkSize);
                //
                //     imgui::sys::igDockBuilderSplitNode(
                //         center_dockspace_id,
                //         imgui::sys::ImGuiDir_Left,
                //         0.2,
                //         &mut left_dockspace_id as _,
                //         &mut center_dockspace_id as _
                //     );
                //
                //     let mut bottom_dockspace_id = 0u32;
                //     imgui::sys::igDockBuilderSplitNode(
                //         center_dockspace_id,
                //         imgui::sys::ImGuiDir_Down,
                //         0.2,
                //         &mut bottom_dockspace_id as *mut _,
                //         &mut center_dockspace_id as *mut _,
                //     );
                //
                //     imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Demo Window 2").as_ptr(), left_dockspace_id);
                //     imgui::sys::igDockBuilderDockWindow(imgui::im_str!("Demo Window 3").as_ptr(), bottom_dockspace_id);
                //     dbg!(left_dockspace_id);
                //     dbg!(center_dockspace_id);
                //     dbg!(bottom_dockspace_id);
                //     imgui::sys::igDockBuilderFinish(root_dockspace_id);
                // }


                let main_viewport = imgui::sys::igGetMainViewport();
                let work_pos = (*main_viewport).WorkPos.clone();
                let work_size = (*main_viewport).WorkSize.clone();

                imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
                imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
                imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding as _, ImVec2::new(0.0, 0.0));
                imgui::Window::new(imgui::im_str!("Root Window"))
                    .position([work_pos.x, work_pos.y], imgui::Condition::Always)
                    .size([work_size.x, work_size.y], imgui::Condition::Always)
                    .flags(imgui::WindowFlags::NO_TITLE_BAR | imgui::WindowFlags::NO_COLLAPSE | imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_DOCKING | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS)
                    .draw_background(false)
                    .resizable(false)
                    .build(ui, || {
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




                        imgui::sys::igPopStyleVar(3);
                        if imgui::sys::igDockBuilderGetNode(root_dockspace_id) == std::ptr::null_mut() {
                            example_inspect_target.first_call = false;

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
                            dbg!(left_dockspace_id);
                            dbg!(center_dockspace_id);
                            dbg!(bottom_dockspace_id);
                            dbg!(root_dockspace_id);
                            imgui::sys::igDockBuilderFinish(root_dockspace_id);
                        }




                        imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());
                    });
                //imgui::sys::igPopStyleVar(3);



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

                // imgui::sys::igSetNextWindowPos(main_viewport.WorkPos, imgui::sys::ImGuiCond__ImGuiCond_Always as _, imgui::sys::ImVec2 {
                //     x: 0.0,
                //     y: 0.0
                // });

                //imgui::sys::igSetNextWindowSize(main_viewport.WorkSize, imgui::sys::ImGuiCond__ImGuiCond_Always as _);

                //imgui::sys::igSetNextWindowViewport(main_viewport.ID.clone());

                //imgui::sys::igPushStyleVarFloat(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);

                // let work_pos = (*main_viewport).WorkPos.clone();
                // let work_size = (*main_viewport).WorkSize.clone();
                //
                // imgui::Window::new(imgui::im_str!("Root Window"))
                //     .position([work_pos.x, work_pos.y], imgui::Condition::Always)
                //     .size([work_size.x, work_size.y], imgui::Condition::Always)
                //     .no_decoration()
                //     .draw_background(false)
                //     .resizable(false)
                //     .build(ui, || {
                //
                //         let root_dockspace = imgui::Id::from("RootDockspace");
                //         let root_dockspace_id = unsafe {
                //             match root_dockspace {
                //                 imgui::Id::Int(i) => imgui::sys::igGetIDPtr(i as *const std::os::raw::c_void),
                //                 imgui::Id::Ptr(p) => imgui::sys::igGetIDPtr(p),
                //                 imgui::Id::Str(s) => {
                //                     let start = s.as_ptr() as *const std::os::raw::c_char;
                //                     let end = start.add(s.len());
                //                     imgui::sys::igGetIDStrStr(start, end)
                //                 }
                //             }
                //         };
                //         imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());
                //
                //         imgui::Window::new(imgui::im_str!("Demo Window 3"))
                //             .position([550.0, 100.0], imgui::Condition::Once)
                //             .size([300.0, 400.0], imgui::Condition::Once)
                //             .build(ui, || {
                //
                //             });
                //
                //         imgui::Window::new(imgui::im_str!("Demo Window 1"))
                //             .position([150.0, 100.0], imgui::Condition::Once)
                //             .size([300.0, 400.0], imgui::Condition::Once)
                //             .build(ui, || {
                //
                //             });
                //
                //         imgui::Window::new(imgui::im_str!("Demo Window 2"))
                //             .position([550.0, 100.0], imgui::Condition::Once)
                //             .size([300.0, 400.0], imgui::Condition::Once)
                //             .build(ui, || {
                //
                //             });
                //
                //     });

            }






            // let dockspace_id = unsafe {
            //     let g = ui.begin_group();
            //     //imgui::sys::igDockSpace(root_dockspace_id, ImVec2::new(0.0, 0.0), 0, std::ptr::null());
            //     g.end();
            //     //let viewport = imgui::sys::igGetMainViewport();
            //     //imgui::sys::igDockSpaceOverViewport(viewport, )
            // };



            // imgui::Window::new(imgui::im_str!("Demo Window 1"))
            //     .position([550.0, 100.0], imgui::Condition::Once)
            //     .size([300.0, 400.0], imgui::Condition::Once)
            //     .build(ui, || {
            //
            //     });

            //imgui::sys::igDock





            // imgui::Window::new(imgui::im_str!("Inspect Demo"))
            //     .position([550.0, 100.0], imgui::Condition::Once)
            //     .size([300.0, 400.0], imgui::Condition::Once)
            //     .build(ui, || {
            //         // // Add read-only widgets. We pass a slice of refs. Using a slice means we
            //         // // can implement multiple selection
            //         // let selected = vec![&*example_inspect_target];
            //         // <ExampleInspectTarget as imgui_inspect::InspectRenderStruct<
            //         //     ExampleInspectTarget,
            //         // >>::render(
            //         //     &selected,
            //         //     "Example Struct - Read Only",
            //         //     ui,
            //         //     &InspectArgsStruct::default(),
            //         // );
            //         //
            //         // // Now add writable UI widgets. This again takes a slice to handle multiple
            //         // // selection
            //         // let mut selected_mut = vec![example_inspect_target];
            //         // <ExampleInspectTarget as imgui_inspect::InspectRenderStruct<
            //         //     ExampleInspectTarget,
            //         // >>::render_mut(
            //         //     &mut selected_mut,
            //         //     "Example Struct - Writable",
            //         //     ui,
            //         //     &InspectArgsStruct::default(),
            //         // );
            //     });





        });
    }
}

// Creates a window and runs the event loop.
pub fn run() {
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

    // This is the thing we will inspect
    let mut example_inspect_target = ExampleInspectTarget::default();

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
                draw_imgui(&imgui_manager, &mut example_inspect_target);
                imgui_manager.render(&window);
                if let Err(e) =
                renderer.draw(&window, imgui_manager.draw_data(), &example_inspect_target)
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

mod renderer;
use renderer::Renderer;

mod imgui_support;
use imgui_support::ImguiManager;

mod imgui_themes;

mod test_data;

mod draw_ui;
mod draw_ui2;

mod app;
use app::AppState;

use refdb::*;
use crate::test_data::TestData;


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
                draw_ui::draw_imgui(&imgui_manager, &mut app_state);
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

mod renderer;

use std::path::PathBuf;
use renderer::Renderer;

use imgui_support::ImguiManager;

mod data_source;
mod test_data;
mod imgui_support;
mod ui;

mod app;
use app::AppState;
use nexdb::{DataStorageJsonSingleFile, SchemaCacheSingleFile};
use ui::draw_ui;

// Creates a window and runs the event loop.
pub fn run() {
    let test_data_nexdb = test_data::TestData::load_or_init_empty();

    let mut app_state = AppState::new(test_data_nexdb);

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = winit::dpi::LogicalSize::new(1800.0, 1000.0);

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

    let ds_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/schema"));
    let ds = crate::data_source::FileSystemDataSource::new(ds_path);

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
            } => {
                app_state.test_data_nexdb.save();
                //save_state(&app_state.test_data_nexdb.db);
                *control_flow = winit::event_loop::ControlFlow::Exit
            }

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
            } => {
                app_state.test_data_nexdb.save();
                //save_state(&app_state.test_data_nexdb.db);
                *control_flow = winit::event_loop::ControlFlow::Exit
            }

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
                if let Err(e) = renderer.draw(&window, imgui_manager.draw_data(), &app_state) {
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

mod renderer;

use imgui::Image;
use renderer::Renderer;

mod db_state;
mod imgui_support;
//mod builders;
//mod importers;
mod ui;

mod app_state;
use app_state::AppState;
use hydrate_model::SchemaLinker;

mod ui_state;
use crate::app_state::QueuedActions;
use ui::draw_ui;
pub use crate::db_state::DbState;
use hydrate_pipeline::{AssetEngine, AssetEngineBuilder, BuilderRegistry, BuildJobs, ImporterRegistry, ImportJobs};
use hydrate_plugins::{BlenderMaterialAssetPlugin, GlslAssetPlugin, ImageAssetPlugin, SimpleDataAssetPlugin};

// pub fn run() {
//     let mut linker = SchemaLinker::default();
//
//     let mut asset_engine_builder = AssetEngineBuilder::new()
//         .register_plugin::<ImageAssetPlugin>(&mut linker)
//         .register_plugin::<BlenderMaterialAssetPlugin>(&mut linker)
//         .register_plugin::<GlslAssetPlugin>(&mut linker)
//         .register_plugin::<SimpleDataAssetPlugin>(&mut linker);
//
//     let db_state = DbState::load_or_init_empty(linker);
//     let asset_engine = asset_engine_builder.build(&db_state.editor_model, DbState::import_data_source_path(), DbState::build_data_source_path());
//
//     ui_loop(db_state, asset_engine);
// }

// Creates a window and runs the event loop.
pub fn run(db_state: DbState, asset_engine: AssetEngine) {
    let mut app_state = AppState::new(db_state, asset_engine);

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = winit::dpi::LogicalSize::new(1800.0, 1000.0);

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Prototype")
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    // Initialize imgui
    let imgui_manager = imgui_support::init_imgui_manager(&window);
    let mut imnodes_example_editor = imgui_manager.new_imnodes_editor();

    // Create the renderer, which will draw to the window
    let renderer = Renderer::new(&window, imgui_manager.font_atlas_texture());

    // Check if there were errors setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    // Winit gives us files one at a time
    let mut dropped_files = Vec::default();

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
                app_state.action_queue.queue_action(QueuedActions::Quit);
                //app_state.db_state.save();
                //save_state(&app_state.test_data_nexdb.db);
                //*control_flow = winit::event_loop::ControlFlow::Exit
            }

            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::DroppedFile(dropped_file),
                ..
            } => {
                log::info!("Dropped file {:?}", dropped_file);
                dropped_files.push(dropped_file);
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
                //app_state.db_state.save();
                app_state.action_queue.queue_action(QueuedActions::Quit);
                //save_state(&app_state.test_data_nexdb.db);
                //*control_flow = winit::event_loop::ControlFlow::Exit
            }

            //
            // Request a redraw any time we finish processing events
            //
            winit::event::Event::MainEventsCleared => {
                if !dropped_files.is_empty() {
                    // Send files to app queue, clear the buffer
                    let mut dropped = Vec::default();
                    std::mem::swap(&mut dropped, &mut dropped_files);
                    app_state
                        .action_queue
                        .queue_action(QueuedActions::HandleDroppedFiles(dropped))
                }

                app_state.process_queued_actions();
                if app_state.ready_to_quit() {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                } else {
                    // Queue a RedrawRequested event.
                    window.request_redraw();
                }
            }

            //
            // Redraw
            //
            winit::event::Event::RedrawRequested(_window_id) => {
                imgui_manager.begin_frame(&window);
                app_state.asset_engine.update(&app_state.db_state.editor_model);
                draw_ui::draw_imgui(&imgui_manager, &mut imnodes_example_editor, &mut app_state);
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

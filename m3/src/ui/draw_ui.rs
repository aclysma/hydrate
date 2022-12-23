use crate::app_state::{ActionQueueSender, AppState, ModalAction, ModalActionControlFlow, QueuedActions};
use crate::imgui_support::ImguiManager;
use imgui::{im_str, ImString, PopupModal, Ui};
use imnodes::Context;
use crate::db_state::DbState;
use crate::ui_state::UiState;

pub fn draw_properties_window(
    ui: &imgui::Ui,
    _imnodes_editor: &mut imnodes::EditorContext,
    app_state: &mut AppState,
) {
    let window_token =
        imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_PROPERTIES)).begin(ui);

    if let Some(window_token) = window_token {
        crate::ui::windows::properties_window::draw_properties_window(ui, app_state);

        window_token.end();
    }
}

pub fn draw_outline_window(
    ui: &imgui::Ui,
    _imnodes_editor: &mut imnodes::EditorContext,
    _app_state: &mut AppState,
) {

    let window_token =
        imgui::Window::new(&ImString::new(crate::ui::WINDOW_NAME_DOC_OUTLINE)).begin(ui);

    if let Some(window_token) = window_token {
        ui.text(im_str!("outline"));
        window_token.end();
    }
}

pub fn draw_imgui(
    imgui_manager: &ImguiManager,
    imnodes_editor: &mut imnodes::EditorContext,
    app_state: &mut AppState,
) {
    //
    //Draw an inspect window for the example struct
    //
    {
        imgui_manager.with_ui(|ui, imnodes_context| {
            //
            // Dockspace needs to be set up early in draw process. We may decide to reset layout here.
            //
            crate::ui::views::draw_editor_view::draw_dockspace(ui, imnodes_editor, app_state);


            //
            // Properties window
            //
            draw_properties_window(ui, imnodes_editor, app_state);

            //
            // Asset Browser
            //

            //
            // Asset Browser
            //
            crate::ui::windows::assets_window::draw_assets_dockspace_and_window(ui, app_state);
            //crate::ui::windows::external_references_window::draw_external_references_dockspace_and_window(ui, app_state);


            //
            // Outline
            //
            draw_outline_window(ui, imnodes_editor, app_state);

            //
            // Documents?
            //

            //
            // Top Menu Bar
            //
            super::menu_bar::draw_menu_bar(ui, app_state);

            //
            // Non-Modal Dialogs
            //
            if app_state.ui_state.show_imgui_demo_window {
                unsafe {
                    imgui::sys::igShowDemoWindow(&mut app_state.ui_state.show_imgui_demo_window);
                }
            }

            //
            // Modal Dialogs
            //
            if let Some(modal_action) = &mut app_state.modal_action {
                let control_flow = modal_action.draw_imgui(
                    ui,
                    imnodes_context,
                    &mut app_state.db_state,
                    &mut app_state.ui_state,
                    app_state.action_queue.sender()
                );
                if control_flow == ModalActionControlFlow::End {
                    app_state.modal_action = None;
                }
            }

            //
            // End of frame state cleanup
            //
            app_state.ui_state.redock_windows = false;
        });
    }
}

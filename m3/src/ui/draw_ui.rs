use crate::ui::components::draw_ui_inspector::*;
use crate::app_state::AppState;
use crate::imgui_support::ImguiManager;
use imgui::im_str;
use imgui::sys::{
    igDragFloat, igDragScalar, igInputDouble, ImGuiDataType__ImGuiDataType_Double,
    ImGuiInputTextFlags__ImGuiInputTextFlags_None, ImVec2,
};
use std::convert::TryInto;
use crate::ui::views::draw_editor_view::draw_dockspace;

fn draw_menu_bar(
    ui: &imgui::Ui,
    app_state: &mut AppState,
) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), || {
            imgui::MenuItem::new(im_str!("New")).build(ui);
            imgui::MenuItem::new(im_str!("Open")).build(ui);
            imgui::MenuItem::new(im_str!("Save")).build(ui);
            if imgui::MenuItem::new(im_str!("Reset Window Layout")).build(ui) {
                app_state.ui_state.redock_windows = true;
            }
        });
        ui.menu(im_str!("Edit"), || {
            if imgui::MenuItem::new(im_str!("Undo")).build(ui) {
                if let Some(undo_step) = app_state.undo_queue.pop() {
                    undo_step.revert_diff.apply(app_state.db_state.db.data_set_mut());
                }
            }
        });
        ui.menu(im_str!("Debug"), || {
            if imgui::MenuItem::new(im_str!("Toggle ImGui Demo Window")).build(ui) {
                app_state.ui_state.show_imgui_demo_window = !app_state.ui_state.show_imgui_demo_window;
            }
        })
    });
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
            crate::ui::views::draw_editor_view::draw_dockspace(ui, imnodes_editor, app_state);
            //crate::ui::views::draw_assets_view::draw_view(ui, app_state);
            draw_menu_bar(ui, app_state);
            //crate::ui::views::draw_2_pane_view::draw_2_pane_view(ui, app_state);
            //crate::ui::views::draw_3_pane_view::draw_3_pane_view(ui, app_state);

            if app_state.ui_state.show_imgui_demo_window {
                unsafe {
                    imgui::sys::igShowDemoWindow(&mut app_state.ui_state.show_imgui_demo_window);
                }
            }
        });
    }
}

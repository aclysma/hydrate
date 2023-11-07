pub mod asset_browser_grid_drag_drop;
pub mod components;
pub mod draw_ui;
mod menu_bar;
pub mod modals;
pub mod views;
pub mod windows;

const WINDOW_NAME_DOC_OUTLINE: &str = "DocumentOutlineWindow";
//const WINDOW_NAME_DOC_CONTENTS: &str = "DocumentContents";
const WINDOW_NAME_PROPERTIES: &str = "PropertiesWindow";
const WINDOW_NAME_ASSETS: &str = "AssetsWindow";
const WINDOW_NAME_ASSETS_LEFT: &str = "AssetsWindowLeft";
const WINDOW_NAME_ASSETS_RIGHT: &str = "AssetsWindowRight";
const WINDOW_NAME_EXTERNAL_REFERENCES: &str = "ExternalReferencesWindow";
const _WINDOW_NAME_EXTERNAL_REFERENCES_LEFT: &str = "ExternalReferencesWindowLeft";
const _WINDOW_NAME_EXTERNAL_REFERENCES_RIGHT: &str = "ExternalReferencesWindowRight";

struct ImguiDisableHelper {
    is_disabled: bool
}

impl ImguiDisableHelper {
    pub fn new(is_disabled: bool) -> Self {
        if is_disabled {
            unsafe {
                imgui::sys::igPushItemFlag(imgui::sys::ImGuiItemFlags__ImGuiItemFlags_Disabled as _, true);
            }
        }

        ImguiDisableHelper {
            is_disabled
        }
    }
}

impl Default for ImguiDisableHelper {
    fn default() -> Self {
        unsafe {
            imgui::sys::igPushItemFlag(imgui::sys::ImGuiItemFlags__ImGuiItemFlags_Disabled as _, true);
            ImguiDisableHelper {
                is_disabled: true
            }
        }
    }
}

impl Drop for ImguiDisableHelper {
    fn drop(&mut self) {
        if self.is_disabled {
            unsafe {
                imgui::sys::igPopItemFlag();
            }
        }
    }
}
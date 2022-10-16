use crate::db_state::DbState;
use nexdb::{HashSet, ObjectId};

#[derive(PartialEq)]
pub enum ActiveToolRegion {
    AssetBrowserTree,
    AssetBrowserGrid,
}

#[derive(Default)]
pub struct AssetBrowserTreeState {
    pub selected_items: HashSet<String>,
}

#[derive(Default)]
pub struct AssetBrowserGridState {
    pub selected_items: HashSet<ObjectId>,
    pub first_selected: Option<ObjectId>,
    pub last_selected: Option<ObjectId>,
}

#[derive(Default)]
pub struct AssetBrowserState {
    pub tree_state: AssetBrowserTreeState,
    pub grid_state: AssetBrowserGridState,
}

pub struct UiState {
    pub active_tool_region: Option<ActiveToolRegion>,
    pub asset_browser_state: AssetBrowserState,

    pub redock_windows: bool,
    pub show_imgui_demo_window: bool,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            active_tool_region: None,
            asset_browser_state: Default::default(),

            redock_windows: true,
            show_imgui_demo_window: false,
        }
    }
}

// This struct is a simple example of something that can be inspected
pub struct AppState {
    //pub file_system_packages: Vec<FileSystemPackage>,
    pub db_state: DbState,
    pub ui_state: UiState,
}

impl AppState {
    pub fn new(db_state: DbState) -> Self {
        AppState {
            db_state,
            ui_state: UiState::default(),
        }
    }
}

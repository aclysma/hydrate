use hydrate_model::{HashSet, ObjectId, ObjectLocation};

#[derive(PartialEq)]
pub enum ActiveToolRegion {
    AssetBrowserTree,
    AssetBrowserGrid,
}

#[derive(Default)]
pub struct AssetBrowserTreeState {
    pub selected_items: HashSet<ObjectLocation>,
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

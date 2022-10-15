use nexdb::{DeferredTransaction, HashSet, ObjectId, TransactionDiffSet};
use crate::data_source::{FileSystemPackage};
use crate::db_state::DbState;

#[derive(PartialEq)]
pub enum ActiveToolRegion {
    AssetBrowserTree,
    AssetBrowserGrid
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
            show_imgui_demo_window: false
        }
    }
}

pub struct DeferredTransactionState {
    key: String,
    transaction: DeferredTransaction
}

// This struct is a simple example of something that can be inspected
pub struct AppState {
    pub file_system_packages: Vec<FileSystemPackage>,
    pub db_state: DbState,
    pub ui_state: UiState,
    pub deferred_transaction: Option<DeferredTransactionState>,
    pub undo_queue: Vec<TransactionDiffSet>,
}

impl AppState {
    pub fn new(file_system_packages: Vec<FileSystemPackage>, test_data: DbState) -> Self {
        AppState {
            file_system_packages,
            db_state: test_data,
            ui_state: UiState::default(),
            deferred_transaction: None,
            undo_queue: Default::default()
        }
    }

    pub fn give_deferred_transaction(&mut self, key: String, transaction: DeferredTransaction) {
        self.deferred_transaction = Some(DeferredTransactionState {
            key,
            transaction
        })
    }

    pub fn try_resume_transaction(&mut self, key: &str) -> Option<DeferredTransaction> {
        if let Some(deferred_transaction) = self.deferred_transaction.take() {
            if deferred_transaction.key == key {
                return Some(deferred_transaction.transaction)
            }
        }

        None
    }
}
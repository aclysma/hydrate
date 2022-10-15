use std::collections::VecDeque;
use std::sync::Arc;
use nexdb::{DeferredTransaction, HashSet, ObjectId, DataSetDiffSet};
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

pub struct AppTransaction {

}

impl AppTransaction {

    // Writes data to the world without an undo step. The transaction can be cancelled to return
    // the world to the state when the transaction began.
    // update
    //
    //
    // Commits the transaction, writing an undo step
    // commit
    //
    //
    // Reverts the changes that were made in this transaction without writing undo information
    // cancel
    //
    //
}


// pub struct TransactionManager {
//     // Editor transaction will enqueue diffs here to be applied to the world. These are drained
//     // each frame, applied to the world state, and possibly inserted into the undo queue
//     diffs_pending_apply: Vec<TransactionDiffsPendingApply>,
//
//     // Undo/redo steps. Each slot in the chain contains diffs to go forward/backward in the
//     // chain.
//     undo_chain: VecDeque<Arc<TransactionDiffSet>>,
//     undo_chain_position: usize,
//
//     // If a transaction is in progress, the data required to identify it and commit it is
//     // stored here. The ID is used to determine if a transaction provided by downstream code
//     // is the same as the one that's currently in progress. If it isn't the same, we commit
//     // the old transaction and accept the new one. This inserts a new entry in the undo
//     // chain
//     current_transaction_info: Option<CurrentTransactionInfo>,
// }



// This struct is a simple example of something that can be inspected
pub struct AppState {
    pub file_system_packages: Vec<FileSystemPackage>,
    pub db_state: DbState,
    pub ui_state: UiState,
    pub deferred_transaction: Option<DeferredTransactionState>,
    pub undo_queue: Vec<DataSetDiffSet>,
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
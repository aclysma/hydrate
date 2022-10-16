use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use crate::db_state::DbState;
use nexdb::{HashSet, ObjectId, ObjectPath};

#[derive(PartialEq)]
pub enum ActiveToolRegion {
    AssetBrowserTree,
    AssetBrowserGrid,
}

#[derive(Default)]
pub struct AssetBrowserTreeState {
    pub selected_items: HashSet<ObjectPath>,
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

#[derive(Clone, Debug)]
pub enum QueuedActions {
    SaveAll,
    RevertAll,
    Undo,
    Redo,
    Quit,
    //RevertAll,
    //ResetWindowLayout,
    //SelectObjectsInAssetBrowser(Vec<ObjectId>)
}

pub struct ActionQueueSenderInner {
    action_queue_tx: Sender<QueuedActions>,
}

#[derive(Clone)]
pub struct ActionQueueSender {
    inner: Arc<ActionQueueSenderInner>
}

impl ActionQueueSender {
    pub fn queue_action(&self, action: QueuedActions) {
        self.inner.action_queue_tx.send(action).unwrap();
    }
}

pub struct ActionQueueReceiver {
    sender: ActionQueueSender,
    action_queue_tx: Sender<QueuedActions>,
    action_queue_rx: Receiver<QueuedActions>,
}

impl Default for ActionQueueReceiver {
    fn default() -> Self {
        let (action_queue_tx, action_queue_rx) = mpsc::channel();

        let sender_inner = ActionQueueSenderInner {
            action_queue_tx: action_queue_tx.clone()
        };

        let sender = ActionQueueSender {
            inner: Arc::new(sender_inner)
        };


        ActionQueueReceiver {
            sender,
            action_queue_tx,
            action_queue_rx
        }
    }
}

impl ActionQueueReceiver {
    pub fn sender(&self) -> ActionQueueSender {
        self.sender.clone()
    }
    pub fn queue_action(&self, action: QueuedActions) {
        self.action_queue_tx.send(action).unwrap();
    }
}

// This struct is a simple example of something that can be inspected
pub struct AppState {
    //pub file_system_packages: Vec<FileSystemPackage>,
    pub db_state: DbState,
    pub ui_state: UiState,
    pub action_queue: ActionQueueReceiver,
    ready_to_quit: bool,
}

impl AppState {
    pub fn new(db_state: DbState) -> Self {

        AppState {
            db_state,
            ui_state: UiState::default(),
            action_queue: Default::default(),
            ready_to_quit: false
        }
    }

    pub fn process_queued_actions(&mut self) {
        while let Ok(queued_action) = self.action_queue.action_queue_rx.try_recv() {
            match queued_action {
                QueuedActions::SaveAll => self.db_state.editor_model.save_root_edit_context(),
                QueuedActions::RevertAll => self.db_state.editor_model.revert_root_edit_context(),
                QueuedActions::Undo => self.db_state.editor_model.undo(),
                QueuedActions::Redo => self.db_state.editor_model.redo(),
                QueuedActions::Quit => self.ready_to_quit = true
            }
        }
    }

    // Set by sending Quit message to action queue. Window loop will observe this and terminate
    // application. We assume by this point we've already saved/confirmed with user.
    pub fn ready_to_quit(&self) -> bool {
        self.ready_to_quit
    }
}

use crate::db_state::DbState;
use crate::imgui_support::ImguiManager;
use crate::ui::modals::{ConfirmQuitWithoutSavingModal, ImportFilesModal};
use crate::ui_state::UiState;
use imgui::sys::{ImGuiCond, ImVec2};
use imgui::PopupModal;
use imnodes::editor;
use nexdb::{EndContextBehavior, HashSet, ObjectId, ObjectLocation, ObjectPath};
use std::fmt::Formatter;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

#[derive(Debug)]
pub enum QueuedActions {
    SaveAll,
    RevertAll,
    Undo,
    Redo,
    Quit,
    QuitNoConfirm,
    HandleDroppedFiles(Vec<PathBuf>),
    TryBeginModalAction(Box<ModalAction>),
    MoveObjects(Vec<ObjectId>, ObjectLocation),
    //RevertAll,
    //ResetWindowLayout,
    //SelectObjectsInAssetBrowser(Vec<ObjectId>)
}

pub struct ActionQueueSenderInner {
    action_queue_tx: Sender<QueuedActions>,
}

#[derive(Clone)]
pub struct ActionQueueSender {
    inner: Arc<ActionQueueSenderInner>,
}

impl ActionQueueSender {
    pub fn queue_action(
        &self,
        action: QueuedActions,
    ) {
        self.inner.action_queue_tx.send(action).unwrap();
    }

    // shorthand for a common action
    pub fn try_set_modal_action<T: ModalAction + 'static>(
        &self,
        action: T,
    ) {
        self.queue_action(QueuedActions::TryBeginModalAction(Box::new(action)))
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
            action_queue_tx: action_queue_tx.clone(),
        };

        let sender = ActionQueueSender {
            inner: Arc::new(sender_inner),
        };

        ActionQueueReceiver {
            sender,
            action_queue_tx,
            action_queue_rx,
        }
    }
}

impl ActionQueueReceiver {
    pub fn sender(&self) -> ActionQueueSender {
        self.sender.clone()
    }

    pub fn queue_action(
        &self,
        action: QueuedActions,
    ) {
        self.action_queue_tx.send(action).unwrap();
    }

    // shorthand for a common action
    pub fn try_set_modal_action<T: ModalAction + 'static>(
        &self,
        action: T,
    ) {
        self.queue_action(QueuedActions::TryBeginModalAction(Box::new(action)))
    }
}

#[derive(PartialEq)]
pub enum ModalActionControlFlow {
    Continue,
    End,
}

pub trait ModalAction {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow;
}

impl std::fmt::Debug for ModalAction {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ModalAction").finish()
    }
}

// This struct is a simple example of something that can be inspected
pub struct AppState {
    //pub file_system_packages: Vec<FileSystemPackage>,
    pub db_state: DbState,
    pub ui_state: UiState,
    pub action_queue: ActionQueueReceiver,
    ready_to_quit: bool,
    pub modal_action: Option<Box<ModalAction>>,
}

impl AppState {
    pub fn new(db_state: DbState) -> Self {
        AppState {
            db_state,
            ui_state: UiState::default(),
            action_queue: Default::default(),
            ready_to_quit: false,
            modal_action: None,
        }
    }

    fn try_set_modal_action<T: ModalAction + 'static>(
        &mut self,
        action: T,
    ) {
        if self.modal_action.is_none() {
            self.modal_action = Some(Box::new(action))
        }
    }

    pub fn process_queued_actions(&mut self) {
        while let Ok(queued_action) = self.action_queue.action_queue_rx.try_recv() {
            match queued_action {
                QueuedActions::SaveAll => self.db_state.editor_model.save_root_edit_context(),
                QueuedActions::RevertAll => self.db_state.editor_model.revert_root_edit_context(),
                QueuedActions::Undo => self.db_state.editor_model.undo(),
                QueuedActions::Redo => self.db_state.editor_model.redo(),
                QueuedActions::Quit => {
                    self.db_state
                        .editor_model
                        .commit_all_pending_undo_contexts();
                    if self
                        .db_state
                        .editor_model
                        .any_edit_context_has_unsaved_changes()
                    {
                        self.try_set_modal_action(ConfirmQuitWithoutSavingModal::new(
                            &self.db_state.editor_model,
                        ));
                    } else {
                        self.ready_to_quit = true;
                    }
                }
                QueuedActions::QuitNoConfirm => self.ready_to_quit = true,
                QueuedActions::HandleDroppedFiles(files) => {
                    self.try_set_modal_action(ImportFilesModal::new(files));
                }
                QueuedActions::TryBeginModalAction(modal_action) => {
                    if self.modal_action.is_none() {
                        self.modal_action = Some(modal_action);
                    }
                }
                QueuedActions::MoveObjects(objects, destination) => {
                    self.db_state
                        .editor_model
                        .root_edit_context_mut()
                        .with_undo_context("MoveObjects", |edit_context| {
                            for object in objects {
                                edit_context.set_object_location(object, destination.clone());
                            }

                            EndContextBehavior::Finish
                        });
                }
            }
        }
    }

    // Set by sending Quit message to action queue. Window loop will observe this and terminate
    // application. We assume by this point we've already saved/confirmed with user.
    pub fn ready_to_quit(&self) -> bool {
        self.ready_to_quit
    }
}

use crate::db_state::DbState;
use crate::ui::modals::{ConfirmQuitWithoutSavingModal, ImportFilesModal};
use crate::ui_state::UiState;
use hydrate_model::pipeline::import_util::ImportToQueue;
use hydrate_model::pipeline::AssetEngine;
use hydrate_model::{AssetId, AssetLocation, EndContextBehavior};
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
    TryBeginModalAction(Box<dyn ModalAction>),
    MoveAssets(Vec<AssetId>, AssetLocation),
    PersistAssets(Vec<AssetId>),
    //RevertAll,
    //ResetWindowLayout,
    //SelectObjectsInAssetBrowser(Vec<AssetId>)
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
    // pub fn try_set_modal_action<T: ModalAction + 'static>(
    //     &self,
    //     action: T,
    // ) {
    //     self.queue_action(QueuedActions::TryBeginModalAction(Box::new(action)))
    // }
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
        asset_engine: &mut AssetEngine,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow;
}

impl std::fmt::Debug for dyn ModalAction {
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
    // pub importer_registry: ImporterRegistry,
    // pub builder_registry: BuilderRegistry,
    // pub import_jobs: ImportJobs,
    // pub build_jobs: BuildJobs,
    pub asset_engine: AssetEngine,
    pub action_queue: ActionQueueReceiver,
    ready_to_quit: bool,
    pub modal_action: Option<Box<dyn ModalAction>>,
}

impl AppState {
    pub fn new(
        db_state: DbState,
        asset_engine: AssetEngine,
    ) -> Self {
        AppState {
            db_state,
            ui_state: UiState::default(),
            asset_engine,
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
        let mut imports_to_queue = Vec::<ImportToQueue>::default();
        while let Ok(queued_action) = self.action_queue.action_queue_rx.try_recv() {
            match queued_action {
                QueuedActions::SaveAll => self.db_state.editor_model.save_root_edit_context(),
                QueuedActions::RevertAll => self
                    .db_state
                    .editor_model
                    .revert_root_edit_context(&mut imports_to_queue),
                QueuedActions::Undo => self.db_state.editor_model.undo().unwrap(),
                QueuedActions::Redo => self.db_state.editor_model.redo().unwrap(),
                QueuedActions::Quit => {
                    self.db_state
                        .editor_model
                        .commit_all_pending_undo_contexts();

                    let mut unsaved_assets = self
                        .db_state
                        .editor_model
                        .root_edit_context()
                        .modified_assets()
                        .clone();
                    unsaved_assets.retain(|x| !self.db_state.editor_model.is_generated_asset(*x));

                    if !unsaved_assets.is_empty() {
                        self.try_set_modal_action(ConfirmQuitWithoutSavingModal::new(
                            unsaved_assets,
                        ));
                    } else {
                        self.ready_to_quit = true;
                    }
                }
                QueuedActions::QuitNoConfirm => self.ready_to_quit = true,
                QueuedActions::HandleDroppedFiles(files) => {
                    self.try_set_modal_action(ImportFilesModal::new(
                        files,
                        self.asset_engine.importer_registry(),
                    ));
                }
                QueuedActions::TryBeginModalAction(modal_action) => {
                    if self.modal_action.is_none() {
                        self.modal_action = Some(modal_action);
                    }
                }
                QueuedActions::MoveAssets(assets, destination) => {
                    self.db_state
                        .editor_model
                        .root_edit_context_mut()
                        .with_undo_context("MoveAssets", |edit_context| {
                            for asset in assets {
                                edit_context
                                    .set_asset_location(asset, destination.clone())
                                    .unwrap();
                            }

                            EndContextBehavior::Finish
                        });
                }
                QueuedActions::PersistAssets(assets) => {
                    for asset_id in assets {
                        self.db_state.editor_model.persist_generated_asset(asset_id)
                    }
                }
            }
        }

        for import_to_queue in imports_to_queue {
            self.asset_engine.queue_import_operation(
                import_to_queue.requested_importables,
                import_to_queue.importer_id,
                import_to_queue.source_file_path,
                import_to_queue.assets_to_regenerate,
                import_to_queue.import_type,
            );
        }
    }

    // Set by sending Quit message to action queue. Window loop will observe this and terminate
    // application. We assume by this point we've already saved/confirmed with user.
    pub fn ready_to_quit(&self) -> bool {
        self.ready_to_quit
    }
}

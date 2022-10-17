use std::fmt::Formatter;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use imgui::PopupModal;
use imgui::sys::{ImGuiCond, ImVec2};
use imnodes::editor;
use crate::db_state::DbState;
use nexdb::{HashSet, ObjectId, ObjectPath};
use crate::imgui_support::ImguiManager;

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

#[derive(PartialEq)]
pub enum ModalActionControlFlow {
    Continue,
    End
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
            modal_action: None
        }
    }

    fn try_set_modal_action<T: ModalAction + 'static>(&mut self, action: T) {
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
                    self.db_state.editor_model.commit_all_pending_undo_contexts();
                    if self.db_state.editor_model.any_edit_context_has_unsaved_changes() {
                        self.try_set_modal_action(ConfirmQuitWithoutSavingModal::default());
                    } else {
                        self.ready_to_quit = true;
                    }
                },
                QueuedActions::QuitNoConfirm => self.ready_to_quit = true,
                QueuedActions::HandleDroppedFiles(files) => {
                    self.try_set_modal_action(ImportFilesModal::new(files));
                },
                QueuedActions::TryBeginModalAction(modal_action) => {
                    if self.modal_action.is_none() {
                        self.modal_action = Some(modal_action);
                    }
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




#[derive(Default)]
struct ConfirmQuitWithoutSavingModal {
    finished_first_draw: bool
}

impl ModalAction for ConfirmQuitWithoutSavingModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("ConfirmSaveQuit"));
        }

        let result = PopupModal::new(imgui::im_str!("ConfirmSaveQuit")).build(ui, || {
            ui.text("Are you sure you want to quit? Unsaved changes will be lost.");

            if ui.button(imgui::im_str!("Save Changes")) {
                ui.close_current_popup();
                action_queue.queue_action(QueuedActions::SaveAll);
                action_queue.queue_action(QueuedActions::QuitNoConfirm);

                return ModalActionControlFlow::End;
            }

            ui.same_line();

            if ui.button(imgui::im_str!("Discard Changes")) {
                ui.close_current_popup();
                action_queue.queue_action(QueuedActions::QuitNoConfirm);

                return ModalActionControlFlow::End;
            }

            ui.same_line();

            if ui.button(imgui::im_str!("Cancel")) {
                ui.close_current_popup();

                return ModalActionControlFlow::End;
            }

            ModalActionControlFlow::Continue
        });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

struct ImportFilesModal {
    finished_first_draw: bool,
    files_to_import: Vec<PathBuf>
}

impl ImportFilesModal {
    pub fn new(files_to_import: Vec<PathBuf>) -> Self {
        ImportFilesModal {
            finished_first_draw: false,
            files_to_import
        }
    }
}

impl ModalAction for ImportFilesModal {
    fn draw_imgui(
        &mut self,
        ui: &mut imgui::Ui,
        imnodes_context: &mut imnodes::Context,
        db_state: &mut DbState,
        ui_state: &mut UiState,
        action_queue: ActionQueueSender,
    ) -> ModalActionControlFlow {
        if !self.finished_first_draw {
            ui.open_popup(imgui::im_str!("Import Files"));
        }

        unsafe {
            imgui::sys::igSetNextWindowSize(ImVec2::new(600.0, 400.0), imgui::sys::ImGuiCond__ImGuiCond_Appearing as _);
        }

        let result = PopupModal::new(imgui::im_str!("Import Files"))
            .build(ui, || {
            ui.text("Files to be imported:");

            imgui::ChildWindow::new("child1")
                .size([0.0, 100.0])
                .build(ui, || {
                    for file in &self.files_to_import {
                        ui.text(file.to_str().unwrap());
                    }

            });

            ui.separator();
            ui.text("Where to import the files");

            imgui::ChildWindow::new("child2")
                .size([0.0, 180.0])
                .build(ui, || {
                    for i in 0..20 {
                        ui.text("where to import");
                    }

                });

            if ui.button(imgui::im_str!("Cancel")) {
                ui.close_current_popup();

                return ModalActionControlFlow::End;
            }

            unsafe { imgui::sys::igBeginDisabled(true); }

            ui.same_line();
            if ui.button(imgui::im_str!("TODO NOT IMPLEMENTED Import")) {
                ui.close_current_popup();

                return ModalActionControlFlow::End;
            }


            unsafe { imgui::sys::igEndDisabled(); }

            ModalActionControlFlow::Continue
        });

        self.finished_first_draw = true;
        result.unwrap_or(ModalActionControlFlow::End)
    }
}

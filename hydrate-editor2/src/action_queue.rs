use std::fmt::Formatter;
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};
use hydrate_model::edit_context::EditContext;
use hydrate_model::{AssetId, DataSetResult, EditorModel, EndContextBehavior};
use hydrate_model::pipeline::AssetEngine;
use crate::modal_action::ModalAction;
use crate::ui_state::EditorModelUiState;

pub enum UIAction {
    TryBeginModalAction(Box<dyn ModalAction>),
    EditContext(&'static str, Vec<AssetId>, Box<dyn FnOnce(&mut EditContext) -> DataSetResult<EndContextBehavior>>)
}

impl UIAction {
    // title
    // tooltip
    // default keyboard shortcut
    // draw button
    // draw menu item
}

pub struct UIActionQueueSenderInner {
    action_queue_tx: Sender<UIAction>,
}

#[derive(Clone)]
pub struct UIActionQueueSender {
    inner: Arc<UIActionQueueSenderInner>,
}

impl UIActionQueueSender {
    pub fn queue_action(
        &self,
        action: UIAction,
    ) {
        self.inner.action_queue_tx.send(action).unwrap();
    }

    // shorthand for a common action
    pub fn try_set_modal_action<T: ModalAction + 'static>(
        &self,
        action: T,
    ) {
        self.queue_action(UIAction::TryBeginModalAction(Box::new(action)))
    }

    pub fn queue_edit<F: 'static + FnOnce(&mut EditContext) -> DataSetResult<EndContextBehavior>>(
        &self,
        undo_context_name: &'static str,
        assets: Vec<AssetId>,
        f: F
    ) {
        self.queue_action(UIAction::EditContext(undo_context_name, assets, Box::new(f)))
    }
}

pub struct UIActionQueueReceiver {
    sender: UIActionQueueSender,
    action_queue_tx: Sender<UIAction>,
    action_queue_rx: Receiver<UIAction>,
}

impl Default for UIActionQueueReceiver {
    fn default() -> Self {
        let (action_queue_tx, action_queue_rx) = crossbeam_channel::unbounded();

        let sender_inner = UIActionQueueSenderInner {
            action_queue_tx: action_queue_tx.clone(),
        };

        let sender = UIActionQueueSender {
            inner: Arc::new(sender_inner),
        };

        UIActionQueueReceiver {
            sender,
            action_queue_tx,
            action_queue_rx,
        }
    }
}

impl UIActionQueueReceiver {
    pub fn sender(&self) -> UIActionQueueSender {
        self.sender.clone()
    }

    pub fn queue_action(
        &self,
        action: UIAction,
    ) {
        self.action_queue_tx.send(action).unwrap();
    }

    pub fn process(
        &self,
        editor_model: &mut EditorModel,
        asset_engine: &mut AssetEngine,
        ui_state: &EditorModelUiState,
        modal_action: &mut Option<Box<dyn ModalAction>>
    ) {
        for action in self.action_queue_rx.try_iter() {
            match action {
                UIAction::TryBeginModalAction(modal) => {
                    if modal_action.is_none() {
                        *modal_action = Some(modal);
                    }
                },
                UIAction::EditContext(undo_context_name, assets_to_edit, f) => {
                    editor_model.root_edit_context_mut().with_undo_context(&undo_context_name, |x| {
                        f(x).unwrap()
                    })
                }
            }
        }
    }
}

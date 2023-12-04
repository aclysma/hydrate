use crate::app::UiState;
use crate::modal_action::ModalAction;
use crate::ui::modals::ConfirmRevertChanges;
use crate::ui::modals::{ConfirmQuitWithoutSaving};
use crossbeam_channel::{Receiver, Sender};
use hydrate_model::edit_context::EditContext;
use hydrate_model::pipeline::import_util::ImportToQueue;
use hydrate_model::pipeline::AssetEngine;
use hydrate_model::{AssetId, AssetLocation, AssetName, DataSetError, DataSetResult, EditorModel, EndContextBehavior, NullOverride, PropertyPath, SchemaRecord, Value};
use std::sync::Arc;

pub enum UIAction {
    TryBeginModalAction(Box<dyn ModalAction>),
    EditContext(
        &'static str,
        Vec<AssetId>,
        Box<dyn FnOnce(&mut EditContext) -> DataSetResult<EndContextBehavior>>,
    ),
    Undo,
    Redo,
    SaveAll,
    RevertAll,
    RevertAllNoConfirm,
    Quit,
    QuitNoConfirm,
    PersistAssets(Vec<AssetId>),
    ForceRebuild(Vec<AssetId>),
    ShowAssetInAssetGallery(AssetId),
    MoveAsset(AssetId, AssetLocation),
    NewAsset(AssetName, AssetLocation, SchemaRecord, Option<AssetId>),
    DeleteAsset(AssetId),
    SetProperty(AssetId, PropertyPath, Option<Value>, EndContextBehavior),
    ApplyPropertyOverrideToPrototype(AssetId, PropertyPath),
    SetNullOverride(AssetId, PropertyPath, NullOverride),
    AddDynamicArrayOverride(AssetId, PropertyPath),
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

    // pub fn queue_edit<
    //     F: 'static + FnOnce(&mut EditContext) -> DataSetResult<EndContextBehavior>,
    // >(
    //     &self,
    //     undo_context_name: &'static str,
    //     assets: Vec<AssetId>,
    //     f: F,
    // ) {
    //     self.queue_action(UIAction::EditContext(
    //         undo_context_name,
    //         assets,
    //         Box::new(f),
    //     ))
    // }
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
        ui_state: &mut UiState,
        modal_action: &mut Option<Box<dyn ModalAction>>,
        ctx: &egui::Context,
    ) {
        let mut imports_to_queue = Vec::<ImportToQueue>::default();
        while let Ok(action) = self.action_queue_rx.try_recv() {
            match action {
                UIAction::SaveAll => editor_model.save_root_edit_context(),
                UIAction::RevertAll => {
                    if editor_model.any_edit_context_has_unsaved_changes() {
                        *modal_action = Some(Box::new(ConfirmRevertChanges {}))
                    }
                }
                UIAction::RevertAllNoConfirm => {
                    editor_model.revert_root_edit_context(&mut imports_to_queue)
                }
                UIAction::Undo => editor_model.undo().unwrap(),
                UIAction::Redo => editor_model.redo().unwrap(),
                UIAction::Quit => {
                    // Only verify with a modal if there are unsaved changes
                    if editor_model.any_edit_context_has_unsaved_changes() {
                        *modal_action = Some(Box::new(ConfirmQuitWithoutSaving {}))
                    } else {
                        self.action_queue_tx.send(UIAction::QuitNoConfirm).unwrap();
                    }
                }
                UIAction::QuitNoConfirm => {
                    ui_state.user_confirmed_should_quit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                }
                UIAction::TryBeginModalAction(modal) => {
                    if modal_action.is_none() {
                        *modal_action = Some(modal);
                    }
                }
                UIAction::EditContext(undo_context_name, assets_to_edit, f) => editor_model
                    .root_edit_context_mut()
                    .with_undo_context(&undo_context_name, |x| f(x).unwrap()),

                UIAction::PersistAssets(asset_ids) => {
                    for asset_id in asset_ids {
                        editor_model.persist_generated_asset(asset_id);
                    }
                }
                UIAction::ForceRebuild(asset_ids) => {
                    for asset_id in asset_ids {
                        asset_engine.queue_build_operation(asset_id)
                    }
                }
                UIAction::ShowAssetInAssetGallery(asset_id) => {
                    ui_state.asset_gallery_ui_state.selected_assets.clear();
                    ui_state
                        .asset_gallery_ui_state
                        .selected_assets
                        .insert(asset_id);

                    if let Some(location) = editor_model.root_edit_context().asset_location(asset_id) {
                        ui_state.asset_tree_ui_state.selected_tree_node = Some(location);
                    }
                }
                UIAction::MoveAsset(moving_asset, new_location) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "move asset",
                        |edit_context| {
                            let result =
                                edit_context.set_asset_location(moving_asset, new_location);
                            match result {
                                Ok(_) => {
                                    // do nothing
                                }
                                Err(DataSetError::NewLocationIsChildOfCurrentAsset) => {
                                    // do nothing
                                }
                                _ => {
                                    unimplemented!()
                                }
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::NewAsset(asset_name, asset_location, schema_record, prototype) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "new asset",
                        |edit_context| {
                            let new_asset_id = if let Some(prototype) = prototype {
                                edit_context.new_asset_from_prototype(
                                    &asset_name,
                                    &asset_location,
                                    prototype
                                ).unwrap()
                            } else {
                                edit_context.new_asset(
                                    &asset_name,
                                    &asset_location,
                                    &schema_record,
                                )
                            };

                            self.sender
                                .queue_action(UIAction::ShowAssetInAssetGallery(new_asset_id));
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::DeleteAsset(asset_id) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "delete asset",
                        |edit_context| {
                            edit_context.delete_asset(asset_id).unwrap();
                            EndContextBehavior::Finish
                        }
                    );
                }
                UIAction::SetProperty(asset_id, property_path, value, end_context_behavior) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set property",
                        |edit_context| {
                            edit_context.set_property_override(asset_id, property_path.path(), value).unwrap();
                            end_context_behavior
                        }
                    );
                },
                UIAction::ApplyPropertyOverrideToPrototype(asset_id, property_path) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "apply override",
                        |edit_context| {
                            edit_context
                                .apply_property_override_to_prototype(asset_id, property_path.path()).unwrap();
                            EndContextBehavior::Finish
                        }
                    );
                },
                UIAction::SetNullOverride(asset_id, property_path, null_override) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set null override",
                        |edit_context| {
                            edit_context
                                .set_null_override(asset_id, property_path.path(), null_override).unwrap();
                            EndContextBehavior::Finish
                        }
                    );
                }
                UIAction::AddDynamicArrayOverride(asset_id, property_path) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set null override",
                        |edit_context| {
                            edit_context
                                .add_dynamic_array_override(asset_id, property_path.path()).unwrap();
                            EndContextBehavior::Finish
                        }
                    );
                }
            }
        }

        for import_to_queue in imports_to_queue {
            asset_engine.queue_import_operation(
                import_to_queue.requested_importables,
                import_to_queue.importer_id,
                import_to_queue.source_file_path,
                import_to_queue.import_type,
            );
        }
    }
}

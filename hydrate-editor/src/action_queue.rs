use crate::app::UiState;
use crate::modal_action::ModalAction;
use crate::ui::modals::ConfirmQuitWithoutSaving;
use crate::ui::modals::ConfirmRevertChanges;
use crossbeam_channel::{Receiver, Sender};
use egui::KeyboardShortcut;
use hydrate_base::hashing::HashMap;
use hydrate_model::edit_context::EditContext;
use hydrate_model::pipeline::{
    AssetEngine, HydrateProjectConfiguration, ImportJobSourceFile, ImportJobToQueue, ImportLogData,
    ImportType,
};
use hydrate_model::{
    AssetId, AssetLocation, AssetName, DataSetError, DataSetErrorWithBacktrace, DataSetResult,
    EditorModel, EndContextBehavior, NullOverride, OverrideBehavior, PropertyPath, Schema,
    SchemaFingerprint, SchemaRecord, Value,
};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub enum UIAction {
    TryBeginModalAction(Box<dyn ModalAction>),
    EditContext(
        &'static str,
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
    BuildAll,
    ReimportAndRebuild(Vec<AssetId>),
    ForceRebuild(Vec<AssetId>),
    ShowAssetInAssetGallery(AssetId),
    MoveAssets(Vec<AssetId>, AssetLocation),
    MoveOrRename(Vec<AssetId>, Option<AssetName>, AssetLocation),
    NewAsset(AssetName, AssetLocation, SchemaRecord, Option<AssetId>),
    DuplicateAssets(Vec<AssetId>),
    DeleteAssets(Vec<AssetId>),
    SetProperty(
        Vec<AssetId>,
        PropertyPath,
        Option<Value>,
        EndContextBehavior,
    ),
    ClearPropertiesForRecord(Vec<AssetId>, PropertyPath, SchemaFingerprint),
    CommitPendingUndoContext,
    ApplyPropertyOverrideToPrototype(Vec<AssetId>, PropertyPath),
    ApplyPropertyOverrideToPrototypeForRecord(Vec<AssetId>, PropertyPath, SchemaFingerprint),
    ApplyResolvedPropertyToAllSelected(AssetId, Vec<AssetId>, PropertyPath),
    ApplyResolvedPropertyToAllSelectedForRecord(
        AssetId,
        Vec<AssetId>,
        PropertyPath,
        SchemaFingerprint,
    ),
    SetNullOverride(Vec<AssetId>, PropertyPath, NullOverride),
    AddDynamicArrayEntry(AssetId, PropertyPath),
    AddMapEntry(AssetId, PropertyPath),
    RemoveDynamicArrayEntry(AssetId, PropertyPath, Uuid),
    RemoveMapEntry(AssetId, PropertyPath, Uuid),
    MoveDynamicArrayEntryUp(AssetId, PropertyPath, Uuid),
    MoveDynamicArrayEntryDown(AssetId, PropertyPath, Uuid),
    // This moves values, not entries, that's why it's named differently
    MoveStaticArrayOverrideUp(Vec<AssetId>, PropertyPath, usize),
    MoveStaticArrayOverrideDown(Vec<AssetId>, PropertyPath, usize),
    OverrideWithDefault(Vec<AssetId>, PropertyPath),
    SetOverrideBehavior(Vec<AssetId>, PropertyPath, OverrideBehavior),
    ToggleSelectAllAssetGallery,
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
    anything_has_focus_last_frame: bool,
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
            anything_has_focus_last_frame: false,
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
        &mut self,
        project_config: &HydrateProjectConfiguration,
        editor_model: &mut EditorModel,
        asset_engine: &mut AssetEngine,
        ui_state: &mut UiState,
        modal_action: &mut Option<Box<dyn ModalAction>>,
        ctx: &egui::Context,
    ) {
        {
            // If we are editing a text field, it is the focus and we should not try to listen for hotkeys
            let anything_has_focus = ctx.memory(|mem| mem.focus().is_some());
            if !anything_has_focus && !self.anything_has_focus_last_frame {
                ctx.input_mut(|input| {
                    //for command in UICommand::iter() {
                    //     if let Some(kb_shortcut) = command.kb_shortcut() {
                    //         if input.consume_shortcut(&kb_shortcut) {
                    //             return Some(command);
                    //         }
                    //     }
                    //}
                    //None

                    if input.consume_shortcut(&KeyboardShortcut {
                        key: egui::Key::A,
                        modifiers: egui::Modifiers::COMMAND,
                    }) {
                        // Select All
                        self.action_queue_tx
                            .send(UIAction::ToggleSelectAllAssetGallery)
                            .unwrap();
                    }

                    if input.consume_shortcut(&KeyboardShortcut {
                        key: egui::Key::B,
                        modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    }) {
                        self.action_queue_tx.send(UIAction::BuildAll).unwrap();
                    }

                    if input.consume_shortcut(&KeyboardShortcut {
                        key: egui::Key::S,
                        modifiers: egui::Modifiers::COMMAND,
                    }) {
                        self.action_queue_tx.send(UIAction::SaveAll).unwrap();
                    }

                    if input.consume_shortcut(&KeyboardShortcut {
                        key: egui::Key::Z,
                        modifiers: egui::Modifiers::COMMAND,
                    }) {
                        self.action_queue_tx.send(UIAction::Undo).unwrap();
                    }

                    if input.consume_shortcut(&KeyboardShortcut {
                        key: egui::Key::Z,
                        modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    }) {
                        self.action_queue_tx.send(UIAction::Redo).unwrap();
                    }
                });
            }

            self.anything_has_focus_last_frame = anything_has_focus;
        }

        let mut import_job_to_queue = ImportJobToQueue::default();
        while let Ok(action) = self.action_queue_rx.try_recv() {
            match action {
                UIAction::ToggleSelectAllAssetGallery => {
                    ui_state.asset_gallery_ui_state.toggle_select_all();
                }
                UIAction::SaveAll => editor_model.save_root_edit_context(),
                UIAction::RevertAll => {
                    if editor_model.any_edit_context_has_unsaved_changes() {
                        *modal_action = Some(Box::new(ConfirmRevertChanges {}))
                    }
                }
                UIAction::RevertAllNoConfirm => {
                    editor_model.revert_root_edit_context(project_config, &mut import_job_to_queue)
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
                UIAction::EditContext(undo_context_name, f) => editor_model
                    .root_edit_context_mut()
                    .with_undo_context(&undo_context_name, |x| f(x).unwrap()),

                UIAction::PersistAssets(asset_ids) => {
                    for asset_id in asset_ids {
                        editor_model.persist_generated_asset(asset_id);
                    }
                }
                UIAction::BuildAll => {
                    asset_engine.queue_build_all();
                }
                UIAction::MoveOrRename(asset_ids, new_name, new_location) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "MoveOrRename",
                        |edit_context| {
                            for &asset_id in &asset_ids {
                                if let Some(new_name) = &new_name {
                                    edit_context
                                        .set_asset_name(asset_id, new_name.clone())
                                        .unwrap();
                                }

                                edit_context
                                    .set_asset_location(asset_id, new_location)
                                    .unwrap();
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::ReimportAndRebuild(asset_ids) => {
                    // - Only update assets that were requested
                    // - Process source files once
                    // - Match the same import settings as were originally in place, or fail if
                    //   this is not possible.

                    let mut import_job_to_queue = ImportJobToQueue::default();
                    for &asset_id in &asset_ids {
                        let root_edit_context = editor_model.root_edit_context();
                        let import_info = root_edit_context.import_info(asset_id);
                        if let Some(import_info) = import_info {
                            //import_info.importer_id()
                            //
                            let source_file_path: PathBuf = import_info
                                .source_file()
                                .canonicalized_absolute_path(root_edit_context, &PathBuf::default())
                                .unwrap()
                                .path()
                                .into();

                            // let requested_importables = HashMap::default();
                            // let source_file = ImportJobSourceFile {
                            //     importer_id: import_info.importer_id(),
                            //     import_type: ImportType::ImportAlways,
                            //     requested_importables,
                            //     source_file_path,
                            // };

                            let mut asset_id_assignments = HashMap::default();
                            asset_id_assignments
                                .insert(import_info.importable_name().clone(), asset_id);

                            println!("reimport {:?}", source_file_path);
                            hydrate_model::pipeline::recursively_gather_import_operations_and_create_assets(
                                project_config,
                                &source_file_path,
                                asset_engine.importer_registry().importer(import_info.importer_id()).unwrap(),
                                root_edit_context,
                                asset_engine.importer_registry(),
                                &root_edit_context.asset_location(asset_id).unwrap(),
                                Some(&asset_id_assignments),
                                &mut import_job_to_queue,
                            ).unwrap();

                            //hydrate_pipeline::recursively_gather_import_operations_and_create_assets()
                        }
                    }

                    if !import_job_to_queue.is_empty() {
                        asset_engine.queue_import_operation(import_job_to_queue);

                        // Can't do incremental build, manifest won't have *anything* but what was built
                        // for asset_id in asset_ids {
                        //     asset_engine.queue_build_asset(asset_id);
                        // }
                    }

                    asset_engine.queue_build_all();
                }
                UIAction::ForceRebuild(asset_ids) => {
                    // Can't do incremental build, manifest won't have *anything* but what was built
                    // for asset_id in asset_ids {
                    //     asset_engine.queue_build_asset(asset_id);
                    // }

                    asset_engine.queue_build_all();
                }
                UIAction::ShowAssetInAssetGallery(asset_id) => {
                    ui_state
                        .asset_gallery_ui_state
                        .set_selection(Some(asset_id));

                    if let Some(location) =
                        editor_model.root_edit_context().asset_location(asset_id)
                    {
                        ui_state.asset_tree_ui_state.selected_tree_node = Some(location);
                    }
                }
                UIAction::MoveAssets(moving_assets, new_location) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "move asset",
                        |edit_context| {
                            for &moving_asset in &moving_assets {
                                let result =
                                    edit_context.set_asset_location(moving_asset, new_location);
                                match result {
                                    Ok(_) => {
                                        // do nothing
                                    }
                                    Err(DataSetErrorWithBacktrace {
                                        error: DataSetError::NewLocationIsChildOfCurrentAsset,
                                        ..
                                    }) => {
                                        // do nothing
                                    }
                                    _ => {
                                        unimplemented!()
                                    }
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
                                edit_context
                                    .new_asset_from_prototype(
                                        &asset_name,
                                        &asset_location,
                                        prototype,
                                    )
                                    .unwrap()
                            } else {
                                edit_context.new_asset(&asset_name, &asset_location, &schema_record)
                            };

                            self.sender
                                .queue_action(UIAction::ShowAssetInAssetGallery(new_asset_id));
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::DuplicateAssets(asset_ids) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "delete asset",
                        |edit_context| {
                            for asset_id in asset_ids {
                                let new_asset_id = edit_context.duplicate_asset(asset_id).unwrap();
                                if edit_context.import_info(asset_id).is_some() {
                                    asset_engine
                                        .duplicate_import_data(asset_id, new_asset_id)
                                        .unwrap();
                                }
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::DeleteAssets(asset_ids) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "delete asset",
                        |edit_context| {
                            for asset_id in asset_ids {
                                edit_context.delete_asset(asset_id).unwrap();
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::SetProperty(asset_ids, property_path, value, end_context_behavior) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set property",
                        |edit_context| {
                            for asset_id in asset_ids {
                                edit_context
                                    .set_property_override(
                                        asset_id,
                                        property_path.path(),
                                        value.clone(),
                                    )
                                    .unwrap();
                            }
                            end_context_behavior
                        },
                    );
                }
                UIAction::ClearPropertiesForRecord(
                    asset_ids,
                    property_path,
                    record_schema_fingerprint,
                ) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "ClearPropertiesForRecord",
                        |edit_context| {
                            let record_schema = edit_context
                                .schema_set()
                                .find_named_type_by_fingerprint(record_schema_fingerprint)
                                .unwrap()
                                .as_record()
                                .unwrap()
                                .clone();

                            for field in record_schema.fields() {
                                let field_path = property_path.push(field.name());
                                for &asset_id in &asset_ids {
                                    edit_context
                                        .set_property_override(asset_id, field_path.path(), None)
                                        .unwrap();
                                }
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::CommitPendingUndoContext => {
                    editor_model
                        .root_edit_context_mut()
                        .commit_pending_undo_context();
                }
                UIAction::ApplyPropertyOverrideToPrototype(asset_ids, property_path) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "apply override",
                        |edit_context| {
                            for asset_id in asset_ids {
                                edit_context
                                    .apply_property_override_to_prototype(
                                        asset_id,
                                        property_path.path(),
                                    )
                                    .unwrap();
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }

                UIAction::ApplyPropertyOverrideToPrototypeForRecord(
                    asset_ids,
                    property_path,
                    record_schema_fingerprint,
                ) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "ApplyPropertyOverrideToPrototypeForRecord",
                        |edit_context| {
                            let record_schema = edit_context
                                .schema_set()
                                .find_named_type_by_fingerprint(record_schema_fingerprint)
                                .unwrap()
                                .as_record()
                                .unwrap()
                                .clone();

                            for field in record_schema.fields() {
                                let field_path = property_path.push(field.name());
                                for &asset_id in &asset_ids {
                                    edit_context
                                        .apply_property_override_to_prototype(
                                            asset_id,
                                            field_path.path(),
                                        )
                                        .unwrap();
                                }
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }

                UIAction::ApplyResolvedPropertyToAllSelected(
                    src_asset_id,
                    selected_asset_ids,
                    property_path,
                ) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "apply override",
                        |edit_context| {
                            let value = edit_context
                                .resolve_property(src_asset_id, property_path.path())
                                .unwrap()
                                .clone();

                            for &asset_id in &selected_asset_ids {
                                edit_context
                                    .set_property_override(
                                        asset_id,
                                        property_path.path(),
                                        Some(value.clone()),
                                    )
                                    .unwrap();
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::ApplyResolvedPropertyToAllSelectedForRecord(
                    src_asset_id,
                    selected_asset_ids,
                    property_path,
                    record_schema_fingerprint,
                ) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "apply override",
                        |edit_context| {
                            let record_schema = edit_context
                                .schema_set()
                                .find_named_type_by_fingerprint(record_schema_fingerprint)
                                .unwrap()
                                .as_record()
                                .unwrap()
                                .clone();

                            for field in record_schema.fields() {
                                let field_path = property_path.push(field.name());
                                let value = edit_context
                                    .resolve_property(src_asset_id, field_path.path())
                                    .unwrap()
                                    .clone();

                                for &asset_id in &selected_asset_ids {
                                    edit_context
                                        .set_property_override(
                                            asset_id,
                                            field_path.path(),
                                            Some(value.clone()),
                                        )
                                        .unwrap();
                                }
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }

                UIAction::SetNullOverride(asset_ids, property_path, null_override) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set null override",
                        |edit_context| {
                            for &asset_id in &asset_ids {
                                edit_context
                                    .set_null_override(
                                        asset_id,
                                        property_path.path(),
                                        null_override,
                                    )
                                    .unwrap();
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::AddDynamicArrayEntry(asset_id, property_path) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set null override",
                        |edit_context| {
                            edit_context
                                .add_dynamic_array_entry(asset_id, property_path.path())
                                .unwrap();
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::AddMapEntry(asset_id, property_path) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "set null override",
                        |edit_context| {
                            edit_context
                                .add_map_entry(asset_id, property_path.path())
                                .unwrap();
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::RemoveDynamicArrayEntry(asset_id, property_path, entry_uuid) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "RemoveDynamicArrayOverride",
                        |edit_context| {
                            edit_context
                                .remove_dynamic_array_entry(
                                    asset_id,
                                    property_path.path(),
                                    entry_uuid,
                                )
                                .unwrap();

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::RemoveMapEntry(asset_id, property_path, entry_uuid) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "RemoveDynamicArrayOverride",
                        |edit_context| {
                            edit_context
                                .remove_map_entry(asset_id, property_path.path(), entry_uuid)
                                .unwrap();

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::MoveDynamicArrayEntryUp(asset_id, property_path, entry_uuid) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "MoveDynamicArrayOverrideUp",
                        |edit_context| {
                            let overrides: Vec<_> = edit_context
                                .get_dynamic_array_entries(asset_id, property_path.path())
                                .unwrap()
                                .copied()
                                .collect();
                            let current_index =
                                overrides.iter().position(|x| *x == entry_uuid).unwrap();
                            if current_index > 0 {
                                // Remove
                                edit_context
                                    .remove_dynamic_array_entry(
                                        asset_id,
                                        property_path.path(),
                                        entry_uuid,
                                    )
                                    .unwrap();
                                // Insert one index higher
                                edit_context
                                    .insert_dynamic_array_entry(
                                        asset_id,
                                        property_path.path(),
                                        current_index - 1,
                                        entry_uuid,
                                    )
                                    .unwrap();
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::MoveDynamicArrayEntryDown(asset_id, property_path, entry_uuid) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "MoveDynamicArrayOverrideDown",
                        |edit_context| {
                            let overrides: Vec<_> = edit_context
                                .get_dynamic_array_entries(asset_id, property_path.path())
                                .unwrap()
                                .collect();
                            let current_index =
                                overrides.iter().position(|x| **x == entry_uuid).unwrap();
                            if current_index < overrides.len() - 1 {
                                // Remove
                                edit_context
                                    .remove_dynamic_array_entry(
                                        asset_id,
                                        property_path.path(),
                                        entry_uuid,
                                    )
                                    .unwrap();
                                // Re-insert at next index
                                edit_context
                                    .insert_dynamic_array_entry(
                                        asset_id,
                                        property_path.path(),
                                        current_index + 1,
                                        entry_uuid,
                                    )
                                    .unwrap();
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }
                // UIAction::ClearStaticArrayOverride(asset_id, property_path, entry_index) => {
                //
                // }
                UIAction::MoveStaticArrayOverrideUp(asset_ids, property_path, entry_index) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "MoveStaticArrayOverrideUp",
                        |edit_context| {
                            for &asset_id in &asset_ids {
                                let schema_set = edit_context.schema_set().clone();
                                let index_a = entry_index;
                                let property_path_a = property_path.push(&index_a.to_string());
                                let bundle_a = edit_context
                                    .read_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_a.path(),
                                    )
                                    .unwrap();

                                let index_b = entry_index - 1;
                                let property_path_b = property_path.push(&index_b.to_string());
                                let bundle_b = edit_context
                                    .read_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_b.path(),
                                    )
                                    .unwrap();

                                edit_context
                                    .write_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_a.path(),
                                        &bundle_b,
                                    )
                                    .unwrap();
                                edit_context
                                    .write_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_b.path(),
                                        &bundle_a,
                                    )
                                    .unwrap();
                            }

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::MoveStaticArrayOverrideDown(asset_ids, property_path, entry_index) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "MoveStaticArrayOverrideDown",
                        |edit_context| {
                            for &asset_id in &asset_ids {
                                let schema_set = edit_context.schema_set().clone();
                                let index_a = entry_index;
                                let property_path_a = property_path.push(&index_a.to_string());
                                let bundle_a = edit_context
                                    .read_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_a.path(),
                                    )
                                    .unwrap();

                                let index_b = entry_index + 1;
                                let property_path_b = property_path.push(&index_b.to_string());
                                let bundle_b = edit_context
                                    .read_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_b.path(),
                                    )
                                    .unwrap();

                                edit_context
                                    .write_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_a.path(),
                                        &bundle_b,
                                    )
                                    .unwrap();
                                edit_context
                                    .write_properties_bundle(
                                        &schema_set,
                                        asset_id,
                                        property_path_b.path(),
                                        &bundle_a,
                                    )
                                    .unwrap();
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::OverrideWithDefault(asset_ids, property_path) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "OverrideWithDefault",
                        |edit_context| {
                            //let schema = edit_context.asset_schema(asset_id).unwrap().clone();

                            for asset_id in asset_ids {
                                let schema = edit_context
                                    .asset_schema(asset_id)
                                    .unwrap()
                                    .find_property_schema(
                                        property_path.path(),
                                        edit_context.schema_set().schemas(),
                                    )
                                    .unwrap();
                                println!(
                                    "find schema {:?} for property {:?}",
                                    schema,
                                    property_path.path()
                                );
                                override_with_default_values_recursively(
                                    asset_id,
                                    &property_path,
                                    schema,
                                    edit_context,
                                );
                            }

                            //let schema_set = edit_context.schema_set().clone();
                            // let schema = edit_context.asset_schema(asset_id).unwrap().clone();
                            // for field in schema.fields() {
                            //     let field_name = property_path.push(field.name());
                            //     let field_schema = schema.field_schema(field).unwrap();
                            //     match schema {
                            //
                            //     }
                            // }

                            // Value::default_for_schema(&property_schema, schema_set);
                            // edit_context.set_property_override(asset_id, path, schema_set.)

                            EndContextBehavior::Finish
                        },
                    );
                }
                UIAction::SetOverrideBehavior(asset_ids, property_path, override_behavior) => {
                    editor_model.root_edit_context_mut().with_undo_context(
                        "SetOverrideBehavior",
                        |edit_context| {
                            for &asset_id in &asset_ids {
                                edit_context
                                    .set_override_behavior(
                                        asset_id,
                                        property_path.path(),
                                        override_behavior,
                                    )
                                    .unwrap();
                            }
                            EndContextBehavior::Finish
                        },
                    );
                }
            }
        }

        if !import_job_to_queue.is_empty() {
            asset_engine.queue_import_operation(import_job_to_queue);
        }
    }
}

fn override_with_default_values_recursively(
    asset_id: AssetId,
    property_path: &PropertyPath,
    schema: Schema,
    edit_context: &mut EditContext,
) {
    println!("{} {:?} set to default", property_path.path(), schema);
    match schema {
        Schema::Boolean
        | Schema::I32
        | Schema::I64
        | Schema::U32
        | Schema::U64
        | Schema::F32
        | Schema::F64
        | Schema::Bytes
        | Schema::String
        | Schema::AssetRef(_)
        | Schema::Enum(_) => {
            println!("set path {:?} {:?}", property_path.path(), schema);
            edit_context
                .set_property_override(
                    asset_id,
                    property_path.path(),
                    Some(Value::default_for_schema(&schema, edit_context.schema_set()).clone()),
                )
                .unwrap();
        }
        Schema::Nullable(_) => {
            edit_context
                .set_null_override(asset_id, property_path.path(), NullOverride::SetNull)
                .unwrap();
        }
        Schema::StaticArray(schema) => {
            for i in 0..schema.length() {
                let element_path = property_path.push(&i.to_string());
                override_with_default_values_recursively(
                    asset_id,
                    &element_path,
                    schema.item_type().clone(),
                    edit_context,
                );
            }
        }
        Schema::DynamicArray(_) => {
            edit_context
                .set_override_behavior(asset_id, property_path.path(), OverrideBehavior::Replace)
                .unwrap();
        }
        Schema::Map(_) => {
            edit_context
                .set_override_behavior(asset_id, property_path.path(), OverrideBehavior::Replace)
                .unwrap();
        }
        Schema::Record(record_schema) => {
            let record_schema = edit_context
                .schema_set()
                .find_named_type_by_fingerprint(record_schema)
                .unwrap()
                .as_record()
                .unwrap()
                .clone();
            println!(
                "iterate fields of {:?} {:?}",
                record_schema,
                record_schema.fields()
            );
            for field in record_schema.fields() {
                let field_path = property_path.push(field.name());
                let field_schema = record_schema.field_schema(field.name()).unwrap();
                override_with_default_values_recursively(
                    asset_id,
                    &field_path,
                    field_schema.clone(),
                    edit_context,
                );
            }
        }
    }
}

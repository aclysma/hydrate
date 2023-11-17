use hydrate_data::DataSetResult;
use slotmap::DenseSlotMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::edit_context::EditContext;
use crate::{AssetId, AssetLocation, DataSet, DataSetDiffSet, EditContextKey, HashSet};

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.

#[derive(PartialEq)]
pub enum EndContextBehavior {
    Finish,
    AllowResume,
}

pub struct CompletedUndoContextMessage {
    edit_context_key: EditContextKey,
    diff_set: DataSetDiffSet,
}

pub struct UndoStack {
    undo_chain: Vec<CompletedUndoContextMessage>,
    // Undo/Redo will decrease/increase this value, using apply/revert diffs to move backward and
    // forward. Appending new diffs will truncate the chain at current position and push a new
    // step on the chain. Zero means we have undone everything or there are no steps to undo.
    current_undo_index: usize,
    completed_undo_context_tx: Sender<CompletedUndoContextMessage>,
    completed_undo_context_rx: Receiver<CompletedUndoContextMessage>,
}

impl Default for UndoStack {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        UndoStack {
            undo_chain: Default::default(),
            current_undo_index: 0,
            completed_undo_context_tx: tx,
            completed_undo_context_rx: rx,
        }
    }
}

impl UndoStack {
    pub fn completed_undo_context_tx(&self) -> &Sender<CompletedUndoContextMessage> {
        &self.completed_undo_context_tx
    }

    // This pulls incoming steps off the receive queue. These diffs have already been applied, so
    // we mainly just use this to drop undone steps that can no longer be used, and to place them
    // on the end of the chain
    fn drain_rx(&mut self) {
        while let Ok(diff) = self.completed_undo_context_rx.try_recv() {
            self.undo_chain.truncate(self.current_undo_index);
            self.undo_chain.push(diff);
            self.current_undo_index += 1;
        }
    }

    pub fn undo(
        &mut self,
        edit_contexts: &mut DenseSlotMap<EditContextKey, EditContext>,
    ) -> DataSetResult<()> {
        // If we have any incoming steps, consume them now
        self.drain_rx();

        // if we undo the first step in the chain (i.e. our undo index is currently 1), we want to
        // use the revert diff in the 0th index of the chain
        if self.current_undo_index > 0 {
            if let Some(current_step) = self.undo_chain.get(self.current_undo_index) {
                let edit_context = edit_contexts
                    .get_mut(current_step.edit_context_key)
                    .unwrap();
                // We don't want anything being written to the undo context at this point, since we're using it
                edit_context.cancel_pending_undo_context()?;
                let result = edit_context.apply_diff(
                    &current_step.diff_set.revert_diff,
                    &current_step.diff_set.modified_assets,
                    &current_step.diff_set.modified_locations,
                );
                self.current_undo_index -= 1;
                return result;
            }
        }

        Ok(())
    }

    pub fn redo(
        &mut self,
        edit_contexts: &mut DenseSlotMap<EditContextKey, EditContext>,
    ) -> DataSetResult<()> {
        // If we have any incoming steps, consume them now
        self.drain_rx();

        // if we redo the first step in the chain (i.e. our undo index is currently 0), we want to
        // use the apply diff in the 0th index of the chain. If our current step is == length of
        // chain, we have no more steps available to redo
        if let Some(current_step) = self.undo_chain.get(self.current_undo_index) {
            let edit_context = edit_contexts
                .get_mut(current_step.edit_context_key)
                .unwrap();
            // We don't want anything being written to the undo context at this point, since we're using it
            edit_context.cancel_pending_undo_context()?;
            let result = edit_context.apply_diff(
                &current_step.diff_set.apply_diff,
                &current_step.diff_set.modified_assets,
                &current_step.diff_set.modified_locations,
            );
            self.current_undo_index += 1;
            return result;
        }

        Ok(())
    }
}

// Transaction that holds exclusive access for the data and will directly commit changes. It can
// compare directly against the original dataset for changes
pub struct UndoContext {
    edit_context_key: EditContextKey,
    before_state: DataSet,
    tracked_assets: HashSet<AssetId>,
    context_name: Option<&'static str>,
    completed_undo_context_tx: Sender<CompletedUndoContextMessage>,
}

impl UndoContext {
    pub(crate) fn new(
        undo_stack: &UndoStack,
        edit_context_key: EditContextKey,
    ) -> Self {
        UndoContext {
            edit_context_key,
            before_state: Default::default(),
            tracked_assets: Default::default(),
            context_name: Default::default(),
            completed_undo_context_tx: undo_stack.completed_undo_context_tx.clone(),
        }
    }

    // Call after adding a new asset
    pub(crate) fn track_new_asset(
        &mut self,
        asset_id: AssetId,
    ) {
        if self.context_name.is_some() {
            self.tracked_assets.insert(asset_id);
        }
    }

    // Call before editing or deleting an asset
    pub(crate) fn track_existing_asset(
        &mut self,
        after_state: &DataSet,
        asset_id: AssetId,
    ) -> DataSetResult<()> {
        if self.context_name.is_some() {
            //TODO: Preserve sub-assets?
            if !self.tracked_assets.contains(&asset_id) {
                self.tracked_assets.insert(asset_id);
                self.before_state.copy_from(&after_state, asset_id)?;
            }
        }

        Ok(())
    }

    pub(crate) fn has_open_context(&self) -> bool {
        self.context_name.is_some()
    }

    pub(crate) fn begin_context(
        &mut self,
        after_state: &DataSet,
        name: &'static str,
        modified_assets: &mut HashSet<AssetId>,
        modified_locations: &mut HashSet<AssetLocation>,
    ) {
        if self.context_name == Some(name) {
            // don't need to do anything, we can append to the current context
        } else {
            // commit the context that's in flight, if one exists
            if self.context_name.is_some() {
                // This won't do anything if there's nothing to send
                self.commit_context(after_state, modified_assets, modified_locations);
            }

            self.context_name = Some(name);
        }
    }

    pub(crate) fn end_context(
        &mut self,
        after_state: &DataSet,
        end_context_behavior: EndContextBehavior,
        modified_assets: &mut HashSet<AssetId>,
        modified_locations: &mut HashSet<AssetLocation>,
    ) {
        if end_context_behavior != EndContextBehavior::AllowResume {
            // This won't do anything if there's nothing to send
            self.commit_context(after_state, modified_assets, modified_locations);
        }
    }

    pub(crate) fn cancel_context(
        &mut self,
        after_state: &mut DataSet,
    ) -> DataSetResult<()> {
        let mut first_error = None;

        if !self.tracked_assets.is_empty() {
            // Delete newly created assets
            let keys_to_delete: Vec<_> = after_state
                .assets()
                .keys()
                .filter(|x| {
                    self.tracked_assets.contains(x) && !self.before_state.assets().contains_key(x)
                })
                .copied()
                .collect();

            for key_to_delete in keys_to_delete {
                if let Err(e) = after_state.delete_asset(key_to_delete) {
                    if first_error.is_none() {
                        first_error = Some(Err(e));
                    }
                }
            }

            // Overwrite pre-existing assets back to the previous state (before_state only contains
            // assets that were tracked and were pre-existing)
            for (asset_id, _asset) in self.before_state.assets() {
                if let Err(e) = after_state.copy_from(&self.before_state, *asset_id) {
                    if first_error.is_none() {
                        first_error = Some(Err(e));
                    }
                }
            }

            // before state will be cleared
            self.tracked_assets.clear();
        }

        self.before_state = Default::default();
        self.context_name = None;

        first_error.unwrap_or(Ok(()))
    }

    pub(crate) fn commit_context(
        &mut self,
        after_state: &DataSet,
        modified_assets: &mut HashSet<AssetId>,
        modified_locations: &mut HashSet<AssetLocation>,
    ) {
        if !self.tracked_assets.is_empty() {
            // Make a diff and send it if it has changes
            let diff_set = DataSetDiffSet::diff_data_set(
                &self.before_state,
                &after_state,
                &self.tracked_assets,
            );
            if diff_set.has_changes() {
                //println!("Sending change {:#?}", diff_set);

                //
                // Use diff to append to the modified asset/location sets
                //
                modified_assets.extend(diff_set.modified_assets.iter());

                // Can't use extend because we need to clone
                for modified_location in &diff_set.modified_locations {
                    if !modified_locations.contains(modified_location) {
                        modified_locations.insert(modified_location.clone());
                    }
                }

                //
                // Send the undo command
                //
                self.completed_undo_context_tx
                    .send(CompletedUndoContextMessage {
                        edit_context_key: self.edit_context_key,
                        diff_set,
                    })
                    .unwrap();
            }

            self.tracked_assets.clear();
        }

        self.before_state = Default::default();
        self.context_name = None;
    }
}

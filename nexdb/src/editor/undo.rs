use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use slotmap::{DenseSlotMap, SlotMap};

use crate::edit_context::EditContext;
use crate::{DataSet, DataSetDiffSet, EditContextKey, HashSet, ObjectId, ObjectLocation, SchemaSet};

//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.

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
    ) {
        // If we have any incoming steps, consume them now
        self.drain_rx();

        // if we undo the first step in the chain (i.e. our undo index is currently 1), we want to
        // use the revert diff in the 0th index of the chain
        if self.current_undo_index > 0 {
            if let Some(current_step) = self.undo_chain.get(self.current_undo_index - 1) {
                let edit_context = edit_contexts.get_mut(current_step.edit_context_key).unwrap();
                // We don't want anything being written to the undo context at this point, since we're using it
                edit_context.cancel_pending_undo_context();
                let schema_set = edit_context.schema_set().clone();
                current_step.diff_set.revert_diff.apply(&mut edit_context.data_set, &schema_set);
                // Marks the objects as changed
                current_step.diff_set.revert_diff.get_modified_objects(&mut edit_context.modified_objects);
                self.current_undo_index -= 1;
            }
        }
    }

    pub fn redo(
        &mut self,
        edit_contexts: &mut DenseSlotMap<EditContextKey, EditContext>,
    ) {
        // If we have any incoming steps, consume them now
        self.drain_rx();

        // if we redo the first step in the chain (i.e. our undo index is currently 0), we want to
        // use the apply diff in the 0th index of the chain. If our current step is == length of
        // chain, we have no more steps available to redo
        if let Some(current_step) = self.undo_chain.get(self.current_undo_index) {
            let edit_context = edit_contexts.get_mut(current_step.edit_context_key).unwrap();
            // We don't want anything being written to the undo context at this point, since we're using it
            edit_context.cancel_pending_undo_context();
            let schema_set = edit_context.schema_set().clone();
            current_step.diff_set.apply_diff.apply(&mut edit_context.data_set, &schema_set);
            // Marks object as changed
            current_step.diff_set.apply_diff.get_modified_objects(&mut edit_context.modified_objects);
            self.current_undo_index += 1;
        }
    }
}

// Transaction that holds exclusive access for the data and will directly commit changes. It can
// compare directly against the original dataset for changes
pub struct UndoContext {
    edit_context_key: EditContextKey,
    before_state: DataSet,
    tracked_objects: HashSet<ObjectId>,
    context_name: Option<&'static str>,
    completed_undo_context_tx: Sender<CompletedUndoContextMessage>,
}

impl UndoContext {
    pub(crate) fn new(undo_stack: &UndoStack, edit_context_key: EditContextKey) -> Self {
        UndoContext {
            edit_context_key,
            before_state: Default::default(),
            tracked_objects: Default::default(),
            context_name: Default::default(),
            completed_undo_context_tx: undo_stack.completed_undo_context_tx.clone(),
        }
    }

    // Call after adding a new object
    pub(crate) fn track_new_object(
        &mut self,
        object_id: ObjectId,
    ) {
        if self.context_name.is_some() {
            self.tracked_objects.insert(object_id);
        }
    }

    // Call before editing or deleting an object
    pub(crate) fn track_existing_object(
        &mut self,
        after_state: &DataSet,
        object_id: ObjectId,
    ) {
        if self.context_name.is_some() {
            //TODO: Preserve sub-objects?
            if !self.tracked_objects.contains(&object_id) {
                println!("track object");
                self.tracked_objects.insert(object_id);
                self.before_state.copy_from(&after_state, object_id);
            }
        }
    }

    pub(crate) fn has_open_context(&self) -> bool {
        self.context_name.is_some()
    }

    pub(crate) fn begin_context(
        &mut self,
        after_state: &DataSet,
        name: &'static str,
        modified_objects: &mut HashSet<ObjectId>,
        modified_locations: &mut HashSet<ObjectLocation>
    ) {
        if self.context_name == Some(name) {
            // don't need to do anything, we can append to the current context
        } else {
            // commit the context that's in flight, if one exists
            if self.context_name.is_some() {
                // This won't do anything if there's nothing to send
                self.commit_context(after_state, modified_objects, modified_locations);
            }

            self.context_name = Some(name);
        }
    }

    pub(crate) fn end_context(
        &mut self,
        after_state: &DataSet,
        allow_resume: bool,
        modified_objects: &mut HashSet<ObjectId>,
        modified_locations: &mut HashSet<ObjectLocation>,
    ) {
        if !allow_resume {
            // This won't do anything if there's nothing to send
            self.commit_context(after_state, modified_objects, modified_locations);
        }
    }

    pub(crate) fn cancel_context(
        &mut self,
        after_state: &mut DataSet,
    ) {
        if !self.tracked_objects.is_empty() {
            // Overwrite all the objects in the new set with old data
            let mut objects = Default::default();
            std::mem::swap(&mut objects, &mut self.before_state.objects);
            for (object_id, object) in objects {
                after_state.objects.insert(object_id, object);
            }

            // Delete any tracked objects that aren't in the old data
            after_state.objects.retain(|k, _| {
                self.tracked_objects.contains(k) && !self.before_state.objects.contains_key(k)
            });

            self.tracked_objects.clear();
        }

        self.before_state = Default::default();
        self.context_name = None;
    }

    pub(crate) fn commit_context(
        &mut self,
        after_state: &DataSet,
        modified_objects: &mut HashSet<ObjectId>,
        modified_locations: &mut HashSet<ObjectLocation>,
    ) {
        if !self.tracked_objects.is_empty() {
            // Make a diff and send it if it has changes
            let diff_set = DataSetDiffSet::diff_data_set(
                &self.before_state,
                &after_state,
                &self.tracked_objects,
            );
            if diff_set.has_changes() {
                println!("Sending change {:#?}", diff_set);

                //
                // Use diff to append to the modified object/location sets
                //
                modified_objects.extend(diff_set.modified_objects.iter());

                // Can't use extend because we need to clone
                for modified_location in &diff_set.modified_locations {
                    if !modified_locations.contains(modified_location) {
                        modified_locations.insert(modified_location.clone());
                    }
                }

                //
                // Send the undo command
                //
                self.completed_undo_context_tx.send(CompletedUndoContextMessage {
                    edit_context_key: self.edit_context_key,
                    diff_set
                }).unwrap();
            }

            self.tracked_objects.clear();
        }

        self.before_state = Default::default();
        self.context_name = None;
    }
}

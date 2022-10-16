use std::collections::VecDeque;
use super::{HashMap, HashSet, ObjectId};
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::{DataObjectInfo, DataSet, DataSetDiffSet, HashMapKeys, HashSetIter, NullOverride, ObjectLocation, OverrideBehavior, SchemaFingerprint, SchemaNamedType, SchemaRecord, SchemaSet, Value};
use crate::edit_context::Database;


//TODO: Delete unused property data when path ancestor is null or in replace mode

//TODO: Should we make a struct that refs the schema/data? We could have transactions and databases
// return the temp struct with refs and move all the functions to that

//TODO: Read-only sources? For things like network cache. Could only sync files we edit and overlay
// files source over net cache source, etc.




pub struct UndoStack {
    undo_chain: Vec<DataSetDiffSet>,
    competed_context_rx: Receiver<DataSetDiffSet>,
    competed_context_tx: Sender<DataSetDiffSet>,
}

impl Default for UndoStack {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        UndoStack {
            undo_chain: Default::default(),
            competed_context_tx: tx,
            competed_context_rx: rx
        }
    }
}

impl UndoStack {
    pub fn competed_context_tx(&self) -> &Sender<DataSetDiffSet> {
        &self.competed_context_tx
    }

    fn drain_rx(&mut self) {
        while let Ok(diff) = self.competed_context_rx.try_recv() {
            self.undo_chain.push(diff);
        }
    }

    pub fn undo(&mut self, db: &mut Database) {
        self.drain_rx();

        let popped = self.undo_chain.pop();
        if let Some(popped) = popped {
            popped.revert_diff.apply(&mut db.data_set);
        }
    }

    //TODO: Implement undo/redo properly
}



// Transaction that holds exclusive access for the data and will directly commit changes. It can
// compare directly against the original dataset for changes
pub struct UndoContext {
    before_state: DataSet,
    tracked_objects: HashSet<ObjectId>,
    context_name: Option<&'static str>,
    completed_context_tx: Sender<DataSetDiffSet>
}

impl UndoContext {
    pub(crate) fn new(undo_stack: &UndoStack) -> Self {
        UndoContext {
            before_state: Default::default(),
            tracked_objects: Default::default(),
            context_name: Default::default(),
            completed_context_tx: undo_stack.competed_context_tx.clone(),
        }
    }

    // Call after adding a new object
    pub(crate) fn track_new_object(&mut self, object_id: ObjectId) {
        if self.context_name.is_some() {
            self.tracked_objects.insert(object_id);
        }
    }

    // Call before editing or deleting an object
    pub(crate) fn track_existing_object(&mut self, after_state: &DataSet, object_id: ObjectId) {
        if self.context_name.is_some() {
            //TODO: Preserve sub-objects?
            if !self.tracked_objects.contains(&object_id) {
                println!("track object");
                self.tracked_objects.insert(object_id);
                self.before_state.copy_from(&after_state, object_id);
            }
        }
    }

    pub(crate)  fn has_open_context(&self) -> bool {
        self.context_name.is_some()
    }


    pub(crate) fn begin_context(&mut self, after_state: &DataSet, name: &'static str, modified_objects: &mut HashSet<ObjectId>) {
        if self.context_name == Some(name) {
            // don't need to do anything, we can append to the current context
        } else {
            // commit the context that's in flight, if one exists
            if self.context_name.is_some() {
                // This won't do anything if there's nothing to send
                self.commit_context(after_state, modified_objects);
            }

            self.context_name = Some(name);
        }
    }

    pub(crate) fn end_context(&mut self, after_state: &DataSet, name: &'static str, allow_resume: bool, modified_objects: &mut HashSet<ObjectId>) {
        if !allow_resume {
            // This won't do anything if there's nothing to send
            self.commit_context(after_state, modified_objects);
        }
    }

    pub(crate) fn cancel_context(&mut self, after_state: &mut DataSet) {
        if !self.tracked_objects.is_empty() {
            // Overwrite all the objects in the new set with old data
            let mut objects = Default::default();
            std::mem::swap(&mut objects, &mut self.before_state.objects);
            for (object_id, object) in objects {
                after_state.objects.insert(object_id, object);
            }

            // Delete any tracked objects that aren't in the old data
            after_state.objects.retain(|k, v| self.tracked_objects.contains(k) && !self.before_state.objects.contains_key(k));

            self.before_state = Default::default();
            self.tracked_objects.clear();
        }
    }

    pub(crate) fn commit_context(&mut self, after_state: &DataSet, modified_objects: &mut HashSet<ObjectId>) {
        if !self.tracked_objects.is_empty() {
            // Make a diff and send it if it has changes
            let diff_set = DataSetDiffSet::diff_data_set(&self.before_state, &after_state, &self.tracked_objects);
            if diff_set.has_changes() {
                println!("Sending change {:#?}", diff_set);
                self.completed_context_tx.send(diff_set).unwrap();
            }

            for object in &self.tracked_objects {
                modified_objects.insert(*object);
            }

            self.before_state = Default::default();
            self.tracked_objects.clear();
        }
    }
}
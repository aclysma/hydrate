use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};
use crossbeam_channel::{Receiver, Sender};
use hydrate_model::ObjectId;
use dashmap::DashMap;
use crate::disk_io::DiskAssetIOResult;
use crate::distill_loader::LoadHandle;
use crate::distill_loader::storage::{AssetStorage, HandleOp};

// Sequence of operations:
// * User creates a type-safe handle through an interface, as long as it is alive, the asset remains loaded
//   - Do we load subresources as separate handles or just call an API on the handle? Does it create a
//     new handle?
//   - How do we handle weak/strong requests, and streaming priorities? (Maybe these are sent as events?)
//     Maybe subresources are bitfields of if they are requested?
// * We create a non-type-safe handle internally, the APIs for subresources/streaming priorities are mirrored here
// * There would be some book-keeping to decide what is worth loading in, probably on an update loop.
//   Maybe streaming has to be opted-in by the asset type, and we fast-path around when it's not needed
// * When we decide to load something, we need to know what dependencies to load
//   - Could keep this metadata in memory
//   - Could delegate to the IO subsystem
// * Maybe implies that ref-counting needs to happen at a lower level? But we want load handles for the
//   dependencies so if we explicitly request them, they are reused
// * (Streaming system ideally is a separate thing that can layer on top of non-streaming implementation?)
// * Dependency tracking has to be done by shipped game anyways, so maybe we can't avoid memory for it?
//   - If it's memory mapped file maybe we get lucky and not have to page as much of it in? But this
//     adds latency. Or we can store it in-line with the asset, but then we don't know about it
//     to load until we seek. We could have dependencies of dependencies in the list as well guaranteeing
//     latency is up to two seeks on critical path.
//     - If dependency data is inline with asset, then one change requires updating all of the data
//       for assets that reference it. Alternatively, we can store this metadata in a cache file
//       so we just have to rebuild that one file
//   - How does dependency tracking work with hot reloads?
//   - Maybe we need a fully parallel dependecy tracking manager for hot reloads, and swap them?
//   - Maybe we swap the asset storage too
//   - But then we need to reload *everything* even things that haven't changed
//   - We could allow implementing a migrate?
//   - If our references between objects are pointers/references, and we try to reuse the same memory
//     with loaded object, we can have trouble with new/old version of data pointing at each other
//     (i.e. we have to patch/duplicate objects that didn't change)
// * Should weak/strong handles be in the type system or dynamic?
// * I think we lean into the future being NVMe, dependencies are stored with asset, and we at least
//   bound number of times we go to disk
// * If we accept an upper limit of 32 or 64 subresource, we can use atomics to represent what to load
//   without memory allocation requirements
// * So we get dependency information from IO abstraction as part of the fetch


// Create AssetHandle
// Create LoadHandle
// State machine for loader issues requests to IO to get asset
// IO returns that additional assets are needed (loader adds refs)
//



// States are:
// - Want to load it
// - Data request in flight
// - Waiting for dependencies
// - Initialized
// - Committed

// Mappings needed:
// - LoadHandle -> LoadHandleInfo
// - AssetId -> LoadHandle
// -

pub struct ObjectMetadata {
    dependencies: Vec<ObjectId>,
    subresource_count: u32,
    // size?
}

pub struct ObjectData {
    data: Vec<u8>
}

pub struct RequestMetadataResult {
    pub object_id: ObjectId,
    pub load_handle: LoadHandle,
    pub hash: u64,
    pub result: std::io::Result<ObjectMetadata>
}

pub struct RequestDataResult {
    pub object_id: ObjectId,
    pub load_handle: LoadHandle,
    pub hash: u64,
    pub subresource: Option<u32>,
    pub result: std::io::Result<ObjectData>
}

pub(crate) enum LoaderEvent {
    TryLoad(LoadHandle),
    TryUnload(LoadHandle),
    MetadataRequestComplete(RequestMetadataResult),
    DataRequestComplete(RequestDataResult),
    LoadResult(HandleOp)
}


trait LoaderIO {
    fn request_metadata(&self, object_id: ObjectId);
    //fn request_metadata_results(&self, object_id: ObjectId) -> &Receiver<RequestMetadataResult>;
    fn request_data(&self, object_id: ObjectId, subresource: Option<u32>);
    //fn request_data_results(&self) -> &Receiver<RequestDataResult>;
}

// #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
// struct LoadHandle(u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum LoadState {
    // Not loaded, and we haven't started trying to load it. Ref count > 0 implies we want to start
    // loading.
    Unloaded,
    // Metadata request is in flight
    WaitingForMetadata,
    // We've incremented ref counts for dependencies, but they aren't loaded yet
    WaitingForDependencies,
    // Dependencies are loaded, and we have requested the data required to load this asset
    WaitingForData,
    // Data has been passed off to end-user's loader
    Loading,
    // The engine finished loading the asset but it is not available to the game yet
    // When hot reloading, we delay commit until we have loaded new versions of all changed assets,
    // so engine never sees a partial reload
    Loaded,
    // The asset has been committed and is visible to the game
    Committed,
}

struct LoadHandleInfo {
    //strong_ref_count: AtomicU32,
    object_id: ObjectId,
    ref_count: AtomicU32,
    version_counter: u32,
    load_state: LoadState
}

// impl LoadHandleInfo {
//     pub fn new(object_id: ObjectId) -> Self {
//         LoadHandleInfo {
//             object_id,
//             ref_count: Default::default(),
//             version_counter: 0,
//             load_state: LoadState::Unloaded,
//         }
//     }
// }

struct Loader {
    next_handle_index: AtomicU64,
    load_handle_infos: DashMap<LoadHandle, LoadHandleInfo>,
    object_id_to_handle: DashMap<ObjectId, LoadHandle>,
    loader_io: Box<dyn LoaderIO>,

    events_tx: Sender<LoaderEvent>,
    events_rx: Receiver<LoaderEvent>,

    // Stream of events from end-user's loader implementation to inform loading completed or failed
    //handle_op_tx: Sender<HandleOp>,
    //handle_op_rx: Receiver<HandleOp>,

    // Queue of assets that need to be visited on next update to check for state change
    //object_needs_update_tx: Sender<LoadHandle>,
    //object_needs_update_rx: Receiver<LoadHandle>,
}

impl Loader {
    pub fn new(loader_io: Box<dyn LoaderIO>) -> Self {
        //let (handle_op_tx, handle_op_rx) = crossbeam_channel::unbounded();
        //let (object_needs_update_tx, object_needs_update_rx) = crossbeam_channel::unbounded();
        let (events_tx, events_rx)  = crossbeam_channel::unbounded();

        Loader {
            next_handle_index: AtomicU64::new(1),
            object_id_to_handle: Default::default(),
            load_handle_infos: Default::default(),
            loader_io,
            //handle_op_tx,
            //handle_op_rx,
            //object_needs_update_tx,
            //object_needs_update_rx,
            events_tx,
            events_rx,
        }
    }

    fn handle_try_load(&mut self, load_handle: LoadHandle) {
        if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&load_handle) {
            if load_state_info.load_state == LoadState::Unloaded {
                if load_state_info.ref_count.load(Ordering::Acquire) > 0 {
                    self.loader_io.request_data(load_state_info.object_id, None);
                    load_state_info.load_state = LoadState::WaitingForMetadata;
                } else {
                    // it's not referenced anymore, don't bother loading it. If it becomes
                    // referenced again later, we will get another TryLoad event
                }
            } else {
                // If we are in any other state, then we are loading or already loaded.
                // We don't need to do anything in this case.
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_try_unload(&mut self, load_handle: LoadHandle) {
        if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&load_handle) {
            if load_state_info.load_state != LoadState::Unloaded {
                if load_state_info.ref_count.load(Ordering::Acquire) > 0 {
                    // It's referenced, don't unload it
                } else {
                    //TODO: Unload it from storage?
                    //TODO: Remove ref counts from dependencies?
                    load_state_info.load_state = LoadState::Unloaded;
                }
            } else {
                // We are already unloaded and don't need to do anything
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_request_metadata_result(&mut self, result: RequestMetadataResult) {
        if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&result.load_handle) {
            // Bail if the asset is unloaded
            if load_state_info.load_state == LoadState::Unloaded {
                return;
            }

            // add references for other assets, either wait for dependents metadata or start loading
            let metadata = result.result.unwrap();

            let mut all_are_loaded = false;
            for dependency in &metadata.dependencies {
                let dependency_load_handle = self.add_ref(*dependency);
                let dependency_load_state = self.load_handle_infos.get(&dependency_load_handle).as_ref().unwrap().load_state;
                if dependency_load_state != LoadState::Loaded && dependency_load_state != LoadState::Committed {
                    all_are_loaded = false;
                }
            }

            if all_are_loaded {
                self.loader_io.request_data(load_state_info.object_id, None);
                load_state_info.load_state = LoadState::WaitingForData;
            } else {
                load_state_info.load_state = LoadState::WaitingForDependencies;
                //TODO: Wait for dependencies, maybe by putting all assets in this state into a list
                // so we only poll assets that are in this state
                unimplemented!();
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_request_data_result(&mut self, result: RequestDataResult) {
        if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&result.load_handle) {
            // Bail if the asset is unloaded
            if load_state_info.load_state == LoadState::Unloaded {
                return;
            }

            // start loading
            let data = result.result.unwrap();
        }

    }

    fn handle_load_result(&mut self, load_result: HandleOp) {
        //while let Ok(handle_op) = self.handle_op_rx.try_recv() {
            // Handle the operation
            match load_result {
                HandleOp::Error(load_handle, version, error) => {
                    //TODO: How to handle error?
                    log::error!("load error {}", error);
                    panic!("load error {}", error);
                }
                HandleOp::Complete(load_handle, version) => {
                    // Advance state... maybe we can commit now, otherwise we have to wait until other
                    // dependencies are ready
                }
                HandleOp::Drop(load_handle, version) => {
                    log::error!(
                        "load op dropped without calling complete/error, handle {:?} version {}",
                        load_handle,
                        version
                    );
                    panic!(
                        "load op dropped without calling complete/error, handle {:?} version {}",
                        load_handle, version
                    )
                }
            }
        //}
    }

    pub fn update(&mut self, asset_storage: &mut dyn AssetStorage) {
        while let Ok(loader_event) = self.events_rx.try_recv() {
            match loader_event {
                LoaderEvent::TryLoad(load_handle) => self.handle_try_load(load_handle),
                LoaderEvent::TryUnload(load_handle) => self.handle_try_unload(load_handle),
                LoaderEvent::MetadataRequestComplete(result) => self.handle_request_metadata_result(result),
                LoaderEvent::DataRequestComplete(result) => self.handle_request_data_result(result),
                LoaderEvent::LoadResult(load_result) => self.handle_load_result(load_result),
            }
        }








        // Handle any messages we get back from end-user's load handler. We either find out the asset
        // loaded successfully or that it failed
        //self.process_handle_ops();

        // while let Ok(load_handle) = self.object_needs_update_rx.try_recv() {
        //     if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&load_handle) {
        //         match load_state_info.load_state {
        //             LoadState::Unloaded => {
        //                 // not expected
        //                 if load_state_info.ref_count.load(Ordering::Acquire) > 0 {
        //                     // make the request
        //                     // TODO: Support subresources
        //                     self.loader_io.request_data(load_state_info.object_id, None);
        //                     load_state_info.load_state = LoadState::WaitingForMetadata;
        //                 } else {
        //                     // No refs remain, don't proceed
        //                 }
        //             }
        //             _ => {
        //                 // unexpected
        //             }
        //         }
        //     } else {
        //         // Don't think this is possible but I could see it being needed for certain race conditions.
        //         // Maybe we can't delete load handles, because then we can't retain state such as version counter?
        //         unreachable!();
        //     }
        // }
        //
        // for io_result in self.loader_io.results().try_recv() {
        //     if let Some(load_state_info) = self.load_handle_infos.get_mut(&io_result.load_handle) {
        //         if load_state_info.ref_count.load(Ordering::Acquire) > 0 {
        //             load_state_info.
        //         } else {
        //             // No refs remain, go back to unloaded state
        //             load_state_info.load_state = LoadState::Unloaded;
        //         }
        //     }
        // }


        // For each asset queued to do work
        {
            // Check that we still need to do work (compare current state to desired state)

            // Issue the metadata/asset data request if we don't have it

            // Receive the metadata/asset -> we save metadata (so we can remove refs when it is dropped). Add refs to load dependencies
            // Not sure how to handle metadata changes from asset reloads. It may add refs to existing assets, or new assets

            // Issue load requests for anything left remaining

            // Initialize if ready to be initialized (i.e. hand off to asset storage

            // Commit if ready to be committed

            // Drop if no longer referenced
        }
    }

    fn get_or_insert(&self, object_id: ObjectId) -> LoadHandle {
        *self.object_id_to_handle.entry(object_id).or_insert_with(|| {
            let load_handle_index = self.next_handle_index.fetch_add(1, Ordering::Relaxed);
            let load_handle = LoadHandle(load_handle_index);

            self.load_handle_infos.insert(load_handle, LoadHandleInfo {
                object_id,
                ref_count: AtomicU32::new(0),
                version_counter: 0,
                load_state: LoadState::Unloaded,
            });

            load_handle
        })
    }

    pub fn add_ref(&self, object_id: ObjectId) -> LoadHandle {
        let load_handle = self.get_or_insert(object_id);

        // If the asset is now being loaded, put it in queue to be processed
        let previous_ref_count = self.load_handle_infos.get(&load_handle).as_ref().unwrap().ref_count.fetch_add(1, Ordering::Release);
        if previous_ref_count == 0 {
            self.events_tx.send(LoaderEvent::TryLoad(load_handle)).unwrap();
        }

        load_handle
    }

    pub fn remove_ref(&self, load_handle: LoadHandle) {
        let previous_ref_count = self.load_handle_infos.get(&load_handle).as_ref().unwrap().ref_count.fetch_sub(1, Ordering::Release);

        // If this was the last reference to the asset, put it in queue to be dropped
        if previous_ref_count == 1 {
            self.events_tx.send(LoaderEvent::TryUnload(load_handle)).unwrap();
        }
    }
}

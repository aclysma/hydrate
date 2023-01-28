use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};
use crossbeam_channel::{Receiver, Sender};
use hydrate_model::ObjectId;
use dashmap::DashMap;
use crate::disk_io::DiskAssetIOResult;
use crate::distill_core::{AssetRef, AssetTypeId, AssetUuid};
use crate::distill_loader::{LoadHandle};
use crate::distill_loader::storage::{AssetLoadOp, AssetStorage, HandleOp, LoaderInfoProvider};

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

#[derive(Debug)]
pub struct ObjectMetadata {
    pub dependencies: Vec<ObjectId>,
    pub subresource_count: u32,
    pub asset_type: AssetTypeId
    // size?
}

pub struct ObjectData {
    pub data: Vec<u8>
}

impl std::fmt::Debug for ObjectData {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ObjectData")
            .field("data_length", &self.data.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct RequestMetadataResult {
    pub object_id: ObjectId,
    pub load_handle: LoadHandle,
    //pub hash: u64,
    pub version: u32,
    pub result: std::io::Result<ObjectMetadata>
}

#[derive(Debug)]
pub struct RequestDataResult {
    pub object_id: ObjectId,
    pub load_handle: LoadHandle,
    //pub hash: u64,
    pub version: u32,
    pub subresource: Option<u32>,
    pub result: std::io::Result<ObjectData>
}

#[derive(Copy, Clone, Debug)]
pub struct CombinedBuildHash(pub u64);

#[derive(Debug)]
pub enum LoaderEvent {
    // Sent when asset ref count goes from 0 to 1
    TryLoad(LoadHandle, u32),
    // Sent when asset ref count goes from 1 to 0
    TryUnload(LoadHandle, u32),
    // Sent by LoaderIO when metadata request completes or fails
    MetadataRequestComplete(RequestMetadataResult),
    // Sent when all dependencies that were blocking a load have completed loading
    DependenciesLoaded(LoadHandle, u32),
    // Sent by LoaderIO when data request completes or fails
    DataRequestComplete(RequestDataResult),
    // Sent by engine code to indicate success or failure at loading an asset
    LoadResult(HandleOp),
    // Sent by LoaderIO when there are new versions available of the given assets.
    AssetsUpdated(CombinedBuildHash, Vec<ObjectId>)
}

// do we create a new loader IO for every manifest or same IO?
// - different IO means we can use different IO provider for change data?
// - same IO makes it easier to share connections, memory mapped file,
//
// if same IO we need a way to say what version/hash to load
// we don't have a hash when asking for metadata
//
// we could have:
// - separete loader
// - loader tells us what changed
// -
pub trait LoaderIO: Sync + Send {
    // Returns the latest known build hash that we are currently able to read from
    fn latest_build_hash(&self) -> CombinedBuildHash;

    fn request_metadata(&self, build_hash: CombinedBuildHash, load_handle: LoadHandle, object_id: ObjectId, version: u32);
    //fn request_metadata_results(&self, object_id: ObjectId) -> &Receiver<RequestMetadataResult>;
    fn request_data(&self, build_hash: CombinedBuildHash, load_handle: LoadHandle, object_id: ObjectId, subresource: Option<u32>, version: u32);
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

// we have a manifest hash to identify what build we are on
// we can resolve a hash for the asset with object ID + manifest hash
// - this tells us if it has changed
// object ID still resolves to same load handle after changes are made
// we can assign a sequential version that is bumped when things change - this is what the engine sees
// load handle + hash can be used internally to resolve to the load, assuming we always unload previous version after a change
// we could also just use the hash?
//
// we have a bunch of stuff loaded
// we are notified a new manifest is available
// we pause loading/unloading
// - load/unloads are thrown in a Set, we will have to scan them all later
// - we either kill in-flight loads or wait for them all to complete
// we request hashes for all assets we have loaded
// also need to find any assets that have build dependencies on the modified assets (maybe? example: changing a shader, which requires different descriptor set layout, needs to trigger pipeline rebuild)
// do we need to hash the objects plus build dependency hashes?
//
//
// How to handle ref counts with multiple versions
// - We have an "engine ref count" to indicate explicit requests. Just one per object.
// - We have an "internal ref count" to indicate implicit requests due to other things that are explicitly requested. One per *version*
// - We have a RW lock on LoadHandleInfo, which contains both counts
// - We require READ when engine changes ref count
//   - Fire event when count changes from 0 -> 1 and 1 -> 0 to loading/unloading during update()
// - We require READ when internal changes ref count
//   - Fire event when count changes from 0 -> 1 and 1 -> 0?
//
// OTHER IDEA:
//
// Single engine count and per-version internal count
// We increment the engine count and *current* per-version internal count when external refs are needed
// We increment only the per-version internal count when internal refs are needed
// When we are notified of assets changing, we have to create the new version and init it with the
// engine count.
// Ordinarily we could have a race where we read engine count and write it to per-version internal count,
// while an engine count is being changed. But if we write lock when doing this copy we avoid the race.
// Do we need to pause reacting to engine ref count changes while doing a reload?
// Maybe the engine ref counts are accumulated and ingested on the update()? Or maybe we just do that
// during updates.
// Do we have any hazards taking multiple read locks on dashmap while holding a write lock?
// We stop loading new things while reloading because we only want to handle loading from one manifest at a time
// Dependencies found while loading new objects will kick off additional loads and we don't want to have to look up
// what version of the manifest matches the manifest we are using.
// We could pack engine/internal references together in same U64 using compare and swaps
//
// Maybe we don't need to pause.. we are told there are new versions available for assets
// - We immediately copy ref counts from old version to new version for changed assets
// - Any ref count changes apply to new version
// - So anything that loads will be new version, and can't complete loading until everything is updated in one commit
// - We might choose to not unload anything from old version and just unload it when we make the change.. keeping it
//   loaded is not harmful.

struct LoadHandleVersionInfo {
    // load_state
    // asset_type
    // dependencies
    //object_id: ObjectId,
    load_state: LoadState,
    asset_type: AssetTypeId,
    dependency_ref_count: AtomicU32,

    blocking_dependency_count: AtomicU32,

    // load handle and version for any assets that are waiting on this asset to load in order to continue
    blocked_loads: Vec<(LoadHandle, u32)>,

    // load handle and version for any assets that need to be released when this is unloaded
    dependencies: Vec<(LoadHandle, u32)>,

}

struct LoadHandleInfo {
    //strong_ref_count: AtomicU32,
    object_id: ObjectId,
    asset_id: AssetUuid,
    //ref_count: AtomicU32,
    engine_ref_count: AtomicU32,
    next_version: u32,
    //load_state: LoadState,
    versions: Vec<LoadHandleVersionInfo>,
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

struct ReloadAction {
    build_hash: CombinedBuildHash,
    updated_assets: Vec<ObjectId>,
}

struct LoaderUpdateState {
    current_build_hash: CombinedBuildHash,
    current_reload_action: Option<ReloadAction>,
    pending_reload_actions: Vec<ReloadAction>,
}

pub struct Loader {
    next_handle_index: AtomicU64,
    load_handle_infos: DashMap<LoadHandle, LoadHandleInfo>,
    object_id_to_handle: DashMap<ObjectId, LoadHandle>,
    //current_build_hash: AtomicU64,
    loader_io: Box<dyn LoaderIO>,
    update_state: Mutex<LoaderUpdateState>,

    events_tx: Sender<LoaderEvent>,
    events_rx: Receiver<LoaderEvent>,

    // Stream of events from end-user's loader implementation to inform loading completed or failed
    //handle_op_tx: Sender<HandleOp>,
    //handle_op_rx: Receiver<HandleOp>,

    // Queue of assets that need to be visited on next update to check for state change
    //object_needs_update_tx: Sender<LoadHandle>,
    //object_needs_update_rx: Receiver<LoadHandle>,
}

impl LoaderInfoProvider for Loader {
    fn get_load_handle(&self, id: &AssetRef) -> Option<LoadHandle> {
        let object_id = ObjectId(uuid::Uuid::from_bytes(id.expect_uuid().0).as_u128());
        self.object_id_to_handle.get(&object_id).map(|l| *l)
    }

    fn get_asset_id(&self, load: LoadHandle) -> Option<AssetUuid> {
        self.load_handle_infos.get(&load).map(|l| l.asset_id)
    }
}

impl Loader {
    pub fn new(loader_io: Box<dyn LoaderIO>, events_tx: Sender<LoaderEvent>, events_rx: Receiver<LoaderEvent>) -> Self {
        //let (handle_op_tx, handle_op_rx) = crossbeam_channel::unbounded();
        //let (object_needs_update_tx, object_needs_update_rx) = crossbeam_channel::unbounded();
        //let (events_tx, events_rx)  = crossbeam_channel::unbounded();

        let build_hash = loader_io.latest_build_hash();

        Loader {
            next_handle_index: AtomicU64::new(1),
            object_id_to_handle: Default::default(),
            load_handle_infos: Default::default(),
            //current_build_hash: build_hash,
            update_state: Mutex::new(LoaderUpdateState {
                current_build_hash: build_hash,
                current_reload_action: None,
                pending_reload_actions: vec![]
            }),
            loader_io,
            //handle_op_tx,
            //handle_op_rx,
            //object_needs_update_tx,
            //object_needs_update_rx,
            events_tx,
            events_rx,
        }
    }

    fn handle_try_load(&self, build_hash: CombinedBuildHash, load_handle: LoadHandle, version: usize) {
        log::debug!("handle_try_load {:?}", load_handle);

        // Should always exist, we don't delete load handles
        let mut load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();

        // We expect any try_load requests to be for the latest version. If this ends up not being a
        // valid assertion, perhaps we should just load the most recent version.
        assert_eq!(version, load_state_info.versions.len() - 1);
        let object_id = load_state_info.object_id;
        let current_version = &mut load_state_info.versions[version];
        if current_version.load_state == LoadState::Unloaded {
            // We have not started to load this asset, so we can potentially start it now
            if current_version.dependency_ref_count.load(Ordering::Relaxed) > 0 {
                // The engine is still referencing it, so we should start loading it now
                self.loader_io.request_metadata(build_hash, load_handle, object_id, version as u32);
                current_version.load_state = LoadState::WaitingForMetadata;
            } else {
                // it's not referenced anymore, don't bother loading it. If it becomes
                // referenced again later, we will get another TryLoad event
            }
        } else {
            // If we are in any other state, then we are loading or already loaded.
            // We don't need to do anything in this case.
        }
    }

    fn handle_try_unload(&self, load_handle: LoadHandle, version: usize, asset_storage: &mut dyn AssetStorage) {
        log::debug!("handle_try_unload {:?}", load_handle);

        // Should always exist, we don't delete load handles
        let mut load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();

        let mut dependencies = vec![];

        // We expect any try_unload requests to be for the latest version. If this ends up not being a
        // valid assertion, perhaps we should just unload the most recent version.
        assert_eq!(version, load_state_info.versions.len() - 1);

        let current_version = &mut load_state_info.versions[version];
        if current_version.load_state != LoadState::Unloaded {
            // We are somewhere in the state machine to load the asset, we can stop loading it now
            // if it's no longer referenced
            if current_version.dependency_ref_count.load(Ordering::Relaxed) > 0 {
                // It's referenced, don't unload it
            } else {
                // It's not referenced, so go ahead and unloaded it...

                // If it's been loaded, tell asset storage to drop it
                if current_version.load_state == LoadState::Loading || current_version.load_state == LoadState::Loaded {
                    asset_storage.free(&current_version.asset_type, load_handle, version as u32);
                }

                std::mem::swap(&mut dependencies, &mut current_version.dependencies);

                //TODO: Remove ref counts from dependencies?
                current_version.load_state = LoadState::Unloaded;
            }
        } else {
            // We are already unloaded and don't need to do anything
        }

        // Remove dependency refs, we do this after we finish mutating the load info so that we don't
        // take multiple locks, which risks deadlock
        for (depenency_load_handle, version) in dependencies {
            let mut depenency_load_handle_info = self.load_handle_infos.get_mut(&depenency_load_handle).unwrap();
            self.do_remove_internal_ref(depenency_load_handle, &mut depenency_load_handle_info, version);
        }
    }

    fn handle_request_metadata_result(&self, build_hash: CombinedBuildHash, result: RequestMetadataResult) {
        log::debug!("handle_request_metadata_result {:?}", result.load_handle);

        if let Some(mut load_state_info) = self.load_handle_infos.get(&result.load_handle) {
            let load_state = load_state_info.versions[result.version as usize].load_state;
            // Bail if the asset is unloaded
            if load_state == LoadState::Unloaded {
                return;
            }

            assert_eq!(load_state, LoadState::WaitingForMetadata);
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }

        // add references for other assets, either wait for dependents metadata or start loading
        let metadata = result.result.unwrap();

        let mut blocking_dependency_count = 0;

        let mut dependency_load_handles = vec![];
        for dependency in &metadata.dependencies {
            let dependency_load_handle = self.get_or_insert(*dependency);
            let mut dependency_load_handle_info = self.load_handle_infos.get_mut(&dependency_load_handle).unwrap();

            // We want the latest version - this assumes we add a new version for all changed assets immediately, and only make load requests for latest data
            let version = dependency_load_handle_info.versions.len() as u32 - 1;

            dependency_load_handles.push((dependency_load_handle, version));

            self.do_add_internal_ref(dependency_load_handle, &dependency_load_handle_info, version);

            let load_state = dependency_load_handle_info.versions.last().unwrap().load_state;
            if load_state != LoadState::Loaded && load_state != LoadState::Committed {
                blocking_dependency_count += 1;
            }

            dependency_load_handle_info.versions.last_mut().unwrap().blocked_loads.push((result.load_handle, result.version));
        }

        if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&result.load_handle) {
            let object_id = load_state_info.object_id;
            let version = &mut load_state_info.versions[result.version as usize];
            version.asset_type = metadata.asset_type;
            version.dependencies = dependency_load_handles;

            if blocking_dependency_count == 0 {
                log::debug!("load handle {:?} has no dependencies", result.load_handle);
                self.loader_io.request_data(build_hash, result.load_handle, object_id, None, result.version);
                version.load_state = LoadState::WaitingForData;
            } else {
                log::debug!("load handle {:?} has {} dependencies", result.load_handle, blocking_dependency_count);
                version.blocking_dependency_count = AtomicU32::new(blocking_dependency_count);
                version.load_state = LoadState::WaitingForDependencies;
                //TODO: Wait for dependencies, maybe by putting all assets in this state into a list
                // so we only poll assets that are in this state
                unimplemented!();
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_dependencies_loaded(&self, build_hash: CombinedBuildHash, load_handle: LoadHandle, version: usize) {
        log::debug!("handle_dependencies_loaded {:?}", load_handle);

        //are we still in the correct state?

        let mut load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();
        if load_state_info.versions[version].load_state == LoadState::Unloaded {
            return;
        }

        assert_eq!(load_state_info.versions[version].load_state, LoadState::WaitingForDependencies);

        self.loader_io.request_data(build_hash, load_handle, load_state_info.object_id, None, version as u32);
        load_state_info.versions[version].load_state = LoadState::WaitingForData;
    }

    fn handle_request_data_result(&self, result: RequestDataResult, asset_storage: &mut dyn AssetStorage) {
        log::debug!("handle_request_data_result {:?}", result.load_handle);

        if let Some(mut load_state_info) = self.load_handle_infos.get_mut(&result.load_handle) {
            let version = &mut load_state_info.versions[result.version as usize];
            // Bail if the asset is unloaded
            if version.load_state == LoadState::Unloaded {
                return;
            }

            assert_eq!(version.load_state, LoadState::WaitingForData);

            // start loading
            let data = result.result.unwrap();

            let load_op = AssetLoadOp::new(self.events_tx.clone(), result.load_handle, result.version);
            asset_storage.update_asset(self, &version.asset_type, data.data, result.load_handle, load_op, result.version).unwrap();

            version.load_state = LoadState::Loading;
        }
    }

    fn handle_load_result(&self, load_result: HandleOp, asset_storage: &mut dyn AssetStorage) {
        //while let Ok(handle_op) = self.handle_op_rx.try_recv() {
            // Handle the operation
            match load_result {
                HandleOp::Error(load_handle, version, error) => {
                    log::debug!("handle_load_result error {:?}", load_handle);
                    //TODO: How to handle error?
                    log::error!("load error {}", error);
                    panic!("load error {}", error);
                }
                HandleOp::Complete(load_handle, version) => {
                    log::debug!("handle_load_result complete {:?}", load_handle);
                    // Advance state... maybe we can commit now, otherwise we have to wait until other
                    // dependencies are ready

                    // Flag any loads that were waiting on this load to proceed
                    let mut blocked_loads = Vec::default();
                    {
                        let mut load_handle_info = self.load_handle_infos.get_mut(&load_handle).unwrap();
                        std::mem::swap(&mut blocked_loads, &mut load_handle_info.versions[version as usize].blocked_loads);
                    }

                    for (blocked_load_handle, blocked_load_version) in blocked_loads {
                        let mut blocked_load = self.load_handle_infos.get_mut(&blocked_load_handle).unwrap();
                        let previous_blocked_load_count = blocked_load.versions[blocked_load_version as usize].blocking_dependency_count.fetch_sub(1, Ordering::Relaxed);
                        if previous_blocked_load_count == 1 {
                            // Kick off the blocked load

                        }
                    }


                }
                HandleOp::Drop(load_handle, version) => {
                    log::debug!("handle_load_result drop {:?}", load_handle);
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

    pub fn update(&self, asset_storage: &mut dyn AssetStorage) {
        let mut update_state = self.update_state.lock().unwrap();
        let build_hash = update_state.current_build_hash;

        while let Ok(loader_event) = self.events_rx.try_recv() {
            log::debug!("handle event {:?}", loader_event);
            match loader_event {
                LoaderEvent::TryLoad(load_handle, version) => self.handle_try_load(build_hash, load_handle, version as usize),
                LoaderEvent::TryUnload(load_handle, version) => self.handle_try_unload(load_handle, version as usize, asset_storage),
                LoaderEvent::MetadataRequestComplete(result) => self.handle_request_metadata_result(build_hash, result),
                LoaderEvent::DependenciesLoaded(load_handle, version) => self.handle_dependencies_loaded(build_hash, load_handle, version as usize),
                LoaderEvent::DataRequestComplete(result) => self.handle_request_data_result(result, asset_storage),
                LoaderEvent::LoadResult(load_result) => self.handle_load_result(load_result, asset_storage),
                LoaderEvent::AssetsUpdated(build_hash, updated_assets) => {
                    // We probably want to finish existing work, pause starting new work, and do the reload
                    update_state.pending_reload_actions.push(ReloadAction {
                        build_hash,
                        updated_assets
                    });
                }
            }
        }

        if update_state.current_reload_action.is_none() {
            // Pause ref count changes
            // ref counts need to be for particular versions?
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

            let asset_id = AssetUuid(*uuid::Uuid::from_u128(object_id.0).as_bytes());

            log::debug!("Allocate load handle {:?} for object id {:?}", load_handle, asset_id);

            self.load_handle_infos.insert(load_handle, LoadHandleInfo {
                object_id,
                asset_id,
                engine_ref_count: AtomicU32::new(0),
                next_version: 0,
                //load_state: LoadState::Unloaded,
                versions: vec![
                    LoadHandleVersionInfo {
                        load_state: LoadState::Unloaded,
                        asset_type: AssetTypeId::default(),
                        dependency_ref_count: AtomicU32::new(0),
                        blocking_dependency_count: AtomicU32::new(0),
                        blocked_loads: vec![],
                        dependencies: vec![],
                    },
                ],
            });

            load_handle
        })
    }

    pub fn add_engine_ref(&self, object_id: ObjectId) -> LoadHandle {
        let load_handle = self.get_or_insert(object_id);

        let guard = self.load_handle_infos.get(&load_handle);
        let load_handle_info = guard.as_ref().unwrap();
        load_handle_info.engine_ref_count.fetch_add(1, Ordering::Relaxed);
        // Engine always adjusts the latest version count
        self.do_add_internal_ref(load_handle, load_handle_info, load_handle_info.versions.len() as u32 - 1);

        load_handle
    }

    pub fn add_engine_ref_by_handle(&self, load_handle: LoadHandle) {
        let guard = self.load_handle_infos.get(&load_handle);
        let load_handle_info = guard.as_ref().unwrap();
        load_handle_info.engine_ref_count.fetch_add(1, Ordering::Relaxed);
        // Engine always adjusts the latest version count
        self.do_add_internal_ref(load_handle, load_handle_info, load_handle_info.versions.len() as u32 - 1);
    }

    pub fn remove_engine_ref(&self, load_handle: LoadHandle) {
        let guard = self.load_handle_infos.get(&load_handle);
        let load_handle_info = guard.as_ref().unwrap();
        load_handle_info.engine_ref_count.fetch_sub(1, Ordering::Relaxed);
        // Engine always adjusts the latest version count
        self.do_remove_internal_ref(load_handle, load_handle_info, load_handle_info.versions.len() as u32 - 1);
    }

    // fn add_internal_ref(&self, load_handle: LoadHandle) {
    //     let load_handle = self.get_or_insert(object_id);
    //
    //     let load_handle_info = self.load_handle_infos.get(&load_handle).as_ref().unwrap();
    //
    // }

    fn do_add_internal_ref(&self, load_handle: LoadHandle, load_handle_info: &LoadHandleInfo, version: u32) {
        let previous_ref_count = load_handle_info.versions.last().unwrap().dependency_ref_count.fetch_add(1, Ordering::Relaxed);

        // If this is the first reference to the asset, put it in the queue to be loaded
        if previous_ref_count == 0 {
            self.events_tx.send(LoaderEvent::TryLoad(load_handle, version)).unwrap();
        }
    }


    fn do_remove_internal_ref(&self, load_handle: LoadHandle, load_handle_info: &LoadHandleInfo, version: u32) {
        let previous_ref_count = load_handle_info.versions.last().unwrap().dependency_ref_count.fetch_sub(1, Ordering::Relaxed);

        // If this was the last reference to the asset, put it in queue to be dropped
        if previous_ref_count == 1 {
            self.events_tx.send(LoaderEvent::TryUnload(load_handle, version)).unwrap();
        }
    }
}

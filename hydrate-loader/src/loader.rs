use crate::storage::{AssetLoadOp, AssetStorage, HandleOp, IndirectIdentifier};
use crate::ArtifactTypeId;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::handle::{ArtifactRef, LoadState, LoadStateProvider, LoaderInfoProvider, ResolvedLoadHandle};
use hydrate_base::{ArtifactId, AssetId};
use hydrate_base::{ArtifactManifestData, LoadHandle, StringHash};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use hydrate_base::hashing::HashMap;

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
//   - If our references between assets are pointers/references, and we try to reuse the same memory
//     with loaded asset, we can have trouble with new/old version of data pointing at each other
//     (i.e. we have to patch/duplicate assets that didn't change)
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
pub struct ArtifactMetadata {
    pub dependencies: Vec<ArtifactId>,
    pub asset_type: ArtifactTypeId,
    pub hash: u64,
    // size?
}

pub struct ArtifactData {
    pub data: Vec<u8>,
}

impl std::fmt::Debug for ArtifactData {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ArtifactData")
            .field("data_length", &self.data.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct RequestMetadataResult {
    pub artifact_id: ArtifactId,
    pub load_handle: LoadHandle,
    //pub hash: u64,
    pub result: std::io::Result<ArtifactMetadata>,
}

#[derive(Debug)]
pub struct RequestDataResult {
    pub artifact_id: ArtifactId,
    pub load_handle: LoadHandle,
    //pub hash: u64,
    //pub subresource: Option<u32>,
    pub result: std::io::Result<ArtifactData>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CombinedBuildHash(pub u64);

#[derive(Debug)]
pub enum LoaderEvent {
    // Sent when asset ref count goes from 0 to 1
    TryLoad(LoadHandle),
    // Sent when asset ref count goes from 1 to 0
    TryUnload(LoadHandle),
    // Sent by LoaderIO when metadata request succeeds or fails
    MetadataRequestComplete(RequestMetadataResult),
    // Sent when all dependencies that were blocking a load have completed loading
    DependenciesLoaded(LoadHandle),
    // Sent by LoaderIO when data request succeeds or fails
    DataRequestComplete(RequestDataResult),
    // Sent by engine code to indicate success or failure at loading an asset
    LoadResult(HandleOp),
    // Sent by LoaderIO when there are new versions available of the given assets.
    AssetsUpdated(CombinedBuildHash, Vec<AssetId>),
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

    // How does this work with updating to new versions?
    fn manifest_entry(
        &self,
        artifact_id: ArtifactId,
    ) -> Option<&ArtifactManifestData>;

    fn resolve_indirect(
        &self,
        indirect_identifier: &IndirectIdentifier,
    ) -> Option<&ArtifactManifestData>;

    // This results in a RequestMetadataResult being sent to the loader
    fn request_metadata(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        artifact_id: ArtifactId,
    );

    // This results in a RequestDataResult being sent to the loader
    fn request_data(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        artifact_id: ArtifactId,
        hash: u64,
        //subresource: Option<u32>,
    );
}

// #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
// struct LoadHandle(u64);

// we have a manifest hash to identify what build we are on
// we can resolve a hash for the asset with asset ID + manifest hash
// - this tells us if it has changed
// asset ID still resolves to same load handle after changes are made
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
// do we need to hash the assets plus build dependency hashes?
//
//
// How to handle ref counts with multiple versions
// - We have an "engine ref count" to indicate explicit requests. Just one per asset.
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
// We stop loading new things while reloading because we only want to handle loading from one manifest at a time
// Dependencies found while loading new assets will kick off additional loads and we don't want to have to look up
// what version of the manifest matches the manifest we are using.
// We could pack engine/internal references together in same U64 using compare and swaps
//
// Maybe we don't need to pause.. we are told there are new versions available for assets
// - We immediately copy ref counts from old version to new version for changed assets
// - Any ref count changes apply to new version
// - So anything that loads will be new version, and can't complete loading until everything is updated in one commit
// - We might choose to not unload anything from old version and just unload it when we make the change.. keeping it
//   loaded is not harmful.

// /// Describes the state of an indirect Handle
// #[derive(Copy, Clone, PartialEq, Debug)]
// enum IndirectHandleState {
//     //None,
//     //WaitingForMetadata,
//     //RequestingMetadata,
//     Resolved,
// }

#[derive(Debug)]
struct IndirectLoad {
    id: IndirectIdentifier,
    //state: IndirectHandleState,
    resolved_uuid: ArtifactId,
    engine_ref_count: AtomicUsize,
    //pending_reresolve: bool,

    // Per-build hash state of what to point at
}

struct LoadHandleVersionInfo {
    // load_state
    // asset_type
    // dependencies
    //artifact_id: ArtifactId,
    load_state: LoadState,
    asset_type: ArtifactTypeId,
    hash: u64,
    dependency_ref_count: AtomicU32,

    blocking_dependency_count: AtomicU32,

    // load handle and version for any assets that are waiting on this asset to load in order to continue
    blocked_loads: Vec<LoadHandle>,

    // load handle and version for any assets that need to be released when this is unloaded
    dependencies: Vec<LoadHandle>,
}

struct LoadHandleInfo {
    //strong_ref_count: AtomicU32,
    artifact_id: ArtifactId,
    //asset_id: AssetId,
    //ref_count: AtomicU32,
    engine_ref_count: AtomicU32,
    //load_state: LoadState,
    version: LoadHandleVersionInfo,

    // for debugging/convenience, not actually required
    symbol: Option<StringHash>,
    // for debugging/convenience, not actually required
    debug_name: Option<Arc<String>>,
}

struct ReloadAction {
    _build_hash: CombinedBuildHash,
    _updated_assets: Vec<AssetId>,
}

struct LoaderUpdateState {
    current_build_hash: CombinedBuildHash,
    current_reload_action: Option<ReloadAction>,
    pending_reload_actions: Vec<ReloadAction>,
}

pub struct Loader {
    next_handle_index: AtomicU64,
    load_handle_infos: Mutex<HashMap<LoadHandle, LoadHandleInfo>>,
    // This should only have direct handles, TODO: Rename
    artifact_id_to_handle: Mutex<HashMap<ArtifactId, LoadHandle>>,

    loader_io: Box<dyn LoaderIO>,
    update_state: Mutex<LoaderUpdateState>,

    events_tx: Sender<LoaderEvent>,
    events_rx: Receiver<LoaderEvent>,

    indirect_states: Mutex<HashMap<LoadHandle, IndirectLoad>>,
    indirect_to_load: Mutex<HashMap<IndirectIdentifier, Arc<ResolvedLoadHandle>>>,
}

impl LoadStateProvider for Loader {
    fn load_state(
        &self,
        load_handle: &Arc<ResolvedLoadHandle>,
    ) -> LoadState {
        self.load_handle_infos
            .lock()
            .unwrap()
            .get(&load_handle.direct_load_handle())
            .unwrap()
            .version
            .load_state
    }

    fn artifact_id(
        &self,
        load_handle: &Arc<ResolvedLoadHandle>,
    ) -> ArtifactId {
        self.load_handle_infos.lock().unwrap().get(&load_handle.direct_load_handle()).unwrap().artifact_id
    }
}

#[derive(Copy, Clone)]
struct LoadHandleInfoProviderImpl<'a> {
    artifact_id_to_handle: &'a HashMap<ArtifactId, LoadHandle>,
    load_handle_infos: &'a HashMap<LoadHandle, LoadHandleInfo>,
}

impl<'a> LoaderInfoProvider for LoadHandleInfoProviderImpl<'a> {
    fn load_handle(
        &self,
        id: &ArtifactRef,
    ) -> Option<Arc<ResolvedLoadHandle>> {
        let artifact_id = ArtifactId::from_uuid(id.0.as_uuid());
        let load_handle = self.artifact_id_to_handle.get(&artifact_id).map(|l| *l)?;
        Some(ResolvedLoadHandle::new(load_handle, load_handle))
    }

    fn artifact_id(
        &self,
        load: LoadHandle,
    ) -> Option<ArtifactId> {
        self.load_handle_infos.get(&load).map(|l| l.artifact_id)
    }
}

impl Loader {
    pub(crate) fn symbol(
        &self,
        load: LoadHandle,
    ) -> Option<StringHash> {
        self.load_handle_infos
            .lock()
            .unwrap()
            .get(&load)
            .map(|l| l.symbol.clone())
            .flatten()
    }

    pub(crate) fn debug_name(
        &self,
        load: LoadHandle,
    ) -> Option<Arc<String>> {
        self.load_handle_infos
            .lock()
            .unwrap()
            .get(&load)
            .map(|l| l.debug_name.clone())
            .flatten()
    }

    pub(crate) fn new(
        loader_io: Box<dyn LoaderIO>,
        events_tx: Sender<LoaderEvent>,
        events_rx: Receiver<LoaderEvent>,
    ) -> Self {
        let build_hash = loader_io.latest_build_hash();

        Loader {
            next_handle_index: AtomicU64::new(1),
            artifact_id_to_handle: Default::default(),
            load_handle_infos: Default::default(),
            update_state: Mutex::new(LoaderUpdateState {
                current_build_hash: build_hash,
                current_reload_action: None,
                pending_reload_actions: vec![],
            }),
            loader_io,
            events_tx,
            events_rx,
            indirect_states: Default::default(),
            indirect_to_load: Default::default(),
        }
    }

    fn handle_try_load(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,

    ) {
        // Should always exist, we don't delete load handles
        let mut load_state_info = load_handle_infos.get_mut(&load_handle).unwrap();

        log::debug!(
            "handle_try_load {:?} {:?} {:?}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id
        );

        // We expect any try_load requests to be for the latest version. If this ends up not being a
        // valid assertion, perhaps we should just load the most recent version.
        let artifact_id = load_state_info.artifact_id;
        let current_version = &mut load_state_info.version;
        if current_version.load_state == LoadState::Unloaded {
            // We have not started to load this asset, so we can potentially start it now
            if current_version.dependency_ref_count.load(Ordering::Relaxed) > 0 {
                // The engine is still referencing it, so we should start loading it now
                self.loader_io.request_metadata(
                    build_hash,
                    load_handle,
                    artifact_id,
                );
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

    fn handle_try_unload(
        &self,
        load_handle: LoadHandle,
        asset_storage: &mut dyn AssetStorage,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
    ) {
        // Should always exist, we don't delete load handles
        let mut load_state_info = load_handle_infos.get_mut(&load_handle).unwrap();

        log::debug!(
            "handle_try_unload {:?} {:?} {:?}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id
        );

        let mut dependencies = vec![];

        let current_version = &mut load_state_info.version;
        if current_version.load_state != LoadState::Unloaded {
            // We are somewhere in the state machine to load the asset, we can stop loading it now
            // if it's no longer referenced
            if current_version.dependency_ref_count.load(Ordering::Relaxed) > 0 {
                // It's referenced, don't unload it
            } else {
                // It's not referenced, so go ahead and unloaded it...

                // If it's been loaded, tell asset storage to drop it
                if current_version.load_state == LoadState::Loading
                    || current_version.load_state == LoadState::Loaded
                    || current_version.load_state == LoadState::Committed
                {
                    asset_storage.free(&current_version.asset_type, load_handle);
                }

                std::mem::swap(&mut dependencies, &mut current_version.dependencies);

                current_version.load_state = LoadState::Unloaded;
            }
        } else {
            // We are already unloaded and don't need to do anything
        }

        // Remove dependency refs, we do this after we finish mutating the load info so that we don't
        // take multiple locks, which risks deadlock
        for depenency_load_handle in dependencies {
            let mut depenency_load_handle_info = load_handle_infos
                .get_mut(&depenency_load_handle)
                .unwrap();
            self.do_remove_internal_ref(
                depenency_load_handle,
                &mut depenency_load_handle_info,
            );
        }
    }

    fn handle_request_metadata_result(
        &self,
        build_hash: CombinedBuildHash,
        result: RequestMetadataResult,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
        artifact_id_to_handle: &mut HashMap<ArtifactId, LoadHandle>,
    ) {
        if let Some(load_state_info) = load_handle_infos.get(&result.load_handle) {
            log::debug!(
                "handle_request_metadata_result {:?} {:?} {:?}",
                result.load_handle,
                load_state_info.debug_name,
                load_state_info.artifact_id
            );
            let load_state = load_state_info.version.load_state;
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
            let dependency_load_handle = self.get_or_insert_direct(*dependency, load_handle_infos, artifact_id_to_handle);
            let mut dependency_load_handle_info = load_handle_infos
                .get_mut(&dependency_load_handle)
                .unwrap();

            dependency_load_handles.push(dependency_load_handle);

            self.do_add_internal_ref(
                dependency_load_handle,
                &dependency_load_handle_info,
            );

            let load_state = dependency_load_handle_info
                .version
                .load_state;
            if load_state != LoadState::Loaded && load_state != LoadState::Committed {
                blocking_dependency_count += 1;
            }

            dependency_load_handle_info
                .version
                .blocked_loads
                .push(result.load_handle);
        }

        if let Some(mut load_state_info) = load_handle_infos.get_mut(&result.load_handle) {
            let artifact_id = load_state_info.artifact_id;
            let version = &mut load_state_info.version;
            version.asset_type = metadata.asset_type;
            version.hash = metadata.hash;
            version.dependencies = dependency_load_handles;

            if blocking_dependency_count == 0 {
                log::debug!("load handle {:?} has no dependencies", result.load_handle);
                self.loader_io.request_data(
                    build_hash,
                    result.load_handle,
                    artifact_id,
                    metadata.hash,
                    //None,
                );
                version.load_state = LoadState::WaitingForData;
            } else {
                log::debug!(
                    "load handle {:?} has {} dependencies",
                    result.load_handle,
                    blocking_dependency_count
                );
                version.blocking_dependency_count = AtomicU32::new(blocking_dependency_count);
                version.load_state = LoadState::WaitingForDependencies;
                //TODO: Wait for dependencies, maybe by putting all assets in this state into a list
                // so we only poll assets that are in this state
                //unimplemented!();
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_dependencies_loaded(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
    ) {
        //are we still in the correct state?
        let mut load_state_info = load_handle_infos.get_mut(&load_handle).unwrap();
        log::debug!(
            "handle_dependencies_loaded {:?} {:?} {:?}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id
        );
        if load_state_info.version.load_state == LoadState::Unloaded {
            return;
        }

        assert_eq!(
            load_state_info.version.load_state,
            LoadState::WaitingForDependencies
        );

        self.loader_io.request_data(
            build_hash,
            load_handle,
            load_state_info.artifact_id,
            load_state_info.version.hash,
            //None,
        );
        load_state_info.version.load_state = LoadState::WaitingForData;
    }

    fn handle_request_data_result(
        &self,
        result: RequestDataResult,
        asset_storage: &mut dyn AssetStorage,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
        artifact_id_to_handle: &HashMap<ArtifactId, LoadHandle>,
    ) {
        // Should always exist, we don't delete load handles
        let (load_op, load_state_info, data) = {
            let mut load_state_info = load_handle_infos.get(&result.load_handle).unwrap();
            log::debug!(
                "handle_request_data_result {:?} {:?} {:?}",
                result.load_handle,
                load_state_info.debug_name,
                load_state_info.artifact_id
            );
            let version = &load_state_info.version;
            // Bail if the asset is unloaded
            if version.load_state == LoadState::Unloaded {
                return;
            }

            assert_eq!(version.load_state, LoadState::WaitingForData);

            // start loading
            let data = result.result.unwrap();

            let load_op =
                AssetLoadOp::new(self.events_tx.clone(), result.load_handle);

            (load_op, load_state_info, data)
        };

        let info_provider = LoadHandleInfoProviderImpl {
            artifact_id_to_handle,
            load_handle_infos: &*load_handle_infos,
        };

        // We dropped the load_state_info lock before calling this because the serde deserializer may query for asset
        // references, which can cause deadlocks if we are still holding a lock
        asset_storage
            .update_asset(
                &info_provider,
                &load_state_info.version.asset_type,
                load_state_info.artifact_id,
                data.data,
                result.load_handle,
                load_op,
            )
            .unwrap();

        // Should always exist, we don't delete load handles
        let mut load_state_info = load_handle_infos.get_mut(&result.load_handle).unwrap();
        let version = &mut load_state_info.version;
        version.load_state = LoadState::Loading;
    }

    fn handle_load_result(
        &self,
        load_result: HandleOp,
        asset_storage: &mut dyn AssetStorage,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
    ) {
        //while let Ok(handle_op) = self.handle_op_rx.try_recv() {
        // Handle the operation
        match load_result {
            HandleOp::Error(load_handle, error) => {
                let asset_info = load_handle_infos.get(&load_handle).unwrap();
                log::debug!(
                    "handle_load_result error {:?} {:?} {:?}",
                    load_handle,
                    asset_info.debug_name,
                    asset_info.artifact_id
                );
                //TODO: How to handle error?
                log::error!("load error {}", error);
                panic!("load error {}", error);
            }
            HandleOp::Complete(load_handle) => {
                // Advance state... maybe we can commit now, otherwise we have to wait until other
                // dependencies are ready

                // Flag any loads that were waiting on this load to proceed
                let mut blocked_loads = Vec::default();
                let asset_type = {
                    let mut load_handle_info =
                        load_handle_infos.get_mut(&load_handle).unwrap();
                    log::debug!(
                        "handle_load_result complete {:?} {:?} {:?}",
                        load_handle,
                        load_handle_info.debug_name,
                        load_handle_info.artifact_id,
                    );
                    std::mem::swap(
                        &mut blocked_loads,
                        &mut load_handle_info.version.blocked_loads,
                    );
                    load_handle_info.version.load_state = LoadState::Loaded;
                    load_handle_info.version.asset_type
                };

                for blocked_load_handle in blocked_loads {
                    log::trace!("blocked load {:?}", blocked_load_handle);
                    let blocked_load = load_handle_infos
                        .get_mut(&blocked_load_handle)
                        .unwrap();
                    let previous_blocked_load_count = blocked_load.version
                        .blocking_dependency_count
                        .fetch_sub(1, Ordering::Relaxed);
                    if previous_blocked_load_count == 1 {
                        // Kick off the blocked load
                        self.events_tx
                            .send(LoaderEvent::DependenciesLoaded(
                                blocked_load_handle,
                            ))
                            .unwrap();
                    }
                }

                //TODO: Delay commit until everything is ready?
                asset_storage.commit_asset_version(&asset_type, load_handle);
                load_handle_infos
                    .get_mut(&load_handle)
                    .unwrap()
                    .version
                    .load_state = LoadState::Committed;
            }
            HandleOp::Drop(load_handle) => {
                log::debug!("handle_load_result drop {:?}", load_handle);
                log::error!(
                    "load op dropped without calling complete/error, handle {:?}",
                    load_handle,
                );
                panic!(
                    "load op dropped without calling complete/error, handle {:?}",
                    load_handle
                )
            }
        }
        //}
    }

    #[profiling::function]
    pub(crate) fn update(
        &self,
        asset_storage: &mut dyn AssetStorage,
    ) {
        let mut update_state = self.update_state.lock().unwrap();
        let mut load_handle_infos = self.load_handle_infos.lock().unwrap();
        let mut artifact_id_to_handle = self.artifact_id_to_handle.lock().unwrap();
        let build_hash = update_state.current_build_hash;

        while let Ok(loader_event) = self.events_rx.try_recv() {
            log::debug!("handle event {:?}", loader_event);
            match loader_event {
                LoaderEvent::TryLoad(load_handle) => {
                    self.handle_try_load(build_hash, load_handle, &mut *load_handle_infos)
                }
                LoaderEvent::TryUnload(load_handle) => {
                    self.handle_try_unload(load_handle, asset_storage, &mut *load_handle_infos)
                }
                LoaderEvent::MetadataRequestComplete(result) => {
                    self.handle_request_metadata_result(build_hash, result, &mut *load_handle_infos, &mut * artifact_id_to_handle)
                }
                LoaderEvent::DependenciesLoaded(load_handle) => {
                    self.handle_dependencies_loaded(build_hash, load_handle, &mut *load_handle_infos)
                }
                LoaderEvent::DataRequestComplete(result) => {
                    self.handle_request_data_result(result, asset_storage, &mut *load_handle_infos, &*artifact_id_to_handle)
                }
                LoaderEvent::LoadResult(load_result) => {
                    self.handle_load_result(load_result, asset_storage, &mut *load_handle_infos)
                }
                LoaderEvent::AssetsUpdated(build_hash, updated_assets) => {
                    // We probably want to finish existing work, pause starting new work, and do the reload
                    update_state.pending_reload_actions.push(ReloadAction {
                        _build_hash: build_hash,
                        _updated_assets: updated_assets,
                    });
                }
            }
        }

        if update_state.current_reload_action.is_none() {
            // Pause ref count changes
            // ref counts need to be for particular versions?
        }
    }

    fn allocate_load_handle(
        &self,
        is_indirect: bool,
    ) -> LoadHandle {
        let load_handle_index = self.next_handle_index.fetch_add(1, Ordering::Relaxed);
        LoadHandle::new(load_handle_index, is_indirect)
    }

    // This returns a ResolvedLoadHandle which is either already pointing at a direct load or will need
    // to be populated with a direct load
    fn get_or_insert_indirect(
        &self,
        indirect_id: &IndirectIdentifier,
        indirect_states: &mut HashMap<LoadHandle, IndirectLoad>,
        indirect_to_load: &mut HashMap<IndirectIdentifier, Arc<ResolvedLoadHandle>>,
    ) -> Arc<ResolvedLoadHandle> {
        indirect_to_load
            .entry(indirect_id.clone())
            .or_insert_with(|| {
                let load_handle = self.allocate_load_handle(true);

                let resolved = self.loader_io.resolve_indirect(indirect_id);
                if resolved.is_none() {
                    panic!("Couldn't find asset {:?}", indirect_id);
                }

                let manifest_entry = resolved.unwrap();
                log::debug!(
                    "Allocate indirect load handle {:?} for indirect id {:?} -> {:?}",
                    load_handle,
                    &indirect_id,
                    manifest_entry.artifact_id
                );

                let resolved_load_handle = ResolvedLoadHandle::new(load_handle, LoadHandle(0));

                indirect_states.insert(
                    load_handle,
                    IndirectLoad {
                        id: indirect_id.clone(),
                        resolved_uuid: manifest_entry.artifact_id,
                        engine_ref_count: AtomicUsize::new(0),
                    },
                );
                resolved_load_handle
            }).clone()
    }

    fn get_or_insert_direct(
        &self,
        artifact_id: ArtifactId,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
        artifact_id_to_handle: &mut HashMap<ArtifactId, LoadHandle>,
    ) -> LoadHandle {
            *artifact_id_to_handle
            .entry(artifact_id)
            .or_insert_with(|| {
                let load_handle = self.allocate_load_handle(false);
                let manifest_entry = self.loader_io.manifest_entry(artifact_id).unwrap();

                log::debug!(
                    "Allocate load handle {:?} for artifact id {:?}",
                    load_handle,
                    artifact_id,
                );

                load_handle_infos.insert(
                    load_handle,
                    LoadHandleInfo {
                        artifact_id,
                        //asset_id: manifest_entry.,
                        engine_ref_count: AtomicU32::new(0),
                        //load_state: LoadState::Unloaded,
                        version: LoadHandleVersionInfo {
                            load_state: LoadState::Unloaded,
                            asset_type: ArtifactTypeId::default(),
                            hash: 0,
                            dependency_ref_count: AtomicU32::new(0),
                            blocking_dependency_count: AtomicU32::new(0),
                            blocked_loads: vec![],
                            dependencies: vec![],
                        },
                        symbol: manifest_entry.symbol_hash.clone(),
                        debug_name: manifest_entry.debug_name.clone(),
                    },
                );

                load_handle
            })
    }

    // // from add_refs
    // fn add_direct_engine_ref(
    //     &self,
    //     artifact_id: ArtifactId,
    //     load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
    //     artifact_id_to_handle: &mut HashMap<ArtifactId, LoadHandle>,
    // ) -> LoadHandle {
    //     let load_handle = self.get_or_insert_direct(artifact_id, load_handle_infos, artifact_id_to_handle);
    //     self.do_add_engine_ref_by_handle(load_handle, load_handle_infos, artifact_id_to_handle);
    //     load_handle
    // }

    fn do_add_engine_ref_indirect(
        &self,
        id: IndirectIdentifier,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
        artifact_id_to_handle: &mut HashMap<ArtifactId, LoadHandle>,
        indirect_states: &mut HashMap<LoadHandle, IndirectLoad>,
        indirect_to_load: &mut HashMap<IndirectIdentifier, Arc<ResolvedLoadHandle>>,
    ) -> Arc<ResolvedLoadHandle> {
        let indirect_load_handle = self.get_or_insert_indirect(&id, indirect_states, indirect_to_load);

        // It's possible this has already been resolved, but we nee to make certain we add the appropriate
        // ref count
        let direct_load_handle = self.do_add_engine_ref_by_handle_indirect(
            indirect_load_handle.id,
            load_handle_infos,
            artifact_id_to_handle,
            indirect_states
        );

        let direct_load_test = indirect_load_handle.direct_load_handle.swap(direct_load_handle.0, Ordering::Relaxed);

        // Check that the resolved load handle was either unset or is consistent
        assert!(direct_load_test == 0 || direct_load_test == direct_load_handle.0);

        indirect_load_handle
    }

    pub(crate) fn add_engine_ref_indirect(
        &self,
        id: IndirectIdentifier,
    ) -> Arc<ResolvedLoadHandle> {
        self.do_add_engine_ref_indirect(
            id,
            &mut *self.load_handle_infos.lock().unwrap(),
            &mut *self.artifact_id_to_handle.lock().unwrap(),
            &mut *self.indirect_states.lock().unwrap(),
            &mut *self.indirect_to_load.lock().unwrap(),
        )
    }

    // from add_ref_handle
    // Returns the direct load handle
    fn do_add_engine_ref_by_handle_indirect(
        &self,
        load_handle: LoadHandle,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
        artifact_id_to_handle: &mut HashMap<ArtifactId, LoadHandle>,
        indirect_states: &HashMap<LoadHandle, IndirectLoad>,
    ) -> LoadHandle {
        assert!(load_handle.is_indirect());
        let state = indirect_states.get(&load_handle).unwrap();
        state.engine_ref_count.fetch_add(1, Ordering::Relaxed);

        let direct_load_handle = self.get_or_insert_direct(state.resolved_uuid, load_handle_infos, artifact_id_to_handle);
        self.do_add_engine_ref_by_handle_direct(direct_load_handle, load_handle_infos);
        direct_load_handle
    }

    // from add_ref_handle
    // Returns the direct load handle
    fn do_add_engine_ref_by_handle_direct(
        &self,
        load_handle: LoadHandle,
        load_handle_infos: &mut HashMap<LoadHandle, LoadHandleInfo>,
    ) -> LoadHandle {
        assert!(!load_handle.is_indirect());
        let guard = load_handle_infos.get(&load_handle);
        let load_handle_info = guard.as_ref().unwrap();
        load_handle_info
            .engine_ref_count
            .fetch_add(1, Ordering::Relaxed);
        // Engine always adjusts the latest version count
        //TODO: Don't understand this, probably break when there are multiple versions
        self.do_add_internal_ref(
            load_handle,
            load_handle_info,
        );

        load_handle
    }

    pub(crate) fn add_engine_ref_by_handle(
        &self,
        load_handle: LoadHandle,
    ) -> LoadHandle {
        if load_handle.is_indirect() {
            self.do_add_engine_ref_by_handle_indirect(
                load_handle,
                &mut *self.load_handle_infos.lock().unwrap(),
                &mut *self.artifact_id_to_handle.lock().unwrap(),
                &self.indirect_states.lock().unwrap(),
            )
        } else {
            self.do_add_engine_ref_by_handle_direct(
                load_handle,
                &mut *self.load_handle_infos.lock().unwrap(),
            )
        }
    }

    fn remove_engine_ref_indirect(
        &self,
        load_handle: LoadHandle,
        load_handle_infos: &HashMap<LoadHandle, LoadHandleInfo>,
        artifact_id_to_handle: &HashMap<ArtifactId, LoadHandle>,
        indirect_states: &HashMap<LoadHandle, IndirectLoad>,
    ) {
        let state = indirect_states.get(&load_handle).unwrap();
        state.engine_ref_count.fetch_sub(1, Ordering::Relaxed);
        let resolved_uuid = state.resolved_uuid;
        drop(state);
        let load_handle = *artifact_id_to_handle.get(&resolved_uuid).unwrap();
        self.remove_engine_ref_direct(load_handle, load_handle_infos);
    }

    fn remove_engine_ref_direct(
        &self,
        load_handle: LoadHandle,
        load_handle_infos: &HashMap<LoadHandle, LoadHandleInfo>,
    ) {
        let guard = load_handle_infos.get(&load_handle);
        let load_handle_info = guard.as_ref().unwrap();
        load_handle_info
            .engine_ref_count
            .fetch_sub(1, Ordering::Relaxed);

        // Engine always adjusts the latest version count
        self.do_remove_internal_ref(
            load_handle,
            load_handle_info,
        );
    }

    // from remove_refs
    pub(crate) fn remove_engine_ref(
        &self,
        load_handle: LoadHandle,
    ) {
        let mut load_handle_infos = self.load_handle_infos.lock().unwrap();
        if load_handle.is_indirect() {
            let mut artifact_id_to_handle = self.artifact_id_to_handle.lock().unwrap();
            let mut indirect_states = self.indirect_states.lock().unwrap();
            self.remove_engine_ref_indirect(load_handle, &*load_handle_infos, &*artifact_id_to_handle, &*indirect_states);
        } else {
            self.remove_engine_ref_direct(load_handle, &*load_handle_infos);
        }
    }

    fn do_add_internal_ref(
        &self,
        load_handle: LoadHandle,
        load_handle_info: &LoadHandleInfo,
    ) {
        assert!(!load_handle.is_indirect());
        let previous_ref_count = load_handle_info
            .version
            .dependency_ref_count
            .fetch_add(1, Ordering::Relaxed);

        // If this is the first reference to the asset, put it in the queue to be loaded
        if previous_ref_count == 0 {
            self.events_tx
                .send(LoaderEvent::TryLoad(load_handle))
                .unwrap();
        }
    }

    fn do_remove_internal_ref(
        &self,
        load_handle: LoadHandle,
        load_handle_info: &LoadHandleInfo,
    ) {
        assert!(!load_handle.is_indirect());
        let previous_ref_count = load_handle_info
            .version
            .dependency_ref_count
            .fetch_sub(1, Ordering::Relaxed);

        // If this was the last reference to the asset, put it in queue to be dropped
        if previous_ref_count == 1 {
            self.events_tx
                .send(LoaderEvent::TryUnload(load_handle))
                .unwrap();
        }
    }

    /// Returns handles to all active asset loads.
    pub fn get_active_loads(&self) -> Vec<LoadHandle> {
        let mut loading_handles = Vec::default();
        let load_handle_infos = self.load_handle_infos.lock().unwrap();
        for (k, v) in &*load_handle_infos {
            loading_handles.push(k.clone());
        }

        loading_handles
    }

    pub fn get_load_info(
        &self,
        handle: LoadHandle,
    ) -> Option<LoadInfo> {
        let handle = if handle.is_indirect() {
            let indirect_id = self.indirect_states.lock().unwrap().get(&handle).unwrap().id.clone();
            self.indirect_to_load.lock().unwrap().get(&indirect_id).unwrap().direct_load_handle()
        } else {
            handle
        };

        let mut load_handle_infos = self.load_handle_infos.lock().unwrap();
        let load_info = load_handle_infos.get(&handle)?;
        Some(LoadInfo {
            artifact_id: load_info.artifact_id,
            refs: load_info.engine_ref_count.load(Ordering::Relaxed),
            symbol: load_info.symbol.clone(),
            debug_name: load_info.debug_name.clone(),
            //path: load_info.versions.last().unwrap().
        })
    }
}

/// Information about an asset load operation.
///
/// **Note:** The information is true at the time the `LoadInfo` is retrieved. The actual number of
/// references may change.
#[derive(Debug)]
pub struct LoadInfo {
    /// UUID of the artifact.
    pub artifact_id: ArtifactId,
    /// Number of references to the asset.
    pub refs: u32,
    pub symbol: Option<StringHash>,
    pub debug_name: Option<Arc<String>>,
    // Path to asset's source file. Not guaranteed to always be available.
    //pub path: Option<String>,
    // Name of asset's source file. Not guaranteed to always be available.
    //pub file_name: Option<String>,
    // Asset name. Not guaranteed to always be available.
    //pub asset_name: Option<String>,
}

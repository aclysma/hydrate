use crate::storage::{AssetLoadOp, AssetStorage, HandleOp, IndirectIdentifier};
use crate::ArtifactTypeId;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::handle::{
    ArtifactRef, LoadState, LoadStateProvider, LoaderInfoProvider, ResolvedLoadHandle,
};
use hydrate_base::hashing::HashMap;
use hydrate_base::ArtifactId;
use hydrate_base::{ArtifactManifestData, LoadHandle, StringHash};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

//
// Interface for IO
//
#[derive(Debug)]
pub struct ArtifactMetadata {
    pub dependencies: Vec<ArtifactId>,
    pub artifact_type_id: ArtifactTypeId,
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

//
// Loader events which drive state changes for loaded artifacts
//
#[derive(Debug)]
pub enum LoaderEvent {
    // Sent when artifact ref count goes from 0 to 1
    TryLoad(LoadHandle),
    // Sent when artifact ref count goes from 1 to 0
    TryUnload(LoadHandle),
    // Sent by LoaderIO when metadata request succeeds or fails
    MetadataRequestComplete(RequestMetadataResult),
    // Sent when all dependencies that were blocking a load have completed loading
    DependenciesLoaded(LoadHandle),
    // Sent by LoaderIO when data request succeeds or fails
    DataRequestComplete(RequestDataResult),
    // Sent by engine code to indicate success or failure at loading an artifact
    LoadResult(HandleOp),
    // Sent by LoaderIO when there are new versions available of the given artifacts.
    ArtifactsUpdated(CombinedBuildHash, Vec<ArtifactId>),
}

#[derive(Debug)]
struct IndirectLoad {
    // Identifies what this indirect load refers to. This could be a symbol, and artifact, etc.
    id: IndirectIdentifier,
    // The artifact that the identifier currently maps to. This could change if we reload data.
    resolved_uuid: ArtifactId,
    // The reference count of external handles
    engine_ref_count: AtomicUsize,
}

//TODO: This should probably just be merged into LoadHandleInfo
struct LoadHandleVersionInfo {
    // State this particular artifact is in
    load_state: LoadState,
    artifact_type_id: ArtifactTypeId,
    hash: u64,

    //TODO: Rename to internal ref count
    dependency_ref_count: u32,

    // Number of artifacts that need to finish loading before this artifact can request data and load
    //TODO: Probably doesn't need to be atomic
    blocking_dependency_count: AtomicU32,

    // load handles for any artifacts that are waiting on this artifact to load in order to continue
    blocked_loads: Vec<LoadHandle>,

    // load handles for any artifacts that need to be released when this is unloaded
    dependencies: Vec<LoadHandle>,
}

struct LoadHandleInfo {
    artifact_id: ArtifactId,
    engine_ref_count: AtomicU32,
    version: LoadHandleVersionInfo,

    // for debugging/convenience, not actually required
    symbol: Option<StringHash>,
    // for debugging/convenience, not actually required
    debug_name: Option<Arc<String>>,
}

struct ReloadAction {
    _build_hash: CombinedBuildHash,
    _updated_artifacts: Vec<ArtifactId>,
}

struct LoaderUpdateState {
    current_build_hash: CombinedBuildHash,
    current_reload_action: Option<ReloadAction>,
    pending_reload_actions: Vec<ReloadAction>,
}

struct LoaderInner {
    next_handle_index: AtomicU64,
    // This should only have direct handles
    load_handle_infos: HashMap<LoadHandle, LoadHandleInfo>,
    // This should only have direct handles
    artifact_id_to_handle: HashMap<ArtifactId, LoadHandle>,

    loader_io: Box<dyn LoaderIO>,

    //TODO: Probably don't need to put this into a struct?
    update_state: LoaderUpdateState,

    events_tx: Sender<LoaderEvent>,
    events_rx: Receiver<LoaderEvent>,

    indirect_states: HashMap<LoadHandle, IndirectLoad>,
    indirect_to_load: HashMap<IndirectIdentifier, Arc<ResolvedLoadHandle>>,
}

impl LoaderInner {
    // Process all events, possibly changing load status of artifacts
    // Also commit reload of artifact data if needed
    #[profiling::function]
    fn update(
        &mut self,
        asset_storage: &mut dyn AssetStorage,
    ) {
        let build_hash = self.update_state.current_build_hash;

        while let Ok(loader_event) = self.events_rx.try_recv() {
            log::debug!("handle event {:?}", loader_event);
            match loader_event {
                LoaderEvent::TryLoad(load_handle) => self.handle_try_load(build_hash, load_handle),
                LoaderEvent::TryUnload(load_handle) => {
                    self.handle_try_unload(load_handle, asset_storage)
                }
                LoaderEvent::MetadataRequestComplete(result) => {
                    self.handle_request_metadata_result(build_hash, result)
                }
                LoaderEvent::DependenciesLoaded(load_handle) => {
                    self.handle_dependencies_loaded(build_hash, load_handle)
                }
                LoaderEvent::DataRequestComplete(result) => {
                    self.handle_request_data_result(result, asset_storage)
                }
                LoaderEvent::LoadResult(load_result) => {
                    self.handle_load_result(load_result, asset_storage)
                }
                LoaderEvent::ArtifactsUpdated(build_hash, updated_artifacts) => {
                    // We probably want to finish existing work, pause starting new work, and do the reload
                    self.update_state.pending_reload_actions.push(ReloadAction {
                        _build_hash: build_hash,
                        _updated_artifacts: updated_artifacts,
                    });
                }
            }
        }

        if self.update_state.current_reload_action.is_none() {
            // Pause ref count changes
            // ref counts need to be for particular versions?
        }
    }

    fn handle_try_load(
        &mut self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
    ) {
        // Should always exist, we don't delete load handles
        let load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();

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
            // We have not started to load this artifact, so we can potentially start it now
            if current_version.dependency_ref_count > 0 {
                // The engine is still referencing it, so we should start loading it now
                self.loader_io
                    .request_metadata(build_hash, load_handle, artifact_id);
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
        &mut self,
        load_handle: LoadHandle,
        asset_storage: &mut dyn AssetStorage,
    ) {
        // Should always exist, we don't delete load handles
        let load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();

        log::debug!(
            "handle_try_unload {:?} {:?} {:?}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id
        );

        let mut dependencies = vec![];

        let current_version = &mut load_state_info.version;
        if current_version.load_state != LoadState::Unloaded {
            // We are somewhere in the state machine to load the artifact, we can stop loading it now
            // if it's no longer referenced
            if current_version.dependency_ref_count > 0 {
                // It's referenced, don't unload it
            } else {
                // It's not referenced, so go ahead and unloaded it...

                // If it's been loaded, tell asset storage to drop it
                if current_version.load_state == LoadState::Loading
                    || current_version.load_state == LoadState::Loaded
                    || current_version.load_state == LoadState::Committed
                {
                    asset_storage.free(current_version.artifact_type_id, load_handle);
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
            let depenency_load_handle_info =
                self.load_handle_infos.get_mut(&depenency_load_handle).unwrap();
            Self::do_remove_internal_ref(&self.events_tx, depenency_load_handle, depenency_load_handle_info);
        }
    }

    fn handle_request_metadata_result(
        &mut self,
        build_hash: CombinedBuildHash,
        result: RequestMetadataResult,
    ) {
        if let Some(load_state_info) = self.load_handle_infos.get(&result.load_handle) {
            log::debug!(
                "handle_request_metadata_result {:?} {:?} {:?}",
                result.load_handle,
                load_state_info.debug_name,
                load_state_info.artifact_id
            );
            let load_state = load_state_info.version.load_state;
            // Bail if the artifact is unloaded
            if load_state == LoadState::Unloaded {
                return;
            }

            assert_eq!(load_state, LoadState::WaitingForMetadata);
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }

        // add references for other artifacts, either wait for dependents metadata or start loading
        let metadata = result.result.unwrap();

        let mut blocking_dependency_count = 0;

        let mut dependency_load_handles = vec![];
        for dependency in &metadata.dependencies {
            let dependency_load_handle = self.get_or_insert_direct(*dependency);
            let dependency_load_handle_info = self
                .load_handle_infos
                .get_mut(&dependency_load_handle)
                .unwrap();

            dependency_load_handles.push(dependency_load_handle);

            Self::do_add_internal_ref(
                &self.events_tx,
                dependency_load_handle,
                dependency_load_handle_info,
            );

            let load_state = dependency_load_handle_info.version.load_state;
            if load_state != LoadState::Loaded && load_state != LoadState::Committed {
                blocking_dependency_count += 1;
            }

            dependency_load_handle_info
                .version
                .blocked_loads
                .push(result.load_handle);
        }

        if let Some(load_state_info) = self.load_handle_infos.get_mut(&result.load_handle) {
            let artifact_id = load_state_info.artifact_id;
            let version = &mut load_state_info.version;
            version.artifact_type_id = metadata.artifact_type_id;
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
                //TODO: Wait for dependencies, maybe by putting all artifacts in this state into a list
                // so we only poll artifacts that are in this state
                //unimplemented!();
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_dependencies_loaded(
        &mut self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
    ) {
        //are we still in the correct state?
        let load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();
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
        &mut self,
        result: RequestDataResult,
        asset_storage: &mut dyn AssetStorage,
    ) {
        // Should always exist, we don't delete load handles
        let (load_op, load_state_info, data) = {
            let load_state_info = self.load_handle_infos.get(&result.load_handle).unwrap();
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

            let load_op = AssetLoadOp::new(self.events_tx.clone(), result.load_handle);

            (load_op, load_state_info, data)
        };

        let info_provider = LoadHandleInfoProviderImpl {
            artifact_id_to_handle: &self.artifact_id_to_handle,
            load_handle_infos: &self.load_handle_infos,
        };

        // We dropped the load_state_info lock before calling this because the serde deserializer may query for asset
        // references, which can cause deadlocks if we are still holding a lock
        asset_storage
            .update_asset(
                &info_provider,
                &load_state_info.version.artifact_type_id,
                load_state_info.artifact_id,
                data.data,
                result.load_handle,
                load_op,
            )
            .unwrap();

        // Should always exist, we don't delete load handles
        let load_state_info = self.load_handle_infos.get_mut(&result.load_handle).unwrap();
        let version = &mut load_state_info.version;
        version.load_state = LoadState::Loading;
    }

    fn handle_load_result(
        &mut self,
        load_result: HandleOp,
        asset_storage: &mut dyn AssetStorage,
    ) {
        //while let Ok(handle_op) = self.handle_op_rx.try_recv() {
        // Handle the operation
        match load_result {
            HandleOp::Error(load_handle, error) => {
                let load_handle_info = self.load_handle_infos.get(&load_handle).unwrap();
                log::debug!(
                    "handle_load_result error {:?} {:?} {:?}",
                    load_handle,
                    load_handle_info.debug_name,
                    load_handle_info.artifact_id
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
                let artifact_type_id = {
                    let load_handle_info = self.load_handle_infos.get_mut(&load_handle).unwrap();
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
                    load_handle_info.version.artifact_type_id
                };

                for blocked_load_handle in blocked_loads {
                    log::trace!("blocked load {:?}", blocked_load_handle);
                    let blocked_load = self
                        .load_handle_infos
                        .get_mut(&blocked_load_handle)
                        .unwrap();
                    let previous_blocked_load_count = blocked_load
                        .version
                        .blocking_dependency_count
                        .fetch_sub(1, Ordering::Relaxed);
                    if previous_blocked_load_count == 1 {
                        // Kick off the blocked load
                        self.events_tx
                            .send(LoaderEvent::DependenciesLoaded(blocked_load_handle))
                            .unwrap();
                    }
                }

                //TODO: Delay commit until everything is ready?
                asset_storage.commit_asset_version(artifact_type_id, load_handle);
                self.load_handle_infos
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
    }

    //TODO: Does this need to be a separate function?
    //TODO: Does this need to use an atomic?
    fn allocate_load_handle(
        next_handle_index: &AtomicU64,
        is_indirect: bool,
    ) -> LoadHandle {
        let load_handle_index = next_handle_index.fetch_add(1, Ordering::Relaxed);
        LoadHandle::new(load_handle_index, is_indirect)
    }

    // This returns a ResolvedLoadHandle which is either already pointing at a direct load or will need
    // to be populated with a direct load
    fn get_or_insert_indirect(
        &mut self,
        indirect_id: &IndirectIdentifier,
    ) -> Arc<ResolvedLoadHandle> {
        let next_handle_index = &mut self.next_handle_index;
        let indirect_states = &mut self.indirect_states;
        let loader_io = &mut self.loader_io;
        self.indirect_to_load
            .entry(indirect_id.clone())
            .or_insert_with(|| {
                let indirect_load_handle = Self::allocate_load_handle(next_handle_index, true);

                let resolved = loader_io.resolve_indirect(indirect_id);
                if resolved.is_none() {
                    panic!("Couldn't find asset {:?}", indirect_id);
                }

                let manifest_entry = resolved.unwrap();
                log::debug!(
                    "Allocate indirect load handle {:?} for indirect id {:?} -> {:?}",
                    indirect_load_handle,
                    &indirect_id,
                    manifest_entry.artifact_id
                );

                let resolved_load_handle = ResolvedLoadHandle::new(indirect_load_handle, LoadHandle(0));

                indirect_states.insert(
                    indirect_load_handle,
                    IndirectLoad {
                        id: indirect_id.clone(),
                        resolved_uuid: manifest_entry.artifact_id,
                        engine_ref_count: AtomicUsize::new(0),
                    },
                );
                resolved_load_handle
            })
            .clone()
    }

    fn get_or_insert_direct(
        &mut self,
        artifact_id: ArtifactId,
    ) -> LoadHandle {
        let next_handle_index = &mut self.next_handle_index;
        let load_handle_infos = &mut self.load_handle_infos;
        let loader_io = &mut self.loader_io;
        *self
            .artifact_id_to_handle
            .entry(artifact_id)
            .or_insert_with(|| {
                let direct_load_handle = Self::allocate_load_handle(next_handle_index, false);
                let manifest_entry = loader_io.manifest_entry(artifact_id).unwrap();

                log::debug!(
                    "Allocate load handle {:?} for artifact id {:?}",
                    direct_load_handle,
                    artifact_id,
                );

                load_handle_infos.insert(
                    direct_load_handle,
                    LoadHandleInfo {
                        artifact_id,
                        engine_ref_count: AtomicU32::new(0),
                        version: LoadHandleVersionInfo {
                            load_state: LoadState::Unloaded,
                            artifact_type_id: ArtifactTypeId::default(),
                            hash: 0,
                            dependency_ref_count: 0,
                            blocking_dependency_count: AtomicU32::new(0),
                            blocked_loads: vec![],
                            dependencies: vec![],
                        },
                        symbol: manifest_entry.symbol_hash.clone(),
                        debug_name: manifest_entry.debug_name.clone(),
                    },
                );

                direct_load_handle
            })
    }

    fn do_add_engine_ref_indirect(
        &mut self,
        id: IndirectIdentifier,
    ) -> Arc<ResolvedLoadHandle> {
        let indirect_load_handle = self.get_or_insert_indirect(&id);

        // It's possible this has already been resolved, but we nee to make certain we add the appropriate
        // ref count
        let direct_load_handle = self.do_add_engine_ref_by_handle_indirect(indirect_load_handle.id);

        let direct_load_test = indirect_load_handle
            .direct_load_handle
            .swap(direct_load_handle.0, Ordering::Relaxed);

        // Check that the resolved load handle was either unset or is consistent
        assert!(direct_load_test == 0 || direct_load_test == direct_load_handle.0);

        indirect_load_handle
    }

    // Returns the direct load handle
    fn do_add_engine_ref_by_handle_indirect(
        &mut self,
        indirect_load_handle: LoadHandle,
    ) -> LoadHandle {
        assert!(indirect_load_handle.is_indirect());
        let state = self.indirect_states.get(&indirect_load_handle).unwrap();
        state.engine_ref_count.fetch_add(1, Ordering::Relaxed);

        let direct_load_handle = self.get_or_insert_direct(state.resolved_uuid);
        self.do_add_engine_ref_by_handle_direct(direct_load_handle);
        direct_load_handle
    }

    // Returns the direct load handle
    fn do_add_engine_ref_by_handle_direct(
        &mut self,
        direct_load_handle: LoadHandle,
    ) -> LoadHandle {
        assert!(!direct_load_handle.is_indirect());
        let load_handle_info = self.load_handle_infos.get_mut(&direct_load_handle).unwrap();
        load_handle_info
            .engine_ref_count
            .fetch_add(1, Ordering::Relaxed);
        // Engine always adjusts the latest version count
        //TODO: Don't understand this, probably break when there are multiple versions
        Self::do_add_internal_ref(&self.events_tx, direct_load_handle, load_handle_info);

        direct_load_handle
    }

    fn remove_engine_ref_indirect(
        &mut self,
        indirect_load_handle: LoadHandle,
    ) {
        let state = self.indirect_states.get(&indirect_load_handle).unwrap();
        state.engine_ref_count.fetch_sub(1, Ordering::Relaxed);
        let direct_load_handle = *self
            .artifact_id_to_handle
            .get(&state.resolved_uuid)
            .unwrap();
        self.remove_engine_ref_direct(direct_load_handle);
    }

    fn remove_engine_ref_direct(
        &mut self,
        direct_load_handle: LoadHandle,
    ) {
        let load_handle_info = self.load_handle_infos.get_mut(&direct_load_handle).unwrap();
        load_handle_info
            .engine_ref_count
            .fetch_sub(1, Ordering::Relaxed);

        // Engine always adjusts the latest version count
        Self::do_remove_internal_ref(&self.events_tx, direct_load_handle, load_handle_info);
    }

    // Internal references are only to direct load handles
    fn do_add_internal_ref(
        events_tx: &Sender<LoaderEvent>,
        direct_load_handle: LoadHandle,
        load_handle_info: &mut LoadHandleInfo,
    ) {
        assert!(!direct_load_handle.is_indirect());
        load_handle_info
            .version
            .dependency_ref_count += 1;

        // If this is the first reference to the artifact, put it in the queue to be loaded
        if load_handle_info.version.dependency_ref_count == 1 {
            events_tx.send(LoaderEvent::TryLoad(direct_load_handle)).unwrap();
        }
    }

    // Internal references are only to direct load handles
    fn do_remove_internal_ref(
        events_tx: &Sender<LoaderEvent>,
        direct_load_handle: LoadHandle,
        load_handle_info: &mut LoadHandleInfo,
    ) {
        assert!(!direct_load_handle.is_indirect());
        load_handle_info
            .version
            .dependency_ref_count -= 1;
        // If this was the last reference to the artifact, put it in queue to be dropped
        if load_handle_info.version.dependency_ref_count == 0 {
            events_tx
                .send(LoaderEvent::TryUnload(direct_load_handle))
                .unwrap();
        }
    }

    pub fn get_load_info(
        &self,
        handle: LoadHandle,
    ) -> Option<LoadInfo> {
        let handle = if handle.is_indirect() {
            let indirect_id = self.indirect_states.get(&handle).unwrap().id.clone();
            self.indirect_to_load
                .get(&indirect_id)
                .unwrap()
                .direct_load_handle()
        } else {
            handle
        };

        let load_info = self.load_handle_infos.get(&handle)?;
        Some(LoadInfo {
            artifact_id: load_info.artifact_id,
            refs: load_info.engine_ref_count.load(Ordering::Relaxed),
            symbol: load_info.symbol.clone(),
            debug_name: load_info.debug_name.clone(),
            //path: load_info.versions.last().unwrap().
        })
    }
}

/// Information about an artifact load operation.
///
/// **Note:** The information is true at the time the `LoadInfo` is retrieved. The actual number of
/// references may change.
#[derive(Debug)]
pub struct LoadInfo {
    /// UUID of the artifact.
    pub artifact_id: ArtifactId,
    /// Number of references to the artifact.
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

//
// The Loader acts as a semi-public interface for LoaderInner.
//
#[derive(Clone)]
pub struct Loader {
    inner: Arc<Mutex<LoaderInner>>,
}

impl Loader {
    pub(crate) fn new(
        loader_io: Box<dyn LoaderIO>,
        events_tx: Sender<LoaderEvent>,
        events_rx: Receiver<LoaderEvent>,
    ) -> Self {
        let build_hash = loader_io.latest_build_hash();

        let inner = LoaderInner {
            next_handle_index: AtomicU64::new(1),
            artifact_id_to_handle: Default::default(),
            load_handle_infos: Default::default(),
            update_state: LoaderUpdateState {
                current_build_hash: build_hash,
                current_reload_action: None,
                pending_reload_actions: vec![],
            },
            loader_io,
            events_tx,
            events_rx,
            indirect_states: Default::default(),
            indirect_to_load: Default::default(),
        };

        Loader {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub(crate) fn update(
        &self,
        asset_storage: &mut dyn AssetStorage,
    ) {
        self.inner.lock().unwrap().update(asset_storage);
    }

    pub(crate) fn add_engine_ref_indirect(
        &self,
        id: IndirectIdentifier,
    ) -> Arc<ResolvedLoadHandle> {
        self.inner.lock().unwrap().do_add_engine_ref_indirect(id)
    }

    pub(crate) fn add_engine_ref_by_handle(
        &self,
        load_handle: LoadHandle,
    ) -> LoadHandle {
        if load_handle.is_indirect() {
            self.inner
                .lock()
                .unwrap()
                .do_add_engine_ref_by_handle_indirect(load_handle)
        } else {
            self.inner
                .lock()
                .unwrap()
                .do_add_engine_ref_by_handle_direct(load_handle)
        }
    }

    // from remove_refs
    pub(crate) fn remove_engine_ref(
        &self,
        load_handle: LoadHandle,
    ) {
        if load_handle.is_indirect() {
            self.inner
                .lock()
                .unwrap()
                .remove_engine_ref_indirect(load_handle);
        } else {
            self.inner
                .lock()
                .unwrap()
                .remove_engine_ref_direct(load_handle);
        }
    }

    /// Returns handles to all active artifact loads.
    pub fn get_active_loads(&self) -> Vec<LoadHandle> {
        let mut loading_handles = Vec::default();
        let inner = self.inner.lock().unwrap();
        for k in inner.load_handle_infos.keys() {
            loading_handles.push(k.clone());
        }

        loading_handles
    }

    pub fn get_load_info(
        &self,
        handle: LoadHandle,
    ) -> Option<LoadInfo> {
        self.inner.lock().unwrap().get_load_info(handle)
    }
}

//
// This impl allows a handle in hydrate_base to implement load_state and artifact_id on handle itself,
// proxying the call to this loader
//
impl LoadStateProvider for Loader {
    fn load_state(
        &self,
        load_handle: &Arc<ResolvedLoadHandle>,
    ) -> LoadState {
        self.inner
            .lock()
            .unwrap()
            .load_handle_infos
            .get(&load_handle.direct_load_handle())
            .unwrap()
            .version
            .load_state
    }

    fn artifact_id(
        &self,
        load_handle: &Arc<ResolvedLoadHandle>,
    ) -> ArtifactId {
        self.inner
            .lock()
            .unwrap()
            .load_handle_infos
            .get(&load_handle.direct_load_handle())
            .unwrap()
            .artifact_id
    }
}

//
// This is used by SerdeContext to handle serializing/deserializing artifact references. We always
// provide direct handles when an artifact references another artifact. We ensure that if an artifact
// is loading that references another artifact, that artifact is already loaded. So generally we
// should not fail to find the requested artifact id
//
#[derive(Copy, Clone)]
struct LoadHandleInfoProviderImpl<'a> {
    artifact_id_to_handle: &'a HashMap<ArtifactId, LoadHandle>,
    load_handle_infos: &'a HashMap<LoadHandle, LoadHandleInfo>,
}

impl<'a> LoaderInfoProvider for LoadHandleInfoProviderImpl<'a> {
    // Used when deserializing to convert an artifact id into the load handle of the already-loaded
    // artifact
    fn resolved_load_handle(
        &self,
        id: &ArtifactRef,
    ) -> Option<Arc<ResolvedLoadHandle>> {
        let artifact_id = ArtifactId::from_uuid(id.0.as_uuid());
        let load_handle = self.artifact_id_to_handle.get(&artifact_id).map(|l| *l)?;
        Some(ResolvedLoadHandle::new(load_handle, load_handle))
    }

    // Used when serializing to convert a load handle to an artifact id
    fn artifact_id(
        &self,
        load: LoadHandle,
    ) -> Option<ArtifactId> {
        self.load_handle_infos.get(&load).map(|l| l.artifact_id)
    }
}

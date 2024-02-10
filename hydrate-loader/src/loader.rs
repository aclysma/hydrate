use std::fmt::Formatter;
use std::hash::Hash;
use crate::storage::{AssetLoadOp, AssetStorage, HandleOp, IndirectIdentifier};
use crate::ArtifactTypeId;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::handle::{
    ArtifactRef, LoadState, LoadStateProvider, LoaderInfoProvider, ResolvedLoadHandle,
};
use hydrate_base::hashing::HashMap;
use hydrate_base::ArtifactId;
use hydrate_base::{ArtifactManifestData, LoadHandle, StringHash};
use std::sync::atomic::{Ordering};
use std::sync::{Arc, Mutex};
use hydrate_base::handle::LoadState::WaitingForDependencies;

//
// Interface for IO
//
// Data about artifacts is stored in three places:
// - Manifest: full list of all artifacts, always in-memory
// - Metadata: stored usually at the head of the artifact data itself. Requires an IO operation to read
//   but is lightweight in terms of bytes
// - Data: The actual payload of the artifact
//
// The primary use of metadata is to get a list of other artifacts that must be loaded in order to
// a particular artifact. We will fire off requests for those other artifacts and wait for them to
// complete before requesting payload data
//

// Data about an artifact that is not in the manifest
#[derive(Debug)]
pub struct ArtifactMetadata {
    pub dependencies: Vec<ArtifactId>,
    pub artifact_type_id: ArtifactTypeId,
    pub hash: u64,
    // size?
}

// The actual payload data of an artifact
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

// When IO completes a request for artifact metadata, it will send us a loader event containing this
#[derive(Debug)]
pub struct RequestMetadataResult {
    pub artifact_id: ArtifactId,
    pub load_handle: LoadHandle,
    //pub hash: u64,
    pub result: std::io::Result<ArtifactMetadata>,
}

// When IO completes a request for artifact payload data, it will send us a loader event containing this
#[derive(Debug)]
pub struct RequestDataResult {
    pub artifact_id: ArtifactId,
    pub load_handle: LoadHandle,
    //pub hash: u64,
    //pub subresource: Option<u32>,
    pub result: std::io::Result<ArtifactData>,
}

// A hash of a particular data build. This encompasses everything that was in a single manifest.
// If it changes, we need to check for artifacts that have changed, load them, and update indirect
// handles to point at them. The LoaderIO will provide a new build hash to indicate this has occurred.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ManifestBuildHash(pub u64);

impl std::fmt::Debug for ManifestBuildHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ManifestBuildHash({:0>16x})", self.0)
    }
}

// Represents a data source from which we can load content
pub trait LoaderIO: Sync + Send {
    fn update(&mut self);

    // Returns the latest known build hash that we are currently able to read from
    fn current_build_hash(&self) -> ManifestBuildHash;

    // Build hash that we are prepared to switch to
    fn pending_build_hash(&self) -> Option<ManifestBuildHash>;

    // Switches to using the new manifest for future requests
    fn activate_pending_build_hash(&mut self, new_build_hash: ManifestBuildHash);

    // Provide manifest data for a particular artifact by ID
    fn manifest_entry(
        &self,
        artifact_id: ArtifactId,
    ) -> Option<&ArtifactManifestData>;

    // Provide manifest data for an artifact, determined by indirect identifier (for example a
    // symbol name)
    fn resolve_indirect(
        &self,
        indirect_identifier: &IndirectIdentifier,
    ) -> Option<&ArtifactManifestData>;

    // Load the metadata for an artifact.
    // This results in a RequestMetadataResult being sent to the loader
    fn request_metadata(
        &self,
        build_hash: ManifestBuildHash,
        load_handle: LoadHandle,
        artifact_id: ArtifactId,
    );

    // Load the payload for an artifact.
    // This results in a RequestDataResult being sent to the loader
    fn request_data(
        &self,
        build_hash: ManifestBuildHash,
        load_handle: LoadHandle,
        artifact_id: ArtifactId,
        hash: u64,
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
    //ArtifactsUpdated(ManifestBuildHash),
}

// Information about indirect load handles that have been requested
#[derive(Debug)]
struct IndirectLoad {
    // Identifies what this indirect load refers to. This could be a symbol, and artifact, etc.
    id: IndirectIdentifier,
    // The artifact that the identifier currently maps to. This could change if we reload data.
    //TODO: Update this on reload
    resolved_id_and_hash: Option<ArtifactIdAndHash>,
    // The reference count of external handles (i.e. explicitly requested references, not references
    // due to other artifacts depending on this artifact) matching this indirect identifier
    external_ref_count_indirect: u32,
}

// Information about direct load handles that are currently loaded or were loaded at some point in
// the past. A load handle points to a particular version of an artifact, uniquely identified by
// an artifact ID and the hash.
struct LoadHandleInfo {
    artifact_id: ArtifactId,
    artifact_type_id: ArtifactTypeId,

    // Used to uniquely identify a version of this artifact.
    hash: u64,
    // State this particular artifact is in
    load_state: LoadState,

    // This will be set to true if we reload and this artifact is no longer the latest version of
    // the artifact. Already loaded objects may stay loaded, but we would cancel any further attempts
    // to load this object. (Additionally the currently available manifest data won't be compatible
    // with this asset, so we would not be able to continue loading it)
    //replaced_by_newer_version: bool,

    // The reference count of external handles (i.e. explicitly requested references, not references
    // due to other artifacts depending on this artifact) for this artifact. Indirect handles will
    // count here and it may be that this artifact is referenced by multiple unique indirect handles.
    external_ref_count_direct: u32,

    // Number of references to this artifact, including explicitly via indirect handles or implicitly
    // due to other requested artifacts requiring this artifact to be loaded first. This reference
    // count is the primary determinent of when it's safe to free an artifact from in-memory storage
    internal_ref_count: u32,

    // Number of artifacts that need to finish loading before this artifact can request data and load
    blocking_dependency_count: u32,

    // load handles for any artifacts that are waiting on this artifact to load in order to continue
    blocked_loads: Vec<LoadHandle>,

    // load handles for any artifacts that need to be released when this is unloaded. This artifact
    // implicitly requires these artifacts to load fully before this artifact can finish loading.
    dependencies: Vec<LoadHandle>,

    // for debugging/convenience, not actually required
    symbol: Option<StringHash>,
    // for debugging/convenience, not actually required
    debug_name: Option<Arc<String>>,
}

//TODO: This may need to track the changed artifacts to wait for them to load before updating
// indirect handles and removing ref counts from the direct handles they used to be associated with?
struct ReloadAction {
    // old direct handles, there is no new corresponding handle
    //load_handles_to_unload: Vec<LoadHandle>,
    // new direct handles, we don't need the old handle here because ref count changes will
    // eventually cause the old handle to be dropped
    load_handles_to_reload: Vec<LoadHandle>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct ArtifactIdAndHash {
    id: ArtifactId,
    hash: u64,
}

struct LoaderInner {
    next_handle_index: u64,

    // All direct load handles that are currently loaded or were loaded in the past
    // This should only contain direct handles
    load_handle_infos: HashMap<LoadHandle, LoadHandleInfo>,
    // The direct handle for a given artifact ID
    // This should only contain direct handles
    //TODO: This will get updated on reload
    artifact_id_to_handle: HashMap<ArtifactIdAndHash, LoadHandle>,

    // The data source we will load content from
    loader_io: Box<dyn LoaderIO>,

    // The event queue that drives artifact load states changing. Events are produced by API calls
    // from the game, LoaderIO results, significant changes in reference counts, etc.
    events_tx: Sender<LoaderEvent>,
    events_rx: Receiver<LoaderEvent>,

    // All indirect load handles that exist or previously existed
    indirect_states: HashMap<LoadHandle, IndirectLoad>,
    // All indirect identifiers that exist or previously existed, along with the indirect and direct
    // load handles associated with them
    //TODO: The direct handles will be updated on a reload
    indirect_to_load: HashMap<IndirectIdentifier, Arc<ResolvedLoadHandle>>,

    // Update-specific state, mainly to do with reload detection/handling
    current_build_hash: ManifestBuildHash,
    current_reload_action: Option<ReloadAction>,
}

impl LoaderInner {
    // Process all events, possibly changing load status of artifacts
    // Also commit reload of artifact data if needed
    #[profiling::function]
    fn update(
        &mut self,
        asset_storage: &mut dyn AssetStorage,
    ) {
        self.loader_io.update();

        if let Some(current_reload_action) = &self.current_reload_action {
            //
            // See if this reload has completed yet.
            //
            let mut reload_complete = true;
            for &load_handle in &current_reload_action.load_handles_to_reload {
                let load_handle_info = self.load_handle_infos.get(&load_handle).unwrap();
                if load_handle_info.load_state != LoadState::Committed {
                    reload_complete = false;
                    break;
                }
            }

            if reload_complete {
                log::info!("All artifacts we need to reload are ready, updating indirect handles to point at new data");

                // - Don't forget to handle having forced the artifact to unloaded
                //   - clear the bool flag if reloaded
                //   - make sure events for that artifact are handled cleanly
                for (old_load_handle_indirect, indirect_load) in &mut self.indirect_states {
                    // If the resolved UUID changes, we need to drop ref counts and add ref counts to
                    // direct handles
                    let mut old_artifact_id_and_hash = indirect_load.resolved_id_and_hash;
                    if let Some(new_artifact) = self.loader_io.resolve_indirect(&indirect_load.id) {
                        let new_artifact_id_and_hash = ArtifactIdAndHash {
                            id: new_artifact.artifact_id,
                            hash: new_artifact.combined_build_hash,
                        };

                        if old_artifact_id_and_hash == Some(new_artifact_id_and_hash) {
                            // This artifact did not change, we shouldn't do anything
                            continue;
                        }

                        let new_load_handle_direct = *self.artifact_id_to_handle.get(&new_artifact_id_and_hash).unwrap();
                        let new_load_handle_info = self
                            .load_handle_infos
                            .get_mut(&new_load_handle_direct)
                            .unwrap();

                        // Point the indirect load to the new version
                        indirect_load.resolved_id_and_hash = Some(new_artifact_id_and_hash);
                        let old_load_handle_direct = self.indirect_to_load.get(&indirect_load.id).unwrap().direct_load_handle.swap(new_load_handle_direct.0, Ordering::Relaxed);
                        log::info!("Update indirect handle {:?} => {:?} -> {:?}", indirect_load.id, LoadHandle(old_load_handle_direct), new_load_handle_direct);

                        // Add a direct ref count to new version
                        Self::add_internal_ref(&self.events_tx, new_load_handle_direct, new_load_handle_info);
                    } else {
                        // Point the indirect load to None
                        indirect_load.resolved_id_and_hash = None;
                        let old_load_handle_direct = self.indirect_to_load.get(&indirect_load.id).unwrap().direct_load_handle.swap(0, Ordering::Relaxed);
                        log::info!("Update indirect handle {:?} => {:?} -> {:?}", indirect_load.id, LoadHandle(old_load_handle_direct), LoadHandle(0));
                    }

                    // If we are here, this indirect load handle was changed to point somewhere else.
                    // So drop ref count to old version.
                    if let Some(old_artifact_id_and_hash) = old_artifact_id_and_hash {
                        let old_load_handle_direct = *self.artifact_id_to_handle.get(&old_artifact_id_and_hash).unwrap();
                        let old_load_handle_info = self
                            .load_handle_infos
                            .get_mut(&old_load_handle_direct)
                            .unwrap();
                        log::info!("Remove temporary load handle for {:?}", old_load_handle_direct);
                        Self::remove_internal_ref(&self.events_tx, old_load_handle_direct, old_load_handle_info);
                    }
                }

                //remove temporary ref count added when we started the reload
                //TODO: We might be able to remove this remove_internal_ref and the above add_internal_ref.
                // They are likely redundant.
                for &load_handle in &current_reload_action.load_handles_to_reload {
                    let load_handle_info = self
                        .load_handle_infos
                        .get_mut(&load_handle)
                        .unwrap();
                    log::info!("Remove temporary load handle for {:?}", load_handle);
                    Self::remove_internal_ref(&self.events_tx, load_handle, load_handle_info);
                }

                // indicate that the reload is complete
                log::info!("Finished asset reload, now on manifest build hash {:?}", self.current_build_hash);
                self.current_reload_action = None;
            }
        } else if let Some(pending_build_hash) = self.loader_io.pending_build_hash() {
            //
            // See if there is a new build hash available. If there is, we may need to reload/unload
            // some artifacts.
            //

            let old_build_hash = self.current_build_hash;
            // After this point, we will only have info for the new manifest
            self.loader_io.activate_pending_build_hash(pending_build_hash);
            self.current_build_hash = self.loader_io.current_build_hash();

            log::info!("Begin asset reload {:?} -> {:?}", old_build_hash, self.current_build_hash);

            let mut artifacts_to_reload = vec![];
            let mut load_handles_to_reload = vec![];

            for (indirect_handle, indirect_load) in &self.indirect_states {
                let new_manifest_entry = self.loader_io.resolve_indirect(&indirect_load.id);
                let new_id_and_hash = new_manifest_entry.map(|x| {
                    ArtifactIdAndHash {
                        id: x.artifact_id,
                        hash: x.combined_build_hash
                    }
                });
                let is_still_current_version = indirect_load.resolved_id_and_hash == new_id_and_hash;

                if !is_still_current_version {
                    log::info!("indirect load {:?} is in the new manifest but has changed, hash {:?} -> {:?}", indirect_load.id, indirect_load.resolved_id_and_hash, new_id_and_hash);
                    // Either add the artifact to the reload or unload list
                    if let Some(new_manifest_entry) = &new_manifest_entry {
                        artifacts_to_reload.push(ArtifactIdAndHash {
                            id: new_manifest_entry.artifact_id,
                            hash: new_manifest_entry.combined_build_hash
                        });
                    }
                }

            }

/*
            for (old_load_handle, old_load_handle_info) in &mut self.load_handle_infos {
                //TODO: This seems to try to reload for all artifacts, even old stuff that was already unloaded?
                //[2024-02-09T17:28:46Z INFO  hydrate_loader::loader] artifact 43550966-01bf-4a2c-a362-baa47c6d7e67 is in the new manifest but has changed, hash 1018d949f83fe074 -> dc911224afa08f85
                //[2024-02-09T17:28:46Z INFO  hydrate_loader::loader] artifact 43550966-01bf-4a2c-a362-baa47c6d7e67 is in the new manifest but has changed, hash 1018d949f83fe074 -> dc911224afa08f85
                //[2024-02-09T17:28:46Z INFO  hydrate_loader::loader] artifact a883cfa0-c682-4b8b-b81c-f3c5c260a819 is in the new manifest but has changed, hash 5bca18d5d0b10e38 -> cc89cb6d579f6ff1
                //[2024-02-09T17:28:46Z INFO  hydrate_loader::loader] Add temporary load handle for LoadHandle(2)
                //[2024-02-09T17:28:46Z INFO  hydrate_loader::loader] Add temporary load handle for LoadHandle(2)
                //[2024-02-09T17:28:46Z INFO  hydrate_loader::loader] Add temporary load handle for LoadHandle(7)
                //TODO: Then it seems to add a ref count to the old version??
                //[2024-02-09T17:28:46Z DEBUG hydrate_loader::loader] handle_try_load LoadHandle(2) Some("assets://test_transform_ref") ArtifactId(43550966-01bf-4a2c-a362-baa47c6d7e67) 1018d949f83fe074
                // THis repro *was* reloading to an old version of stuff
                //
                // Also noticed somehow we remove more temporary handles than we add??



                // Check if this artifact needs to reload or unload
                let new_manifest_entry = self.loader_io.manifest_entry(old_load_handle_info.artifact_id);
                let mut is_still_current_version = false;
                if let Some(new_manifest_entry) = &new_manifest_entry {
                    if new_manifest_entry.combined_build_hash == old_load_handle_info.hash {
                        // still exists and matches new manifest
                        is_still_current_version = true;
                    } else {
                        // still exists but no longer latest version
                    }
                } else {
                    // no longer exists
                }

                if !is_still_current_version {
                    // // Mark this load handle as no longer
                    // if old_load_handle_info.load_state != LoadState::Committed {
                    //     old_load_handle_info.replaced_by_newer_version = true;
                    //     //self.events_tx.send(LoaderEvent::TryUnload(*old_load_handle)).unwrap();
                    // }

                    // Either add the artifact to the reload or unload list
                    if let Some(new_manifest_entry) = &new_manifest_entry {
                        log::info!("artifact {} is in the new manifest but has changed, hash {:0>16x} -> {:0>16x}", old_load_handle_info.artifact_id, old_load_handle_info.hash, new_manifest_entry.combined_build_hash);
                        artifacts_to_reload.push(ArtifactIdAndHash {
                            id: new_manifest_entry.artifact_id,
                            hash: new_manifest_entry.combined_build_hash
                        });
                    } else {
                        log::info!("artifact {} is not in the new manifest", old_load_handle_info.artifact_id);
                    }
                }
            }

 */

            // Add temporary ref counts to new version of anything that has changed (causing it to load)
            for new_handle in artifacts_to_reload {
                let new_load_handle = self.get_or_insert_direct(new_handle);
                let new_load_handle_info = self
                    .load_handle_infos
                    .get_mut(&new_load_handle)
                    .unwrap();

                // This reference is temporary and will be removed when we finish the reload
                log::info!("Add temporary load handle for {:?}", new_load_handle);
                Self::add_internal_ref(&self.events_tx, new_load_handle, new_load_handle_info);
                load_handles_to_reload.push(new_load_handle);
            }

            self.current_reload_action = Some(ReloadAction {
                load_handles_to_reload,
            });
        }






        // Don't want to break or interrupt loading for old versions.. we could end up
        // switching back to them
        //
        // Two phases:
        // - Start loading new versions of anything that has changed
        //   - This creates/increments temporary ref counts on load handles
        // - Any further requests for data will be for new versions
        // - When all new data is loaded, we can update indirect handles
        //   - add/remove "real" ref counts for the indirect handles
        //   - remove the temporary ref counts
        //
        // Handling in-flight loads
        // - Mark them as "abandoned"? We have to cleanly handle coming back to them if we
        //   reload data and it matches hash of a cancelled load
        // - Have multiple manifests loaded
        //   - Makes SerdeContext a little uglier


        while let Ok(loader_event) = self.events_rx.try_recv() {
            log::debug!("handle event {:?}", loader_event);
            match loader_event {
                LoaderEvent::TryLoad(load_handle) => self.handle_try_load(self.current_build_hash, load_handle),
                LoaderEvent::TryUnload(load_handle) => {
                    self.handle_try_unload(load_handle, asset_storage)
                }
                LoaderEvent::MetadataRequestComplete(result) => {
                    self.handle_request_metadata_result(self.current_build_hash, result)
                }
                LoaderEvent::DependenciesLoaded(load_handle) => {
                    self.handle_dependencies_loaded(self.current_build_hash, load_handle)
                }
                LoaderEvent::DataRequestComplete(result) => {
                    self.handle_request_data_result(result, asset_storage)
                }
                LoaderEvent::LoadResult(load_result) => {
                    self.handle_load_result(load_result, asset_storage)
                }
            }
        }
    }

    fn handle_try_load(
        &mut self,
        build_hash: ManifestBuildHash,
        load_handle: LoadHandle,
    ) {
        // Should always exist, we don't delete load handles
        let load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();

        log::debug!(
            "handle_try_load {:?} {:?} {:?} {:0>16x}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id,
            load_state_info.hash
        );

        // We expect any try_load requests to be for the latest version. If this ends up not being a
        // valid assertion, perhaps we should just load the most recent version.
        let artifact_id = load_state_info.artifact_id;
        if load_state_info.load_state == LoadState::Unloaded {
            // We have not started to load this artifact, so we can potentially start it now
            if load_state_info.internal_ref_count > 0 {
                // The engine is still referencing it, so we should start loading it now
                self.loader_io
                    .request_metadata(build_hash, load_handle, artifact_id);
                load_state_info.load_state = LoadState::WaitingForMetadata;
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
            "handle_try_unload {:?} {:?} {:?} {:0>16x}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id,
            load_state_info.hash
        );

        let mut dependencies = vec![];

        if load_state_info.load_state != LoadState::Unloaded {
            // We are somewhere in the state machine to load the artifact, we can stop loading it now
            // if it's no longer referenced
            if load_state_info.internal_ref_count > 0 {
                // It's referenced, don't unload it
            } else {
                // It's not referenced, so go ahead and unloaded it...

                // If it's been loaded, tell asset storage to drop it
                if load_state_info.load_state == LoadState::Loading
                    || load_state_info.load_state == LoadState::Loaded
                    || load_state_info.load_state == LoadState::Committed
                {
                    asset_storage.free(load_state_info.artifact_type_id, load_handle);
                }

                std::mem::swap(&mut dependencies, &mut load_state_info.dependencies);

                load_state_info.load_state = LoadState::Unloaded;
            }
        } else {
            // We are already unloaded and don't need to do anything
        }

        // Remove dependency refs, we do this after we finish mutating the load info so that we don't
        // take multiple locks, which risks deadlock
        for depenency_load_handle in dependencies {
            let depenency_load_handle_info =
                self.load_handle_infos.get_mut(&depenency_load_handle).unwrap();
            Self::remove_internal_ref(&self.events_tx, depenency_load_handle, depenency_load_handle_info);
        }
    }

    fn handle_request_metadata_result(
        &mut self,
        build_hash: ManifestBuildHash,
        result: RequestMetadataResult,
    ) {
        if let Some(load_state_info) = self.load_handle_infos.get(&result.load_handle) {
            log::debug!(
                "handle_request_metadata_result {:?} {:?} {:?} {:0>16x}",
                result.load_handle,
                load_state_info.debug_name,
                load_state_info.artifact_id,
                load_state_info.hash
            );
            let load_state = load_state_info.load_state;
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
            let dependency_manifest_entry = self.loader_io.manifest_entry(*dependency).unwrap();

            let dependency_load_handle = self.get_or_insert_direct(ArtifactIdAndHash {
                id: *dependency,
                hash: dependency_manifest_entry.combined_build_hash
            });
            let dependency_load_handle_info = self
                .load_handle_infos
                .get_mut(&dependency_load_handle)
                .unwrap();

            dependency_load_handles.push(dependency_load_handle);

            Self::add_internal_ref(
                &self.events_tx,
                dependency_load_handle,
                dependency_load_handle_info,
            );

            let load_state = dependency_load_handle_info.load_state;
            if load_state != LoadState::Loaded && load_state != LoadState::Committed {
                blocking_dependency_count += 1;
            }

            dependency_load_handle_info
                .blocked_loads
                .push(result.load_handle);
        }

        if let Some(load_state_info) = self.load_handle_infos.get_mut(&result.load_handle) {
            let artifact_id = load_state_info.artifact_id;
            load_state_info.artifact_type_id = metadata.artifact_type_id;
            load_state_info.hash = metadata.hash;
            load_state_info.dependencies = dependency_load_handles;

            if blocking_dependency_count == 0 {
                log::debug!("load handle {:?} has no dependencies", result.load_handle);
                self.loader_io.request_data(
                    build_hash,
                    result.load_handle,
                    artifact_id,
                    metadata.hash,
                    //None,
                );
                load_state_info.load_state = LoadState::WaitingForData;
            } else {
                log::debug!(
                    "load handle {:?} has {} dependencies",
                    result.load_handle,
                    blocking_dependency_count
                );
                load_state_info.blocking_dependency_count = blocking_dependency_count;
                load_state_info.load_state = LoadState::WaitingForDependencies;
                // Processing for this artifact will continue with dependencies load and our
                // blocking_dependency_count hits 0. (It will be decremented as dependencies are
                // loaded in)
            }
        } else {
            // We don't recognize the load_handle.. we currently never delete them so this shouldn't happen
            unreachable!();
        }
    }

    fn handle_dependencies_loaded(
        &mut self,
        build_hash: ManifestBuildHash,
        load_handle: LoadHandle,
    ) {
        //are we still in the correct state?
        let load_state_info = self.load_handle_infos.get_mut(&load_handle).unwrap();
        log::debug!(
            "handle_dependencies_loaded {:?} {:?} {:?} {:0>16x}",
            load_handle,
            load_state_info.debug_name,
            load_state_info.artifact_id,
            load_state_info.hash,
        );
        if load_state_info.load_state == LoadState::Unloaded {
            return;
        }

        assert_eq!(
            load_state_info.load_state,
            LoadState::WaitingForDependencies
        );

        self.loader_io.request_data(
            build_hash,
            load_handle,
            load_state_info.artifact_id,
            load_state_info.hash,
            //None,
        );
        load_state_info.load_state = LoadState::WaitingForData;
    }

    fn handle_request_data_result(
        &mut self,
        result: RequestDataResult,
        asset_storage: &mut dyn AssetStorage,
    ) {
        // if self.artifact_id_to_handle.get(&result.artifact_id).unwrap() != result.artifact_id {
        //     assert!(version.load_state == LoadState::
        //     // This
        //     // let load_state_info = self.load_handle_infos.get_mut(&result.load_handle).unwrap();
        //     // let version = &mut load_state_info.version;
        //     // version.load_state = LoadState::Unloaded
        // }

        // Should always exist, we don't delete load handles
        let (load_op, load_state_info, data) = {
            let load_state_info = self.load_handle_infos.get(&result.load_handle).unwrap();
            log::debug!(
                "handle_request_data_result {:?} {:?} {:?} {:0>16x}",
                result.load_handle,
                load_state_info.debug_name,
                load_state_info.artifact_id,
                load_state_info.hash
            );
            // Bail if the asset is unloaded
            if load_state_info.load_state == LoadState::Unloaded {
                return;
            }

            assert_eq!(load_state_info.load_state, LoadState::WaitingForData);

            // start loading
            let data = result.result.unwrap();

            let load_op = AssetLoadOp::new(self.events_tx.clone(), result.load_handle);

            (load_op, load_state_info, data)
        };

        let info_provider = LoadHandleInfoProviderImpl {
            artifact_id_to_handle: &self.artifact_id_to_handle,
            load_handle_infos: &self.load_handle_infos,
            loader_io: &*self.loader_io,
        };

        // We dropped the load_state_info lock before calling this because the serde deserializer may query for asset
        // references, which can cause deadlocks if we are still holding a lock
        asset_storage
            .update_asset(
                &info_provider,
                &load_state_info.artifact_type_id,
                load_state_info.artifact_id,
                data.data,
                result.load_handle,
                load_op,
            )
            .unwrap();

        // Should always exist, we don't delete load handles
        let load_state_info = self.load_handle_infos.get_mut(&result.load_handle).unwrap();
        load_state_info.load_state = LoadState::Loading;
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
                    "handle_load_result error {:?} {:?} {:?} {:0>16x}",
                    load_handle,
                    load_handle_info.debug_name,
                    load_handle_info.artifact_id,
                    load_handle_info.hash
                );
                //TODO: How to handle errors?
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
                        "handle_load_result complete {:?} {:?} {:?} {:0>16x}",
                        load_handle,
                        load_handle_info.debug_name,
                        load_handle_info.artifact_id,
                        load_handle_info.hash
                    );
                    std::mem::swap(
                        &mut blocked_loads,
                        &mut load_handle_info.blocked_loads,
                    );
                    load_handle_info.load_state = LoadState::Loaded;
                    load_handle_info.artifact_type_id
                };

                for blocked_load_handle in blocked_loads {
                    log::trace!("blocked load {:?}", blocked_load_handle);
                    let blocked_load = self
                        .load_handle_infos
                        .get_mut(&blocked_load_handle)
                        .unwrap();
                    blocked_load
                        .blocking_dependency_count -= 1;
                    if blocked_load.blocking_dependency_count == 0 {
                        // Kick off the blocked load
                        self.events_tx
                            .send(LoaderEvent::DependenciesLoaded(blocked_load_handle))
                            .unwrap();
                    }
                }

                asset_storage.commit_asset_version(artifact_type_id, load_handle);
                self.load_handle_infos
                    .get_mut(&load_handle)
                    .unwrap()
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
                let indirect_load_handle = LoadHandle::new(*next_handle_index, true);
                *next_handle_index += 1;

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
                        resolved_id_and_hash: Some(ArtifactIdAndHash {
                            id: manifest_entry.artifact_id,
                            hash: manifest_entry.combined_build_hash,
                        }),
                        external_ref_count_indirect: 0,
                    },
                );
                resolved_load_handle
            })
            .clone()
    }

    fn get_or_insert_direct(
        &mut self,
        artifact_id_and_hash: ArtifactIdAndHash,
    ) -> LoadHandle {
        let next_handle_index = &mut self.next_handle_index;
        let load_handle_infos = &mut self.load_handle_infos;
        let loader_io = &mut self.loader_io;
        *self
            .artifact_id_to_handle
            .entry(artifact_id_and_hash)
            .or_insert_with(|| {
                let direct_load_handle = LoadHandle::new(*next_handle_index, false);
                *next_handle_index += 1;
                let manifest_entry = loader_io.manifest_entry(artifact_id_and_hash.id).unwrap();
                assert_eq!(manifest_entry.combined_build_hash, artifact_id_and_hash.hash);

                log::debug!(
                    "Allocate load handle {:?} for artifact id {:?}",
                    direct_load_handle,
                    artifact_id_and_hash,
                );

                load_handle_infos.insert(
                    direct_load_handle,
                    LoadHandleInfo {
                        artifact_id: artifact_id_and_hash.id,
                        external_ref_count_direct: 0,
                        load_state: LoadState::Unloaded,
                        artifact_type_id: ArtifactTypeId::default(),
                        hash: artifact_id_and_hash.hash,
                        //replaced_by_newer_version: false,
                        internal_ref_count: 0,
                        blocking_dependency_count: 0,
                        blocked_loads: vec![],
                        dependencies: vec![],
                        symbol: manifest_entry.symbol_hash.clone(),
                        debug_name: manifest_entry.debug_name.clone(),
                    },
                );

                direct_load_handle
            })
    }

    fn add_engine_ref_indirect(
        &mut self,
        id: IndirectIdentifier,
    ) -> Arc<ResolvedLoadHandle> {
        let indirect_load_handle = self.get_or_insert_indirect(&id);

        // It's possible this has already been resolved, but we still need to add a ref count.
        let direct_load_handle = self.add_engine_ref_by_handle_indirect(indirect_load_handle.id);

        // We expect that the direct handle in the ResolvedLoadHandle is either unset (0) or
        // is consistent with the direct handle returned by add_engine_ref_by_handle_indirect().
        // If it's unset, we need to set it.
        let direct_load_test = indirect_load_handle
            .direct_load_handle
            .swap(direct_load_handle.0, Ordering::Relaxed);
        assert!(direct_load_test == 0 || direct_load_test == direct_load_handle.0);

        indirect_load_handle
    }

    // Returns the direct load handle
    fn add_engine_ref_by_handle_indirect(
        &mut self,
        indirect_load_handle: LoadHandle,
    ) -> LoadHandle {
        assert!(indirect_load_handle.is_indirect());
        let mut state = self.indirect_states.get_mut(&indirect_load_handle).unwrap();
        state.external_ref_count_indirect += 1;

        let resolved_id_and_hash = state.resolved_id_and_hash;
        if let Some(resolved_id_and_hash) = resolved_id_and_hash {
            let direct_load_handle = self.get_or_insert_direct(resolved_id_and_hash);
            self.add_engine_ref_by_handle_direct(direct_load_handle);
            direct_load_handle
        } else {
            LoadHandle(0)
        }
    }

    // Returns the direct load handle
    fn add_engine_ref_by_handle_direct(
        &mut self,
        direct_load_handle: LoadHandle,
    ) -> LoadHandle {
        assert!(!direct_load_handle.is_indirect());
        let load_handle_info = self.load_handle_infos.get_mut(&direct_load_handle).unwrap();
        load_handle_info
            .external_ref_count_direct += 1;

        Self::add_internal_ref(&self.events_tx, direct_load_handle, load_handle_info);

        direct_load_handle
    }

    fn remove_engine_ref_indirect(
        &mut self,
        indirect_load_handle: LoadHandle,
    ) {
        let mut state = self.indirect_states.get_mut(&indirect_load_handle).unwrap();
        state.external_ref_count_indirect -= 1;
        if let Some(resolved_id_and_hash) = &state.resolved_id_and_hash {
            let direct_load_handle = *self
                .artifact_id_to_handle
                .get(resolved_id_and_hash)
                .unwrap();
            self.remove_engine_ref_direct(direct_load_handle);
        }
    }

    fn remove_engine_ref_direct(
        &mut self,
        direct_load_handle: LoadHandle,
    ) {
        let load_handle_info = self.load_handle_infos.get_mut(&direct_load_handle).unwrap();
        load_handle_info
            .external_ref_count_direct -= 1;

        // Engine always adjusts the latest version count
        Self::remove_internal_ref(&self.events_tx, direct_load_handle, load_handle_info);
    }

    // Internal references are only to direct load handles
    fn add_internal_ref(
        events_tx: &Sender<LoaderEvent>,
        direct_load_handle: LoadHandle,
        load_handle_info: &mut LoadHandleInfo,
    ) {
        assert!(!direct_load_handle.is_indirect());
        load_handle_info
            .internal_ref_count += 1;

        // If this is the first reference to the artifact, put it in the queue to be loaded
        if load_handle_info.internal_ref_count == 1 {
            events_tx.send(LoaderEvent::TryLoad(direct_load_handle)).unwrap();
        }
    }

    // Internal references are only to direct load handles
    fn remove_internal_ref(
        events_tx: &Sender<LoaderEvent>,
        direct_load_handle: LoadHandle,
        load_handle_info: &mut LoadHandleInfo,
    ) {
        assert!(!direct_load_handle.is_indirect());
        load_handle_info
            .internal_ref_count -= 1;
        // If this was the last reference to the artifact, put it in queue to be dropped
        if load_handle_info.internal_ref_count == 0 {
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
            refs: load_info.external_ref_count_direct,
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
        let build_hash = loader_io.current_build_hash();

        let inner = LoaderInner {
            // start at 1 because 0 means null
            next_handle_index: 1,
            artifact_id_to_handle: Default::default(),
            load_handle_infos: Default::default(),
            loader_io,
            events_tx,
            events_rx,
            indirect_states: Default::default(),
            indirect_to_load: Default::default(),
            current_build_hash: build_hash,
            current_reload_action: None,
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
        self.inner.lock().unwrap().add_engine_ref_indirect(id)
    }

    pub(crate) fn add_engine_ref_by_handle(
        &self,
        load_handle: LoadHandle,
    ) -> LoadHandle {
        if load_handle.is_indirect() {
            self.inner
                .lock()
                .unwrap()
                .add_engine_ref_by_handle_indirect(load_handle)
        } else {
            self.inner
                .lock()
                .unwrap()
                .add_engine_ref_by_handle_direct(load_handle)
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
    artifact_id_to_handle: &'a HashMap<ArtifactIdAndHash, LoadHandle>,
    load_handle_infos: &'a HashMap<LoadHandle, LoadHandleInfo>,
    loader_io: &'a dyn LoaderIO,
}

impl<'a> LoaderInfoProvider for LoadHandleInfoProviderImpl<'a> {
    // Used when deserializing to convert an artifact id into the load handle of the already-loaded
    // artifact
    fn resolved_load_handle(
        &self,
        id: &ArtifactRef,
    ) -> Option<Arc<ResolvedLoadHandle>> {
        let artifact_id = ArtifactId::from_uuid(id.0.as_uuid());
        let build_hash = self.loader_io.manifest_entry(artifact_id).unwrap().combined_build_hash;

        let load_handle = self.artifact_id_to_handle.get(&ArtifactIdAndHash {
            id: artifact_id,
            hash: build_hash,
        }).map(|l| *l)?;
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

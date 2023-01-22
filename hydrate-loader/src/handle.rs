use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};
use hydrate_model::ObjectId;
use dashmap::DashMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct LoadHandle(u64);

#[derive(Default)]
struct LoadInfo {
    strong_ref_count: AtomicU32,
    weak_ref_count: AtomicU32,
}

struct Loader {
    next_handle_index: AtomicU64,
    object_to_handle: DashMap<ObjectId, LoadHandle>,
    handles: DashMap<LoadHandle, LoadInfo>,
}

impl Default for Loader {
    fn default() -> Self {
        Loader {
            next_handle_index: AtomicU64::new(1),
            object_to_handle: Default::default(),
            handles: Default::default()
        }
    }
}

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
//     latency is up to two seeks on critical path
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
// - LoadHandle -> LoadInfo
// - AssetId -> LoadHandle
// -



impl Loader {
    pub fn update(&self) {
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

    pub fn get_or_insert(&self, resource_key: ObjectId) -> LoadHandle {
        *self.object_to_handle.entry(resource_key).or_insert_with(|| {
            let load_handle_index = self.next_handle_index.fetch_add(1, Ordering::Relaxed);
            let load_handle = LoadHandle(load_handle_index);

            self.handles.insert(load_handle, LoadInfo {
                strong_ref_count: AtomicU32::default(),
                weak_ref_count: AtomicU32::default(),
            });

            load_handle
        })
    }

    pub fn add_ref(&self, object_id: ObjectId) -> LoadHandle {
        let load_handle = self.get_or_insert(object_id);

        let previous_ref_count = self.handles.get(&load_handle).as_ref().unwrap().strong_ref_count.fetch_add(1, Ordering::Release);
        // mark as needing processing?

        load_handle
    }

    pub fn remove_ref(&self, load_handle: LoadHandle) {
        let previous_ref_count = self.handles.get(&load_handle).as_ref().unwrap().strong_ref_count.fetch_sub(1, Ordering::Release);
        // mark as needing processing?
    }
}

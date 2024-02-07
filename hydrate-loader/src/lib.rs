pub mod asset_storage;
mod disk_io;
pub mod loader;
pub mod storage;

pub use crate::asset_storage::{AssetStorageSet, DynAssetLoader};
use crate::disk_io::DiskAssetIO;
use crate::loader::Loader;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::handle::RefOp;
use hydrate_base::{ArtifactId, StringHash};
use std::path::PathBuf;
use type_uuid::TypeUuid;

mod artifact_type_id;
pub use artifact_type_id::ArtifactTypeId;

use crate::storage::IndirectIdentifier;
pub use hydrate_base::handle::Handle;

// Asset states can be:
// Unloaded, not subscribed
// Unloaded, subscribed, not requested yet
// Unloaded, subscribed, request in flight
// Unloaded, subscribed, request ready to load
// Loaded, unsubscribed

// States:
// Loaded | Unloaded
// Subscribed | Not Subscribed
// No request in flight | Request in flight | Request ready to load
//
// Unloaded, Not Subscribed, No Request in Flight -> Do nothing
// Unloaded, Not Subscribed, Request in Flight -> We could cancel the request, otherwise wait until ready to load
// Unloaded, Not Subscribed, Request Ready to Load -> Drop the data
// Unloaded, Subscribed, No Request in Flight -> Kick off the request
// Unloaded, Subscribed, Request in Flight -> Wait until ready to load
// Unloaded, Subscribed, Request Ready to Load -> Load the data
// Loaded, Subscribed, No Request in Flight -> Do nothing
// Loaded, Subscribed, Request in Flight -> Invalid
// Loaded, Subscribed, Request Ready to Load -> Invalid
// Loaded, Not Subscribed, No Request in Flight -> Drop the data
// Loaded, Not Subscribed, Request in Flight ->  Invalid
// Loaded, Not Subscribed, Request Ready to Load -> Invalid
//
// Request can only be in flight if we are not loaded
//
// Unloaded, Unsubscribed, No Request in flight
// Unloaded, Subscribed, No Request
// - potentially bail back to unloaded/unsubscribed
// Unloaded, Subscribed, Request in Flight
// - potentially bail back to unloaded, but the request needs to be cancelled/completed
// Unloaded, Subscribed, Request Ready to Load
// - potentially bail back to unloaded
// Loaded, Subscribed, No Request in flight
// Loaded, Unsubscribed, No Request in flight
// Unloaded, Unsubscribed, No Request in flight
//
// how to handle updates?
// (disk) <-> (request queue) <-> (version handling state machine?) <-> (streaming priority manager) <-> (asset handle tracker)
//
// streaming...
// - list of things we want to load, with score of value in having loaded
// - list of thigns that are loaded, with score of value in having loaded
// - by default, load requests are mandatory (max score?)
// - requests can be both assets and asset sub-resources
//
// how to handle updates
// - we have some code that works ignoring the updates
// - then another thing that is lower priority that tracks the additional thing to be loaded
// - injects the differences to the main version handling state machine
// - how to handle handles being allocated while streaming in updates?
// - how to handle an update arriving faster than the original asset version?
// - treat different versions as different assets?

// Create an Asset handle
// Ref-count tracking causes us to call subscribe/unsubscribe for the asset
// When we subscribe, we look up all the things we need to load
// Keep track of what we have subscribed to
// Provide all the assets that the subscriptions imply we need
// The loaded data for the asset will be built up in the IO and then taken by the game loader
//

pub fn process_ref_ops(
    loader: &Loader,
    rx: &Receiver<RefOp>,
) {
    while let Ok(ref_op) = rx.try_recv() {
        match ref_op {
            RefOp::Decrease(handle) => loader.remove_engine_ref(handle),
            RefOp::Increase(handle) => {
                loader.add_engine_ref_by_handle(handle);
            }
            RefOp::IncreaseUuid(uuid) => {
                loader.add_engine_ref(ArtifactId::from_uuid(uuid.as_uuid()));
            }
        }
    }
}

pub struct AssetManager {
    //build_root_path: PathBuf,
    //asset_io: DiskAssetIO,
    //asset_storage: DummyAssetStorage,
    asset_storage: AssetStorageSet,
    loader: Loader,
    ref_op_tx: Sender<RefOp>,
    ref_op_rx: Receiver<RefOp>,
}

impl AssetManager {
    pub fn new(build_data_root_path: PathBuf) -> Result<Self, String> {
        let (ref_op_tx, ref_op_rx) = crossbeam_channel::unbounded();
        let (loader_events_tx, loader_events_rx) = crossbeam_channel::unbounded();

        let asset_io = DiskAssetIO::new(build_data_root_path, loader_events_tx.clone())?;
        let loader = Loader::new(Box::new(asset_io), loader_events_tx, loader_events_rx);
        let asset_storage = AssetStorageSet::new(ref_op_tx.clone(), loader.indirection_table());

        let mut loader = AssetManager {
            asset_storage,
            loader,
            ref_op_tx,
            ref_op_rx,
        };

        loader.update();

        Ok(loader)
    }

    pub fn loader(&self) -> &Loader {
        &self.loader
    }

    pub fn storage(&self) -> &AssetStorageSet {
        &self.asset_storage
    }

    pub fn add_storage<T>(&mut self)
    where
        T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send,
    {
        self.asset_storage.add_storage::<T>();
    }

    pub fn add_storage_with_loader<AssetDataT, AssetT, LoaderT>(
        &mut self,
        loader: Box<LoaderT>,
    ) where
        AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static,
        AssetT: TypeUuid + 'static + Send,
        LoaderT: DynAssetLoader<AssetT> + 'static,
    {
        self.asset_storage
            .add_storage_with_loader::<AssetDataT, AssetT, LoaderT>(loader);
    }

    pub fn load_asset<T: TypeUuid + 'static + Send>(
        &self,
        artifact_id: ArtifactId,
    ) -> Handle<T> {
        let data_type_uuid = self
            .storage()
            .asset_to_data_type_uuid::<T>()
            .expect("Called load_asset_path with unregistered asset type");
        let load_handle = self.loader.add_engine_ref_indirect(IndirectIdentifier::ArtifactId(artifact_id, data_type_uuid));
        Handle::<T>::new(self.ref_op_tx.clone(), load_handle)
    }

    pub fn load_asset_symbol_name<T: TypeUuid + 'static + Send>(
        &self,
        symbol_name: &'static str,
    ) -> Handle<T> {
        self.load_asset_symbol_string_hash(StringHash::from_static_str(symbol_name))
    }

    pub fn load_asset_symbol_string_hash<T: TypeUuid + 'static + Send>(
        &self,
        symbol: StringHash,
    ) -> Handle<T> {
        let data_type_uuid = self
            .storage()
            .asset_to_data_type_uuid::<T>()
            .expect("Called load_asset_path with unregistered asset type");

        let load_handle = self
            .loader
            .add_engine_ref_indirect(IndirectIdentifier::SymbolWithType(symbol, data_type_uuid));
        Handle::<T>::new(self.ref_op_tx.clone(), load_handle)
    }

    pub fn update(&mut self) {
        process_ref_ops(&self.loader, &self.ref_op_rx);
        self.loader.update(&mut self.asset_storage);
    }
}

pub mod artifact_storage;
mod disk_io;
pub mod loader;
pub mod storage;

pub use crate::artifact_storage::{ArtifactStorageSet, DynArtifactLoader};
use crate::disk_io::DiskArtifactIO;
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
        }
    }
}

pub struct ArtifactManager {
    artifact_storage: ArtifactStorageSet,
    loader: Loader,
    ref_op_tx: Sender<RefOp>,
    ref_op_rx: Receiver<RefOp>,
}

impl ArtifactManager {
    pub fn new(build_data_root_path: PathBuf) -> Result<Self, String> {
        let (ref_op_tx, ref_op_rx) = crossbeam_channel::unbounded();
        let (loader_events_tx, loader_events_rx) = crossbeam_channel::unbounded();

        let artifact_io = DiskArtifactIO::new(build_data_root_path, loader_events_tx.clone())?;
        let loader = Loader::new(Box::new(artifact_io), loader_events_tx, loader_events_rx);
        let artifact_storage = ArtifactStorageSet::new(ref_op_tx.clone());

        let mut loader = ArtifactManager {
            artifact_storage,
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

    pub fn storage(&self) -> &ArtifactStorageSet {
        &self.artifact_storage
    }

    pub fn add_storage<T>(&mut self)
    where
        T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send,
    {
        self.artifact_storage.add_storage::<T>();
    }

    pub fn add_storage_with_loader<ArtifactDataT, ArtifactT, LoaderT>(
        &mut self,
        loader: Box<LoaderT>,
    ) where
        ArtifactDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static,
        ArtifactT: TypeUuid + 'static + Send,
        LoaderT: DynArtifactLoader<ArtifactT> + 'static,
    {
        self.artifact_storage
            .add_storage_with_loader::<ArtifactDataT, ArtifactT, LoaderT>(loader);
    }

    pub fn load_artifact<T: TypeUuid + 'static + Send>(
        &self,
        artifact_id: ArtifactId,
    ) -> Handle<T> {
        let data_type_uuid = self
            .storage()
            .artifact_to_data_type_uuid::<T>()
            .expect("Called load_artifact with unregistered asset type");
        let load_handle = self
            .loader
            .add_engine_ref_indirect(IndirectIdentifier::ArtifactId(artifact_id, data_type_uuid));
        Handle::<T>::new(self.ref_op_tx.clone(), load_handle)
    }

    pub fn load_artifact_symbol_name<T: TypeUuid + 'static + Send>(
        &self,
        symbol_name: &'static str,
    ) -> Handle<T> {
        self.load_artifact_symbol_string_hash(StringHash::from_static_str(symbol_name))
    }

    pub fn load_artifact_symbol_string_hash<T: TypeUuid + 'static + Send>(
        &self,
        symbol: StringHash,
    ) -> Handle<T> {
        let data_type_uuid = self
            .storage()
            .artifact_to_data_type_uuid::<T>()
            .expect("Called load_artifact with unregistered asset type");

        let load_handle = self
            .loader
            .add_engine_ref_indirect(IndirectIdentifier::SymbolWithType(symbol, data_type_uuid));
        Handle::<T>::new(self.ref_op_tx.clone(), load_handle)
    }

    pub fn update(&mut self) {
        process_ref_ops(&self.loader, &self.ref_op_rx);
        self.loader.update(&mut self.artifact_storage);
    }
}

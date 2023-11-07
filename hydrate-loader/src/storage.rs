use std::{error::Error, sync::Arc};

use crate::loader::LoaderEvent;
use crossbeam_channel::Sender;
use dashmap::DashMap;
use hydrate_base::handle::LoaderInfoProvider;
use hydrate_base::{AssetTypeId, LoadHandle};

#[derive(Debug)]
pub enum HandleOp {
    Error(LoadHandle, u32, Box<dyn Error + Send>),
    Complete(LoadHandle, u32),
    Drop(LoadHandle, u32),
}

/// Type that allows the downstream asset storage implementation to signal that an asset update
/// operation has completed. See [`AssetStorage::update_asset`].
pub struct AssetLoadOp {
    //sender: Option<Sender<HandleOp>>,
    sender: Option<Sender<LoaderEvent>>,
    handle: LoadHandle,
    version: u32,
}

impl AssetLoadOp {
    pub(crate) fn new(
        sender: Sender<LoaderEvent>,
        handle: LoadHandle,
        version: u32,
    ) -> Self {
        Self {
            sender: Some(sender),
            handle,
            version,
        }
    }

    /// Returns the `LoadHandle` associated with the load operation
    pub fn load_handle(&self) -> LoadHandle {
        self.handle
    }

    /// Signals that this load operation has completed succesfully.
    pub fn complete(mut self) {
        log::debug!("LoadOp for {:?} complete", self.handle);
        let _ = self
            .sender
            .as_ref()
            .unwrap()
            .send(LoaderEvent::LoadResult(HandleOp::Complete(
                self.handle,
                self.version,
            )));
        self.sender = None;
    }

    /// Signals that this load operation has completed with an error.
    pub fn error<E: Error + 'static + Send>(
        mut self,
        error: E,
    ) {
        println!("LoadOp for {:?} error {:?}", self.handle, error);
        let _ = self
            .sender
            .as_ref()
            .unwrap()
            .send(LoaderEvent::LoadResult(HandleOp::Error(
                self.handle,
                self.version,
                Box::new(error),
            )));
        self.sender = None;
    }
}

impl Drop for AssetLoadOp {
    fn drop(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(LoaderEvent::LoadResult(HandleOp::Drop(
                self.handle,
                self.version,
            )));
        }
    }
}

/// Storage for all assets of all asset types.
///
/// Consumers are expected to provide the implementation for this, as this is the bridge between
/// [`Loader`](crate::loader::Loader) and the application.
pub trait AssetStorage {
    /// Updates the backing data of an asset.
    ///
    /// An example usage of this is when a texture such as "player.png" changes while the
    /// application is running. The asset ID is the same, but the underlying pixel data can differ.
    ///
    /// # Parameters
    ///
    /// * `loader`: Loader implementation calling this function.
    /// * `asset_type_id`: UUID of the asset type.
    /// * `data`: The updated asset byte data.
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular asset.
    /// * `load_op`: Allows the loading implementation to signal when loading is done / errors.
    /// * `version`: Runtime load version of this asset, increments each time the asset is updated.
    fn update_asset(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        asset_type_id: &AssetTypeId,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>>;

    /// Commits the specified asset version as loaded and ready to use.
    ///
    /// # Parameters
    ///
    /// * `asset_type_id`: UUID of the asset type.
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular asset.
    /// * `version`: Runtime load version of this asset, increments each time the asset is updated.
    fn commit_asset_version(
        &mut self,
        asset_type: &AssetTypeId,
        load_handle: LoadHandle,
        version: u32,
    );

    /// Frees the asset identified by the load handle.
    ///
    /// # Parameters
    ///
    /// * `asset_type_id`: UUID of the asset type.
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular asset.
    fn free(
        &mut self,
        asset_type_id: &AssetTypeId,
        load_handle: LoadHandle,
        version: u32,
    );
}

// Sort of replaced with LoadState

// /// Asset loading status.
// #[derive(Debug)]
// pub enum LoadStatus {
//     /// There is no request for the asset to be loaded.
//     NotRequested,
//     /// The asset is being loaded.
//     Loading,
//     /// The asset is loaded.
//     Loaded,
//     /// The asset is being unloaded.
//     Unloading,
//     /// The asset does not exist.
//     DoesNotExist,
//     /// There was an error during loading / unloading of the asset.
//     Error(Box<dyn Error>),
// }

// /// Information about an asset load operation.
// ///
// /// **Note:** The information is true at the time the `LoadInfo` is retrieved. The actual number of
// /// references may change.
// #[derive(Debug)]
// pub struct LoadInfo {
//     /// UUID of the asset.
//     pub asset_id: AssetUuid,
//     /// Number of references to the asset.
//     pub refs: u32,
//     /// Asset name. Not guaranteed to always be available.
//     pub asset_name: Option<String>,
// }

// LoaderInfoProvider - Moved to hydrate_base
// HandleAllocator - Removed

/// An indirect identifier that can be resolved to a specific [`AssetUuid`] by an [`IndirectionResolver`] impl.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum IndirectIdentifier {
    //PathWithTagAndType(String, String, AssetTypeId),
    PathWithType(String, AssetTypeId),
    //Path(String),
}
impl IndirectIdentifier {
    pub fn path(&self) -> &str {
        match self {
            //IndirectIdentifier::PathWithTagAndType(path, _, _) => path.as_str(),
            IndirectIdentifier::PathWithType(path, _) => path.as_str(),
            //IndirectIdentifier::Path(path) => path.as_str(),
        }
    }

    pub fn type_id(&self) -> Option<&AssetTypeId> {
        match self {
            //IndirectIdentifier::PathWithTagAndType(_, _, ty) => Some(ty),
            IndirectIdentifier::PathWithType(_, ty) => Some(ty),
            //IndirectIdentifier::Path(_) => None,
        }
    }
}

// Should not need a resolver, paths are unambiguous

// /// Resolves ambiguous [`IndirectIdentifier`]s to a single asset ID given a set of candidates.
// pub trait IndirectionResolver {
//     fn resolve(
//         &self,
//         id: &IndirectIdentifier,
//         candidates: Vec<(PathBuf, Vec<AssetMetadata>)>,
//     ) -> Option<AssetUuid>;
// }
//
// /// Default implementation of [`IndirectionResolver`] which resolves to the first asset in the list of candidates
// /// of the appropriate type.
// pub struct DefaultIndirectionResolver;
// impl IndirectionResolver for DefaultIndirectionResolver {
//     fn resolve(
//         &self,
//         id: &IndirectIdentifier,
//         candidates: Vec<(PathBuf, Vec<AssetMetadata>)>,
//     ) -> Option<AssetUuid> {
//         let id_type = id.type_id();
//         for candidate in candidates {
//             for asset in candidate.1 {
//                 if let Some(artifact) = asset.artifact {
//                     if id_type.is_none() || *id_type.unwrap() == artifact.type_id {
//                         return Some(asset.id);
//                     }
//                 }
//             }
//         }
//         None
//     }
// }

/// Resolves indirect [`LoadHandle`]s. See [`LoadHandle::is_indirect`] for details.
#[derive(Clone)]
pub struct IndirectionTable(pub(crate) Arc<DashMap<LoadHandle, LoadHandle>>);
impl IndirectionTable {
    pub fn resolve(
        &self,
        indirect_handle: LoadHandle,
    ) -> Option<LoadHandle> {
        self.0.get(&indirect_handle).map(|l| *l)
    }
}

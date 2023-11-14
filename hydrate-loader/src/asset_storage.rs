use hydrate_base::{ArtifactId, handle::{ArtifactHandle, RefOp, TypedArtifactStorage}, LoadHandle};
use std::{collections::HashMap, error::Error, sync::Mutex};

use crate::storage::{AssetLoadOp, AssetStorage, IndirectionTable};
use crossbeam_channel::{Receiver, Sender};
use downcast_rs::Downcast;
use hydrate_base::handle::LoaderInfoProvider;
use hydrate_base::handle::SerdeContext;
use std::marker::PhantomData;
use type_uuid::TypeUuid;
use crate::ArtifactTypeId;

// Used to dynamic dispatch into a storage, supports checked downcasting
pub trait DynAssetStorage: Downcast + Send {
    fn update_artifact(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>>;
    fn commit_artifact_version(
        &mut self,
        handle: LoadHandle,
        version: u32,
    );
    fn free(
        &mut self,
        handle: LoadHandle,
        version: u32,
    );

    fn type_name(&self) -> &'static str;
}

downcast_rs::impl_downcast!(DynAssetStorage);

pub struct AssetStorageSetInner {
    storage: HashMap<ArtifactTypeId, Box<dyn DynAssetStorage>>,
    data_to_asset_type_uuid: HashMap<ArtifactTypeId, ArtifactTypeId>,
    asset_to_data_type_uuid: HashMap<ArtifactTypeId, ArtifactTypeId>,
    refop_sender: Sender<RefOp>,
    indirection_table: IndirectionTable,
}

// Contains a storage per asset type
pub struct AssetStorageSet {
    inner: Mutex<AssetStorageSetInner>,
}

impl AssetStorageSet {
    pub fn new(
        refop_sender: Sender<RefOp>,
        indirection_table: IndirectionTable,
    ) -> Self {
        let inner = AssetStorageSetInner {
            storage: Default::default(),
            data_to_asset_type_uuid: Default::default(),
            asset_to_data_type_uuid: Default::default(),
            refop_sender,
            indirection_table,
        };

        Self {
            inner: Mutex::new(inner),
        }
    }

    pub fn add_storage<T>(&self)
    where
        T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send,
    {
        let mut inner = self.inner.lock().unwrap();
        let indirection_table = inner.indirection_table.clone();
        let refop_sender = inner.refop_sender.clone();
        let old = inner
            .data_to_asset_type_uuid
            .insert(ArtifactTypeId::from_bytes(T::UUID), ArtifactTypeId::from_bytes(T::UUID));
        assert!(old.is_none());
        let old = inner
            .asset_to_data_type_uuid
            .insert(ArtifactTypeId::from_bytes(T::UUID), ArtifactTypeId::from_bytes(T::UUID));
        assert!(old.is_none());
        inner.storage.insert(
            ArtifactTypeId::from_bytes(T::UUID),
            Box::new(Storage::<T>::new(
                refop_sender,
                Box::new(DefaultAssetLoader::default()),
                indirection_table,
            )),
        );
    }

    pub fn add_storage_with_loader<AssetDataT, AssetT, LoaderT>(
        &self,
        loader: Box<LoaderT>,
    ) where
        AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static,
        AssetT: TypeUuid + 'static + Send,
        LoaderT: DynAssetLoader<AssetT> + 'static,
    {
        let mut inner = self.inner.lock().unwrap();
        let indirection_table = inner.indirection_table.clone();
        let refop_sender = inner.refop_sender.clone();
        let old = inner
            .data_to_asset_type_uuid
            .insert(ArtifactTypeId::from_bytes(AssetDataT::UUID), ArtifactTypeId::from_bytes(AssetT::UUID));
        assert!(old.is_none());
        let old = inner
            .asset_to_data_type_uuid
            .insert(ArtifactTypeId::from_bytes(AssetT::UUID), ArtifactTypeId::from_bytes(AssetDataT::UUID));
        assert!(old.is_none());
        inner.storage.insert(
            ArtifactTypeId::from_bytes(AssetT::UUID),
            Box::new(Storage::<AssetT>::new(
                refop_sender,
                loader,
                indirection_table,
            )),
        );
    }

    pub fn asset_to_data_type_uuid<AssetT>(&self) -> Option<ArtifactTypeId>
    where
        AssetT: TypeUuid + 'static + Send,
    {
        let inner = self.inner.lock().unwrap();
        inner
            .asset_to_data_type_uuid
            .get(&ArtifactTypeId::from_bytes(AssetT::UUID))
            .cloned()
    }
}

// Implement distill's AssetStorage - an untyped trait that finds the asset_type's storage and
// forwards the call
impl AssetStorage for AssetStorageSet {
    fn update_asset(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        asset_data_type_id: &ArtifactTypeId,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>> {
        let mut inner = self.inner.lock().unwrap();

        let asset_type_id = *inner
            .data_to_asset_type_uuid
            .get(asset_data_type_id)
            .expect("unknown asset data type");

        let x = inner
            .storage
            .get_mut(&asset_type_id)
            .expect("unknown asset type")
            .update_artifact(loader_info, &data, load_handle, load_op, version);
        x
    }

    fn commit_asset_version(
        &mut self,
        asset_data_type_id: &ArtifactTypeId,
        load_handle: LoadHandle,
        version: u32,
    ) {
        let mut inner = self.inner.lock().unwrap();

        let asset_type_id = *inner
            .data_to_asset_type_uuid
            .get(asset_data_type_id)
            .expect("unknown asset data type");

        inner
            .storage
            .get_mut(&asset_type_id)
            .expect("unknown asset type")
            .commit_artifact_version(load_handle, version)
    }

    fn free(
        &mut self,
        asset_data_type_id: &ArtifactTypeId,
        load_handle: LoadHandle,
        version: u32,
    ) {
        let mut inner = self.inner.lock().unwrap();

        let asset_type_id = *inner
            .data_to_asset_type_uuid
            .get(asset_data_type_id)
            .expect("unknown asset data type");

        inner
            .storage
            .get_mut(&asset_type_id)
            .expect("unknown asset type")
            .free(load_handle, version)
    }
}

// Implement distill's TypedAssetStorage - a typed trait that finds the asset_type's storage and
// forwards the call
impl<A: TypeUuid + 'static + Send> TypedArtifactStorage<A> for AssetStorageSet {
    fn get<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<&A> {
        // This transmute can probably be unsound, but I don't have the energy to fix it right now
        unsafe {
            std::mem::transmute(
                self.inner
                    .lock()
                    .unwrap()
                    .storage
                    .get(&ArtifactTypeId::from_bytes(A::UUID))
                    .expect("unknown asset type")
                    .as_ref()
                    .downcast_ref::<Storage<A>>()
                    .expect("failed to downcast")
                    .get(handle),
            )
        }
    }
    fn get_version<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<u32> {
        self.inner
            .lock()
            .unwrap()
            .storage
            .get(&ArtifactTypeId::from_bytes(A::UUID))
            .expect("unknown asset type")
            .as_ref()
            .downcast_ref::<Storage<A>>()
            .expect("failed to downcast")
            .get_version(handle)
    }
    fn get_artifact_with_version<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<(&A, u32)> {
        // This transmute can probably be unsound, but I don't have the energy to fix it right now
        unsafe {
            std::mem::transmute(
                self.inner
                    .lock()
                    .unwrap()
                    .storage
                    .get(&ArtifactTypeId::from_bytes(A::UUID))
                    .expect("unknown asset type")
                    .as_ref()
                    .downcast_ref::<Storage<A>>()
                    .expect("failed to downcast")
                    .get_asset_with_version(handle),
            )
        }
    }
}

// Loaders can return immediately by value, or later by returning a channel
pub enum UpdateAssetResult<AssetT>
where
    AssetT: Send,
{
    Result(AssetT),
    AsyncResult(Receiver<AssetT>),
}

// Implements loading logic (i.e. turning bytes into an asset. The asset may contain runtime-only
// data and may be created asynchronously
pub trait DynAssetLoader<AssetT>: Send
where
    AssetT: TypeUuid + 'static + Send,
{
    fn update_asset(
        &mut self,
        refop_sender: &Sender<RefOp>,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<UpdateAssetResult<AssetT>, Box<dyn Error + Send + 'static>>;

    fn commit_asset_version(
        &mut self,
        handle: LoadHandle,
        version: u32,
    );

    fn free(
        &mut self,
        handle: LoadHandle,
    );
}

// A simple loader that just deserializes data
struct DefaultAssetLoader<AssetDataT>
where
    AssetDataT: TypeUuid + Send + for<'a> serde::Deserialize<'a> + 'static,
{
    phantom_data: PhantomData<AssetDataT>,
}

impl<AssetDataT> Default for DefaultAssetLoader<AssetDataT>
where
    AssetDataT: TypeUuid + Send + for<'a> serde::Deserialize<'a> + 'static,
{
    fn default() -> Self {
        DefaultAssetLoader {
            phantom_data: Default::default(),
        }
    }
}

impl<AssetDataT> DynAssetLoader<AssetDataT> for DefaultAssetLoader<AssetDataT>
where
    AssetDataT: TypeUuid + Send + for<'a> serde::Deserialize<'a> + 'static,
{
    fn update_asset(
        &mut self,
        refop_sender: &Sender<RefOp>,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        _load_handle: LoadHandle,
        load_op: AssetLoadOp,
        _version: u32,
    ) -> Result<UpdateAssetResult<AssetDataT>, Box<dyn Error + Send + 'static>> {
        log::debug!("DefaultAssetLoader update_asset");

        let asset = SerdeContext::with(loader_info, refop_sender.clone(), || {
            log::debug!("bincode deserialize");
            let x = bincode::deserialize::<AssetDataT>(data)
                // Coerce into boxed error
                .map_err(|x| -> Box<dyn Error + Send + 'static> { Box::new(x) });
            println!("finished deserialize");
            x
        })?;
        log::debug!("call load_op.complete()");

        load_op.complete();
        log::debug!("return");
        Ok(UpdateAssetResult::Result(asset))
    }

    fn commit_asset_version(
        &mut self,
        _handle: LoadHandle,
        _version: u32,
    ) {
    }

    fn free(
        &mut self,
        _handle: LoadHandle,
    ) {
    }
}

struct UncommittedArtifactState<A: Send> {
    version: u32,
    artifact_id: ArtifactId,
    result: UpdateAssetResult<A>,
}

struct ArtifactState<A> {
    version: u32,
    artifact_id: ArtifactId,
    asset: A,
}

// A strongly typed storage for a single asset type
pub struct Storage<AssetT: TypeUuid + Send> {
    refop_sender: Sender<RefOp>,
    artifacts: HashMap<LoadHandle, ArtifactState<AssetT>>,
    uncommitted: HashMap<LoadHandle, UncommittedArtifactState<AssetT>>,
    loader: Box<dyn DynAssetLoader<AssetT>>,
    indirection_table: IndirectionTable,
}

impl<AssetT: TypeUuid + Send> Storage<AssetT> {
    fn new(
        sender: Sender<RefOp>,
        loader: Box<dyn DynAssetLoader<AssetT>>,
        indirection_table: IndirectionTable,
    ) -> Self {
        Self {
            refop_sender: sender,
            artifacts: HashMap::new(),
            uncommitted: HashMap::new(),
            loader,
            indirection_table,
        }
    }
    fn get<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<&AssetT> {
        let handle = if handle.load_handle().is_indirect() {
            self.indirection_table.resolve(handle.load_handle())?
        } else {
            handle.load_handle()
        };
        self.artifacts.get(&handle).map(|a| &a.asset)
    }
    fn get_version<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<u32> {
        let handle = if handle.load_handle().is_indirect() {
            self.indirection_table.resolve(handle.load_handle())?
        } else {
            handle.load_handle()
        };
        self.artifacts.get(&handle).map(|a| a.version)
    }
    fn get_asset_with_version<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<(&AssetT, u32)> {
        let handle = if handle.load_handle().is_indirect() {
            self.indirection_table.resolve(handle.load_handle())?
        } else {
            handle.load_handle()
        };
        self.artifacts.get(&handle).map(|a| (&a.asset, a.version))
    }
}

impl<AssetT: TypeUuid + 'static + Send> DynAssetStorage for Storage<AssetT> {
    fn update_artifact(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>> {
        log::debug!(
            "update_asset {} {:?} {:?} {}",
            core::any::type_name::<AssetT>(),
            load_handle,
            loader_info.artifact_id(load_handle).unwrap(),
            version
        );

        let result = self.loader.update_asset(
            &self.refop_sender,
            loader_info,
            data,
            load_handle,
            load_op,
            version,
        )?;
        let asset_uuid = loader_info.artifact_id(load_handle).unwrap();

        // Add to list of uncommitted assets
        self.uncommitted.insert(
            load_handle,
            UncommittedArtifactState {
                artifact_id: asset_uuid,
                result,
                version,
            },
        );

        Ok(())
    }

    fn commit_artifact_version(
        &mut self,
        load_handle: LoadHandle,
        version: u32,
    ) {
        // Remove from the uncommitted list
        let uncommitted_asset_state = self
            .uncommitted
            .remove(&load_handle)
            .expect("asset not present when committing");

        log::debug!(
            "commit_asset_version {} {:?} {:?} {}",
            core::any::type_name::<AssetT>(),
            load_handle,
            uncommitted_asset_state.artifact_id,
            version
        );

        let asset_uuid = uncommitted_asset_state.artifact_id;
        let version = uncommitted_asset_state.version;
        let asset = match uncommitted_asset_state.result {
            UpdateAssetResult::Result(asset) => asset,
            UpdateAssetResult::AsyncResult(rx) => rx
                .try_recv()
                .expect("LoadOp committed but result not sent via channel"),
        };

        // If a load handler exists, trigger the commit_asset_version callback
        self.loader.commit_asset_version(load_handle, version);

        let asset_state = ArtifactState {
            asset,
            artifact_id: asset_uuid,
            version,
        };

        // Commit the result
        self.artifacts.insert(load_handle, asset_state);
    }

    fn free(
        &mut self,
        load_handle: LoadHandle,
        version: u32,
    ) {
        if let Some(asset_state) = self.artifacts.get(&load_handle) {
            if asset_state.version == version {
                // Remove it from the list of assets
                let asset_state = self.artifacts.remove(&load_handle).unwrap();

                log::debug!(
                    "free {} {:?} {:?}",
                    core::any::type_name::<AssetT>(),
                    load_handle,
                    asset_state.artifact_id
                );
                // Trigger the free callback on the load handler, if one exists
                self.loader.free(load_handle);
            }
        }
    }

    fn type_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }
}

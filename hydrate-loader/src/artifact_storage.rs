use hydrate_base::{
    handle::{ArtifactHandle, RefOp, TypedArtifactStorage},
    ArtifactId, LoadHandle,
};
use std::{collections::HashMap, error::Error, sync::Mutex};

use crate::storage::{ArtifactLoadOp, ArtifactStorage};
use crate::ArtifactTypeId;
use crossbeam_channel::{Receiver, Sender};
use downcast_rs::Downcast;
use hydrate_base::handle::LoaderInfoProvider;
use hydrate_base::handle::SerdeContext;
use std::marker::PhantomData;
use type_uuid::TypeUuid;

// Used to dynamic dispatch into a storage, supports checked downcasting
pub trait DynArtifactStorage: Downcast + Send {
    fn load_artifact(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        artifact_id: ArtifactId,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: ArtifactLoadOp,
    ) -> Result<(), Box<dyn Error + Send + 'static>>;
    fn commit_artifact(
        &mut self,
        handle: LoadHandle,
    );
    fn free_artifact(
        &mut self,
        handle: LoadHandle,
    );

    fn type_name(&self) -> &'static str;
}

downcast_rs::impl_downcast!(DynArtifactStorage);

pub struct ArtifactStorageSetInner {
    storage: HashMap<ArtifactTypeId, Box<dyn DynArtifactStorage>>,
    data_to_artifact_type_uuid: HashMap<ArtifactTypeId, ArtifactTypeId>,
    artifact_to_data_type_uuid: HashMap<ArtifactTypeId, ArtifactTypeId>,
    refop_sender: Sender<RefOp>,
}

// Contains a storage per artifact type
pub struct ArtifactStorageSet {
    inner: Mutex<ArtifactStorageSetInner>,
}

impl ArtifactStorageSet {
    pub fn new(refop_sender: Sender<RefOp>) -> Self {
        let inner = ArtifactStorageSetInner {
            storage: Default::default(),
            data_to_artifact_type_uuid: Default::default(),
            artifact_to_data_type_uuid: Default::default(),
            refop_sender,
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
        let refop_sender = inner.refop_sender.clone();
        let old = inner.data_to_artifact_type_uuid.insert(
            ArtifactTypeId::from_bytes(T::UUID),
            ArtifactTypeId::from_bytes(T::UUID),
        );
        assert!(old.is_none());
        let old = inner.artifact_to_data_type_uuid.insert(
            ArtifactTypeId::from_bytes(T::UUID),
            ArtifactTypeId::from_bytes(T::UUID),
        );
        assert!(old.is_none());
        inner.storage.insert(
            ArtifactTypeId::from_bytes(T::UUID),
            Box::new(Storage::<T>::new(
                refop_sender,
                Box::new(DefaultArtifactLoader::default()),
            )),
        );
    }

    pub fn add_storage_with_loader<ArtifactDataT, ArtifactT, LoaderT>(
        &self,
        loader: Box<LoaderT>,
    ) where
        ArtifactDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static,
        ArtifactT: TypeUuid + 'static + Send,
        LoaderT: DynArtifactLoader<ArtifactT> + 'static,
    {
        let mut inner = self.inner.lock().unwrap();
        let refop_sender = inner.refop_sender.clone();
        let old = inner.data_to_artifact_type_uuid.insert(
            ArtifactTypeId::from_bytes(ArtifactDataT::UUID),
            ArtifactTypeId::from_bytes(ArtifactT::UUID),
        );
        assert!(old.is_none());
        let old = inner.artifact_to_data_type_uuid.insert(
            ArtifactTypeId::from_bytes(ArtifactT::UUID),
            ArtifactTypeId::from_bytes(ArtifactDataT::UUID),
        );
        assert!(old.is_none());
        inner.storage.insert(
            ArtifactTypeId::from_bytes(ArtifactT::UUID),
            Box::new(Storage::<ArtifactT>::new(refop_sender, loader)),
        );
    }

    pub fn artifact_to_data_type_uuid<ArtifactT>(&self) -> Option<ArtifactTypeId>
    where
        ArtifactT: TypeUuid + 'static + Send,
    {
        let inner = self.inner.lock().unwrap();
        inner
            .artifact_to_data_type_uuid
            .get(&ArtifactTypeId::from_bytes(ArtifactT::UUID))
            .cloned()
    }
}

// Implement distill's ArtifactStorage - an untyped trait that finds the artifact_type's storage and
// forwards the call
impl ArtifactStorage for ArtifactStorageSet {
    fn load_artifact(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        artifact_type_id: &ArtifactTypeId,
        artifact_id: ArtifactId,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: ArtifactLoadOp,
    ) -> Result<(), Box<dyn Error + Send + 'static>> {
        let mut inner = self.inner.lock().unwrap();

        let artifact_type_id = *inner
            .data_to_artifact_type_uuid
            .get(artifact_type_id)
            .expect("unknown artifact data type");

        let x = inner
            .storage
            .get_mut(&artifact_type_id)
            .expect("unknown artifact type")
            .load_artifact(loader_info, artifact_id, &data, load_handle, load_op);
        x
    }

    fn commit_artifact(
        &mut self,
        artifact_data_type_id: ArtifactTypeId,
        load_handle: LoadHandle,
    ) {
        let mut inner = self.inner.lock().unwrap();

        let artifact_type_id = *inner
            .data_to_artifact_type_uuid
            .get(&artifact_data_type_id)
            .expect("unknown artifact data type");

        inner
            .storage
            .get_mut(&artifact_type_id)
            .expect("unknown artifact type")
            .commit_artifact(load_handle)
    }

    fn free_artifact(
        &mut self,
        artifact_data_type_id: ArtifactTypeId,
        load_handle: LoadHandle,
    ) {
        let mut inner = self.inner.lock().unwrap();

        let artifact_type_id = *inner
            .data_to_artifact_type_uuid
            .get(&artifact_data_type_id)
            .expect("unknown artifact data type");

        inner
            .storage
            .get_mut(&artifact_type_id)
            .expect("unknown artifact type")
            .free_artifact(load_handle)
    }
}

// Implement distill's TypedArtifactStorage - a typed trait that finds the artifact_type's storage and
// forwards the call
impl<A: TypeUuid + 'static + Send> TypedArtifactStorage<A> for ArtifactStorageSet {
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
                    .expect("unknown artifact type")
                    .as_ref()
                    .downcast_ref::<Storage<A>>()
                    .expect("failed to downcast")
                    .get(handle),
            )
        }
    }
}

// Loaders can return immediately by value, or later by returning a channel
pub enum UpdateArtifactResult<ArtifactT>
where
    ArtifactT: Send,
{
    Result(ArtifactT),
    AsyncResult(Receiver<ArtifactT>),
}

// Implements loading logic (i.e. turning bytes into an artifact. The artifact may contain runtime-only
// data and may be created asynchronously
pub trait DynArtifactLoader<ArtifactT>: Send
where
    ArtifactT: TypeUuid + 'static + Send,
{
    fn load_artifact(
        &mut self,
        refop_sender: &Sender<RefOp>,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: ArtifactLoadOp,
    ) -> Result<UpdateArtifactResult<ArtifactT>, Box<dyn Error + Send + 'static>>;

    fn commit_artifact(
        &mut self,
        handle: LoadHandle,
    );

    fn free_artifact(
        &mut self,
        handle: LoadHandle,
    );
}

// A simple loader that just deserializes data
struct DefaultArtifactLoader<ArtifactDataT>
where
    ArtifactDataT: TypeUuid + Send + for<'a> serde::Deserialize<'a> + 'static,
{
    phantom_data: PhantomData<ArtifactDataT>,
}

impl<ArtifactDataT> Default for DefaultArtifactLoader<ArtifactDataT>
where
    ArtifactDataT: TypeUuid + Send + for<'a> serde::Deserialize<'a> + 'static,
{
    fn default() -> Self {
        DefaultArtifactLoader {
            phantom_data: Default::default(),
        }
    }
}

impl<ArtifactDataT> DynArtifactLoader<ArtifactDataT> for DefaultArtifactLoader<ArtifactDataT>
where
    ArtifactDataT: TypeUuid + Send + for<'a> serde::Deserialize<'a> + 'static,
{
    fn load_artifact(
        &mut self,
        refop_sender: &Sender<RefOp>,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        _load_handle: LoadHandle,
        load_op: ArtifactLoadOp,
    ) -> Result<UpdateArtifactResult<ArtifactDataT>, Box<dyn Error + Send + 'static>> {
        log::debug!("DefaultArtifactLoader load_artifact");

        let artifact_data = SerdeContext::with(loader_info, refop_sender.clone(), || {
            log::debug!("bincode deserialize");
            let x = bincode::deserialize::<ArtifactDataT>(data)
                // Coerce into boxed error
                .map_err(|x| -> Box<dyn Error + Send + 'static> { Box::new(x) });
            println!("finished deserialize");
            x
        })?;
        log::debug!("call load_op.complete()");

        load_op.complete();
        log::debug!("return");
        Ok(UpdateArtifactResult::Result(artifact_data))
    }

    fn commit_artifact(
        &mut self,
        _handle: LoadHandle,
    ) {
    }

    fn free_artifact(
        &mut self,
        _handle: LoadHandle,
    ) {
    }
}

struct UncommittedArtifactState<A: Send> {
    artifact_id: ArtifactId,
    result: UpdateArtifactResult<A>,
}

struct ArtifactState<A> {
    artifact_id: ArtifactId,
    artifact: A,
}

// A strongly typed storage for a single artifact type
pub struct Storage<ArtifactT: TypeUuid + Send> {
    refop_sender: Sender<RefOp>,
    artifacts: HashMap<LoadHandle, ArtifactState<ArtifactT>>,
    uncommitted: HashMap<LoadHandle, UncommittedArtifactState<ArtifactT>>,
    loader: Box<dyn DynArtifactLoader<ArtifactT>>,
}

impl<ArtifactT: TypeUuid + Send> Storage<ArtifactT> {
    fn new(
        sender: Sender<RefOp>,
        loader: Box<dyn DynArtifactLoader<ArtifactT>>,
    ) -> Self {
        Self {
            refop_sender: sender,
            artifacts: HashMap::new(),
            uncommitted: HashMap::new(),
            loader,
        }
    }
    fn get<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<&ArtifactT> {
        let handle = handle.direct_load_handle();
        self.artifacts.get(&handle).map(|a| &a.artifact)
    }
}

impl<ArtifactT: TypeUuid + 'static + Send> DynArtifactStorage for Storage<ArtifactT> {
    fn load_artifact(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        artifact_id: ArtifactId,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: ArtifactLoadOp,
    ) -> Result<(), Box<dyn Error + Send + 'static>> {
        log::debug!(
            "load_artifact {} {:?} {:?}",
            core::any::type_name::<ArtifactT>(),
            load_handle,
            artifact_id,
        );

        let result = self.loader.load_artifact(
            &self.refop_sender,
            loader_info,
            data,
            load_handle,
            load_op,
        )?;

        // Add to list of uncommitted artifacts
        self.uncommitted.insert(
            load_handle,
            UncommittedArtifactState {
                artifact_id,
                result,
            },
        );

        Ok(())
    }

    fn commit_artifact(
        &mut self,
        load_handle: LoadHandle,
    ) {
        // Remove from the uncommitted list
        let uncommitted_artifact_state = self
            .uncommitted
            .remove(&load_handle)
            .expect("artifact not present when committing");

        log::debug!(
            "commit_artifact {} {:?} {:?}",
            core::any::type_name::<ArtifactT>(),
            load_handle,
            uncommitted_artifact_state.artifact_id,
        );

        let artifact_id = uncommitted_artifact_state.artifact_id;
        let artifact = match uncommitted_artifact_state.result {
            UpdateArtifactResult::Result(artifact) => artifact,
            UpdateArtifactResult::AsyncResult(rx) => rx
                .try_recv()
                .expect("LoadOp committed but result not sent via channel"),
        };

        // If a load handler exists, trigger the commit_artifact callback
        self.loader.commit_artifact(load_handle);

        let artifact_state = ArtifactState {
            artifact,
            artifact_id,
        };

        // Commit the result
        self.artifacts.insert(load_handle, artifact_state);
    }

    fn free_artifact(
        &mut self,
        load_handle: LoadHandle,
    ) {
        if let Some(artifact_state) = self.artifacts.remove(&load_handle) {
            log::debug!(
                "free {} {:?} {:?}",
                core::any::type_name::<ArtifactT>(),
                load_handle,
                artifact_state.artifact_id
            );
            // Trigger the free callback on the load handler, if one exists
            self.loader.free_artifact(load_handle);
        }
    }

    fn type_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }
}

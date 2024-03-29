use std::error::Error;

use crate::loader::LoaderEvent;
use crate::ArtifactTypeId;
use crossbeam_channel::Sender;
use hydrate_base::handle::LoaderInfoProvider;
use hydrate_base::{ArtifactId, LoadHandle, StringHash};

#[derive(Debug)]
pub enum HandleOp {
    Error(LoadHandle, Box<dyn Error + Send>),
    Complete(LoadHandle),
    Drop(LoadHandle),
}

/// Type that allows the downstream artifact storage implementation to signal that an artifact update
/// operation has completed. See [`ArtifactStorage::load_artifact`].
pub struct ArtifactLoadOp {
    sender: Option<Sender<LoaderEvent>>,
    handle: LoadHandle,
}

impl ArtifactLoadOp {
    pub(crate) fn new(
        sender: Sender<LoaderEvent>,
        handle: LoadHandle,
    ) -> Self {
        Self {
            sender: Some(sender),
            handle,
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
            .send(LoaderEvent::LoadResult(HandleOp::Complete(self.handle)));
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
                Box::new(error),
            )));
        self.sender = None;
    }
}

impl Drop for ArtifactLoadOp {
    fn drop(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(LoaderEvent::LoadResult(HandleOp::Drop(self.handle)));
        }
    }
}

/// Storage for all artifacts of all artifact types.
///
/// Consumers are expected to provide the implementation for this, as this is the bridge between
/// [`Loader`](crate::loader::Loader) and the application.
pub trait ArtifactStorage {
    /// Updates the backing data of an artifact.
    ///
    /// An example usage of this is when a texture such as "player.png" changes while the
    /// application is running. The artifact ID is the same, but the underlying pixel data can differ.
    ///
    /// # Parameters
    ///
    /// * `loader`: Loader implementation calling this function.
    /// * `artifact_type_id`: UUID of the artifact type.
    /// * `data`: The updated artifact byte data.
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular artifact.
    /// * `load_op`: Allows the loading implementation to signal when loading is done / errors.
    fn load_artifact(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        artifact_type_id: &ArtifactTypeId,
        artifact_id: ArtifactId,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: ArtifactLoadOp,
    ) -> Result<(), Box<dyn Error + Send + 'static>>;

    /// Commits the specified artifact as loaded and ready to use.
    ///
    /// # Parameters
    ///
    /// * `artifact_type`: UUID of the artifact type.
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular artifact.
    fn commit_artifact(
        &mut self,
        artifact_type: ArtifactTypeId,
        load_handle: LoadHandle,
    );

    /// Frees the artifact identified by the load handle.
    ///
    /// # Parameters
    ///
    /// * `artifact_type_id`: UUID of the artifact type.
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular artifact.
    fn free_artifact(
        &mut self,
        artifact_type_id: ArtifactTypeId,
        load_handle: LoadHandle,
    );
}

/// An indirect identifier that can be resolved to a specific [`ArtifactID`] by an [`IndirectionResolver`] impl.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum IndirectIdentifier {
    ArtifactId(ArtifactId, ArtifactTypeId),
    SymbolWithType(StringHash, ArtifactTypeId),
}

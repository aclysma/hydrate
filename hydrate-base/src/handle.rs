use std::cell::RefCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};
use std::hash::Hasher;

use crate::ArtifactId;
use crossbeam_channel::Sender;
use serde::{de, ser, Deserialize, Serialize};
use uuid::Uuid;

/// Loading ID allocated by [`Loader`](crate::loader::Loader) to track loading of a particular artifact
/// or an indirect reference to an artifact.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct LoadHandle(pub u64);

impl LoadHandle {
    pub fn new(
        load_handle: u64,
        is_indirect: bool,
    ) -> Self {
        if is_indirect {
            Self(load_handle | (1 << 63))
        } else {
            Self(load_handle)
        }
    }

    /// Returns true if the handle needs to be resolved through the [`IndirectionTable`] before use.
    /// An "indirect" LoadHandle represents a load operation for an identifier that is late-bound,
    /// meaning the identifier may change which [`ArtifactId`] it resolves to.
    /// An example of an indirect LoadHandle would be one that loads by filesystem path.
    /// The specific artifact at a path may change as files change, move or are deleted, while a direct
    /// LoadHandle (one that addresses by ArtifactId) is guaranteed to refer to an ArtifactId for its
    /// whole lifetime.
    pub fn is_indirect(&self) -> bool {
        (self.0 & (1 << 63)) == 1 << 63
    }
}

/// A potentially unresolved reference to an asset
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ArtifactRef(pub ArtifactId);

/// Provides information about mappings between `ArtifactId` and `LoadHandle`.
/// Intended to be used for `Handle` serde.
pub trait LoaderInfoProvider: Send + Sync {
    /// Returns the load handle for the artifact with the given UUID, if present.
    ///
    /// This will only return `Some(..)` if there has been a previous call to [`crate::loader::Loader::add_ref`].
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the artifact.
    fn load_handle(
        &self,
        artifact_ref: &ArtifactRef,
    ) -> Option<Arc<ResolvedLoadHandle>>;

    /// Returns the ArtifactId for the given LoadHandle, if present.
    ///
    /// # Parameters
    ///
    /// * `load_handle`: ID allocated by [`Loader`](crate::loader::Loader) to track loading of the artifact.
    fn artifact_id(
        &self,
        load: LoadHandle,
    ) -> Option<ArtifactId>;
}

#[derive(Debug)]
pub struct ResolvedLoadHandle {
    pub id: LoadHandle,
    pub direct_load_handle: AtomicU64,
}

impl PartialEq for ResolvedLoadHandle {
    fn eq(&self, other: &Self) -> bool {
        // Only need to check the indirect ID
        self.id == other.id
    }
}

impl Eq for ResolvedLoadHandle {

}

impl Hash for ResolvedLoadHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Only care about hashing the id
        self.id.hash(state)
    }
}

impl ResolvedLoadHandle {
    pub fn new(id: LoadHandle, resolved_load_handle: LoadHandle) -> Arc<Self> {
        Arc::new(ResolvedLoadHandle {
            id,
            direct_load_handle: AtomicU64::new(resolved_load_handle.0)
        })
    }

    pub fn new_null_handle() -> Arc<Self> {
        Arc::new(ResolvedLoadHandle {
            id: LoadHandle(0),
            direct_load_handle: AtomicU64::default()
        })
    }

    pub fn direct_load_handle(&self) -> LoadHandle {
        LoadHandle(self.direct_load_handle.load(Ordering::Relaxed))
    }
}

/// Operations on an artifact reference.
#[derive(Debug)]
pub enum RefOp {
    Decrease(LoadHandle),
    Increase(LoadHandle),
}

/// Keeps track of whether a handle ref is a strong, weak or "internal" ref
#[derive(Debug)]
pub enum HandleRefType {
    /// Strong references decrement the count on drop
    Strong(Sender<RefOp>),
    /// Weak references do nothing on drop.
    Weak(Sender<RefOp>),
    /// Internal references do nothing on drop, but turn into Strong references on clone.
    /// Should only be used for references stored in loaded artifacts to avoid self-referencing
    Internal(Sender<RefOp>),
    /// Implementation detail, used when changing state in this enum
    None,
}

struct HandleRef {
    resolved_load_handle: Arc<ResolvedLoadHandle>,
    ref_type: HandleRefType,
}
impl PartialEq for HandleRef {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.resolved_load_handle.id.eq(&other.resolved_load_handle.id)
    }
}
impl Hash for HandleRef {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.resolved_load_handle.id.hash(state)
    }
}
impl Eq for HandleRef {}
impl Debug for HandleRef {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.resolved_load_handle.id.fmt(f)
    }
}

impl Drop for HandleRef {
    fn drop(&mut self) {
        use HandleRefType::*;
        self.ref_type = match std::mem::replace(&mut self.ref_type, None) {
            Strong(sender) => {
                let _ = sender.send(RefOp::Decrease(self.resolved_load_handle.id));
                Weak(sender)
            }
            r => r,
        };
    }
}

impl Clone for HandleRef {
    fn clone(&self) -> Self {
        use HandleRefType::*;
        Self {
            resolved_load_handle: self.resolved_load_handle.clone(),
            ref_type: match &self.ref_type {
                Internal(sender) | Strong(sender) => {
                    let _ = sender.send(RefOp::Increase(self.resolved_load_handle.id));
                    Strong(sender.clone())
                }
                Weak(sender) => Weak(sender.clone()),
                None => panic!("unexpected ref type in clone()"),
            },
        }
    }
}

impl ArtifactHandle for HandleRef {
    fn resolved_load_handle(&self) -> &Arc<ResolvedLoadHandle> {
        &self.resolved_load_handle
    }
}

/// Handle to an artifact.
#[derive(Eq)]
pub struct Handle<T: ?Sized> {
    handle_ref: HandleRef,
    marker: PhantomData<T>,
}

impl<T: ?Sized> PartialEq for Handle<T> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.handle_ref == other.handle_ref
    }
}

impl<T: ?Sized> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            handle_ref: self.handle_ref.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.handle_ref.hash(state);
    }
}

impl<T: ?Sized> Debug for Handle<T> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("handle_ref", &self.handle_ref)
            .finish()
    }
}

impl<T: ?Sized> From<GenericHandle> for Handle<T> {
    fn from(handle: GenericHandle) -> Self {
        Self {
            handle_ref: handle.handle_ref,
            marker: PhantomData,
        }
    }
}

impl<T> Handle<T> {
    /// Creates a new handle with `HandleRefType::Strong`
    pub fn new(
        chan: Sender<RefOp>,
        resolved_load_handle: Arc<ResolvedLoadHandle>,
    ) -> Self {
        Self {
            handle_ref: HandleRef {
                resolved_load_handle,
                ref_type: HandleRefType::Strong(chan),
            },
            marker: PhantomData,
        }
    }

    /// Creates a new handle with `HandleRefType::Internal`
    pub(crate) fn new_internal(
        chan: Sender<RefOp>,
        resolved_load_handle: Arc<ResolvedLoadHandle>,
    ) -> Self {
        Self {
            handle_ref: HandleRef {
                resolved_load_handle,
                ref_type: HandleRefType::Internal(chan),
            },
            marker: PhantomData,
        }
    }

    pub fn artifact<'a>(
        &self,
        storage: &'a impl TypedArtifactStorage<T>,
    ) -> Option<&'a T> {
        ArtifactHandle::artifact(self, storage)
    }
}

impl<T> ArtifactHandle for Handle<T> {
    fn resolved_load_handle(&self) -> &Arc<ResolvedLoadHandle> {
        &self.handle_ref.resolved_load_handle
    }
}

/// Handle to an artifact whose type is unknown during loading.
///
/// This is returned by `Loader::load_artifact_generic` for artifacts loaded by UUID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericHandle {
    handle_ref: HandleRef,
}

impl GenericHandle {
    /// Creates a new handle with `HandleRefType::Strong`
    pub fn new(
        chan: Sender<RefOp>,
        resolved_load_handle: Arc<ResolvedLoadHandle>,
    ) -> Self {
        Self {
            handle_ref: HandleRef {
                resolved_load_handle,
                ref_type: HandleRefType::Strong(chan),
            },
        }
    }

    /// Creates a new handle with `HandleRefType::Internal`
    pub(crate) fn new_internal(
        chan: Sender<RefOp>,
        resolved_load_handle: Arc<ResolvedLoadHandle>,
    ) -> Self {
        Self {
            handle_ref: HandleRef {
                resolved_load_handle,
                ref_type: HandleRefType::Internal(chan),
            },
        }
    }
}

impl ArtifactHandle for GenericHandle {
    fn resolved_load_handle(&self) -> &Arc<ResolvedLoadHandle> {
        &self.handle_ref.resolved_load_handle
    }
}

impl<T: ?Sized> From<Handle<T>> for GenericHandle {
    fn from(handle: Handle<T>) -> Self {
        Self {
            handle_ref: handle.handle_ref,
        }
    }
}

/// Handle to an artifact that does not prevent the artifact from being unloaded.
///
/// Weak handles are primarily used when you want to use something that is already loaded.
///
/// For example, a strong handle to an artifact may be guaranteed to exist elsewhere in the program,
/// and so you can simply get and use a weak handle to that artifact in other parts of your code. This
/// removes reference counting overhead, but also ensures that the system which uses the weak handle
/// is not in control of when to unload the artifact.
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct WeakHandle {
    resolved_load_handle: Arc<ResolvedLoadHandle>,
}

impl WeakHandle {
    pub fn new(handle: Arc<ResolvedLoadHandle>) -> Self {
        WeakHandle { resolved_load_handle: handle }
    }
}

impl ArtifactHandle for WeakHandle {
    fn resolved_load_handle(&self) -> &Arc<ResolvedLoadHandle> {
        &self.resolved_load_handle
    }
}

std::thread_local!(static LOADER: std::cell::RefCell<Option<&'static dyn LoaderInfoProvider>> = RefCell::new(None));
std::thread_local!(static REFOP_SENDER: std::cell::RefCell<Option<Sender<RefOp>>> = RefCell::new(None));

/// Used to make some limited Loader interactions available to `serde` Serialize/Deserialize
/// implementations by using thread-local storage. Required to support Serialize/Deserialize of Handle.
pub struct SerdeContext;
impl SerdeContext {
    pub fn with_active<R>(f: impl FnOnce(&dyn LoaderInfoProvider, &Sender<RefOp>) -> R) -> R {
        //LOADER.with(|l| REFOP_SENDER.with(|r| f(*l, r)))
        LOADER.with(|loader| {
            //*loader.borrow_mut() = Some(loader_info_provider);
            REFOP_SENDER.with(|refop_sender| {
                (f)(
                    *loader.borrow().as_ref().unwrap(),
                    refop_sender.borrow().as_ref().unwrap(),
                )
                //*refop_sender.borrow_mut() = Some(sender);
                //let output = (f)(l, r);
                //*refop_sender.borrow_mut() = None;
                //output
            })
            //*loader.borrow_mut() = None;
            //output
        })
    }

    pub fn with<T, F>(
        loader: &dyn LoaderInfoProvider,
        sender: Sender<RefOp>,
        f: F,
    ) -> T
    where
        F: FnOnce() -> T,
    {
        // The loader lifetime needs to be transmuted to 'static to be able to be stored in thread_local.
        // This is safe since SerdeContext's lifetime cannot be shorter than the opened scope, and the loader
        // must live at least as long.
        let loader_info_provider = unsafe {
            std::mem::transmute::<&dyn LoaderInfoProvider, &'static dyn LoaderInfoProvider>(loader)
        };

        LOADER.with(|loader| {
            *loader.borrow_mut() = Some(loader_info_provider);
            let output = REFOP_SENDER.with(|refop_sender| {
                *refop_sender.borrow_mut() = Some(sender);
                let output = (f)();
                *refop_sender.borrow_mut() = None;
                output
            });
            *loader.borrow_mut() = None;
            output
        })

        // *LOADER.borrow_mut() = Some(loader);
        // *REFOP_SENDER.borrow_mut() = Some(sender);
        //
        //
        // (*f)();
        //
        // *LOADER.borrow_mut() = None;
        // *REFOP_SENDER.borrow_mut() = None;

        // LOADER.(|x| {
        //
        // })
        //
        // LOADER.scope(loader, REFOP_SENDER.scope(sender, f)).await
    }
}

/// This context can be used to maintain ArtifactId references through a serialize/deserialize cycle
/// even if the LoadHandles produced are invalid. This is useful when a loader is not
/// present, such as when processing in the Distill Daemon.
pub struct DummySerdeContext {
    maps: RwLock<DummySerdeContextMaps>,
    current: Mutex<DummySerdeContextCurrent>,
    ref_sender: Sender<RefOp>,
    handle_gen: AtomicU64,
}

struct DummySerdeContextMaps {
    uuid_to_load: HashMap<ArtifactRef, LoadHandle>,
    load_to_uuid: HashMap<LoadHandle, ArtifactRef>,
}

struct DummySerdeContextCurrent {
    current_serde_dependencies: HashSet<ArtifactRef>,
    current_serde_artifact: Option<ArtifactId>,
}

impl DummySerdeContext {
    pub fn new() -> Self {
        let (tx, _) = crossbeam_channel::unbounded();
        Self {
            maps: RwLock::new(DummySerdeContextMaps {
                uuid_to_load: HashMap::default(),
                load_to_uuid: HashMap::default(),
            }),
            current: Mutex::new(DummySerdeContextCurrent {
                current_serde_dependencies: HashSet::new(),
                current_serde_artifact: None,
            }),
            ref_sender: tx,
            handle_gen: AtomicU64::new(1),
        }
    }
}

impl LoaderInfoProvider for DummySerdeContext {
    fn load_handle(
        &self,
        artifact_ref: &ArtifactRef,
    ) -> Option<Arc<ResolvedLoadHandle>> {
        let mut maps = self.maps.write().unwrap();
        let maps = &mut *maps;
        let uuid_to_load = &mut maps.uuid_to_load;
        let load_to_uuid = &mut maps.load_to_uuid;

        let entry = uuid_to_load.entry(artifact_ref.clone());
        let handle = entry.or_insert_with(|| {
            let new_id = self.handle_gen.fetch_add(1, Ordering::Relaxed);
            let handle = LoadHandle(new_id);
            load_to_uuid.insert(handle, artifact_ref.clone());
            handle
        });

        Some(Arc::new(ResolvedLoadHandle {
            id: *handle,
            direct_load_handle: AtomicU64::new(handle.0),
        }))
    }

    fn artifact_id(
        &self,
        load: LoadHandle,
    ) -> Option<ArtifactId> {
        let maps = self.maps.read().unwrap();
        let maybe_artifact = maps.load_to_uuid.get(&load).cloned();
        if let Some(artifact_ref) = maybe_artifact.as_ref() {
            let mut current = self.current.lock().unwrap();
            if let Some(ref current_serde_id) = current.current_serde_artifact {
                if ArtifactRef(*current_serde_id) != *artifact_ref
                    && *artifact_ref != ArtifactRef(ArtifactId::null())
                {
                    current
                        .current_serde_dependencies
                        .insert(artifact_ref.clone());
                }
            }
        }
        if let Some(ArtifactRef(uuid)) = maybe_artifact {
            Some(uuid)
        } else {
            None
        }
    }
}
pub struct DummySerdeContextHandle {
    dummy: Arc<DummySerdeContext>,
}

impl Default for DummySerdeContextHandle {
    fn default() -> Self {
        DummySerdeContextHandle {
            dummy: Arc::new(DummySerdeContext::new()),
        }
    }
}

impl DummySerdeContextHandle {
    pub fn scope<'a, T, F: FnOnce() -> T>(
        &self,
        f: F,
    ) -> T {
        let sender = self.dummy.ref_sender.clone();
        let loader = &*self.dummy;
        SerdeContext::with(loader, sender, f)
    }

    pub fn resolve_ref(
        &mut self,
        artifact_ref: &ArtifactRef,
        artifact: ArtifactId,
    ) {
        let new_ref = ArtifactRef(artifact);
        let mut maps = self.dummy.maps.write().unwrap();
        if let Some(handle) = maps.uuid_to_load.get(artifact_ref) {
            let handle = *handle;
            maps.load_to_uuid.insert(handle, new_ref.clone());
            maps.uuid_to_load.insert(new_ref, handle);
        }
    }

    /// Begin gathering dependencies for an artifact
    pub fn begin_serialize_artifact(
        &mut self,
        artifact: ArtifactId,
    ) {
        let mut current = self.dummy.current.lock().unwrap();
        if current.current_serde_artifact.is_some() {
            panic!("begin_serialize_artifact when current_serde_artifact is already set");
        }
        current.current_serde_artifact = Some(artifact);
    }

    /// Finish gathering dependencies for an artifact
    pub fn end_serialize_artifact(
        &mut self,
        _artifact: ArtifactId,
    ) -> HashSet<ArtifactRef> {
        let mut current = self.dummy.current.lock().unwrap();
        if current.current_serde_artifact.is_none() {
            panic!("end_serialize_artifact when current_serde_artifact is not set");
        }
        current.current_serde_artifact = None;
        std::mem::take(&mut current.current_serde_dependencies)
    }
}

/// Register this context with ArtifactDaemon to add serde support for Handle.
// pub struct HandleSerdeContextProvider;
// impl crate::importer_context::ImporterContext for HandleSerdeContextProvider {
//     fn handle(&self) -> Box<dyn crate::importer_context::ImporterContextHandle> {
//         let dummy = Arc::new(DummySerdeContext::new());
//         Box::new(DummySerdeContextHandle { dummy })
//     }
// }

fn serialize_handle<S>(
    load: LoadHandle,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    SerdeContext::with_active(|loader, _| {
        use ser::SerializeSeq;
        let uuid_bytes: uuid::Bytes = *loader
            .artifact_id(load)
            .unwrap_or_default()
            .as_uuid()
            .as_bytes();
        let mut seq = serializer.serialize_seq(Some(uuid_bytes.len()))?;
        for element in &uuid_bytes {
            seq.serialize_element(element)?;
        }
        seq.end()
    })
}
impl<T> Serialize for Handle<T> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serialize_handle(self.handle_ref.resolved_load_handle.id, serializer)
    }
}
impl Serialize for GenericHandle {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serialize_handle(self.handle_ref.resolved_load_handle.id, serializer)
    }
}

fn get_handle_ref(artifact_ref: ArtifactRef) -> (Arc<ResolvedLoadHandle>, Sender<RefOp>) {
    SerdeContext::with_active(|loader, sender| {
        let handle = if artifact_ref == ArtifactRef(ArtifactId::default()) {
            ResolvedLoadHandle::new_null_handle()
        } else {
            loader
                .load_handle(&artifact_ref)
                .unwrap_or_else(|| panic!("Handle for ArtifactId {:?} was not present when deserializing a Handle. This indicates missing dependency metadata, and can be caused by dependency cycles.", artifact_ref))
        };
        (handle, sender.clone())
    })
}

impl<'de, T> Deserialize<'de> for Handle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Handle<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let artifact_ref = if deserializer.is_human_readable() {
            deserializer.deserialize_any(ArtifactRefVisitor)?
        } else {
            deserializer.deserialize_seq(ArtifactRefVisitor)?
        };
        let (handle, sender) = get_handle_ref(artifact_ref);
        Ok(Handle::new_internal(sender, handle))
    }
}

impl<'de> Deserialize<'de> for GenericHandle {
    fn deserialize<D>(deserializer: D) -> Result<GenericHandle, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let artifact_ref = if deserializer.is_human_readable() {
            deserializer.deserialize_any(ArtifactRefVisitor)?
        } else {
            deserializer.deserialize_seq(ArtifactRefVisitor)?
        };
        let (handle, sender) = get_handle_ref(artifact_ref);
        Ok(GenericHandle::new_internal(sender, handle))
    }
}

struct ArtifactRefVisitor;

impl<'de> de::Visitor<'de> for ArtifactRefVisitor {
    type Value = ArtifactRef;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        formatter.write_str("an array of 16 u8")
    }

    fn visit_newtype_struct<D>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }

    fn visit_seq<A>(
        self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        use de::Error;
        let mut uuid: [u8; 16] = Default::default();
        for (i, uuid_byte) in uuid.iter_mut().enumerate() {
            if let Some(byte) = seq.next_element::<u8>()? {
                *uuid_byte = byte;
            } else {
                return Err(A::Error::custom(format!(
                    "expected byte at element {} when deserializing handle",
                    i
                )));
            }
        }
        if seq.next_element::<u8>()?.is_some() {
            return Err(A::Error::custom(
                "too many elements when deserializing handle",
            ));
        }
        Ok(ArtifactRef(ArtifactId::from_uuid(Uuid::from_bytes(uuid))))
    }

    fn visit_str<E>(
        self,
        v: &str,
    ) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(uuid) = Uuid::parse_str(v) {
            Ok(ArtifactRef(ArtifactId::from_uuid(uuid)))
        } else {
            Err(E::custom(format!("failed to parse Handle string")))
        }
    }

    fn visit_bytes<E>(
        self,
        v: &[u8],
    ) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() != 16 {
            Err(E::custom(format!(
                "byte array len == {}, expected {}",
                v.len(),
                16
            )))
        } else {
            let mut a = <[u8; 16]>::default();
            a.copy_from_slice(v);
            Ok(ArtifactRef(ArtifactId::from_uuid(Uuid::from_bytes(a))))
        }
    }
}

/// Implementors of [`crate::storage::ArtifactStorage`] can implement this trait to enable convenience
/// functions on the common [`ArtifactHandle`] trait, which is implemented by all handle types.
pub trait TypedArtifactStorage<A> {
    /// Returns the artifact for the given handle, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the artifact.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Artifact handle type.
    fn get<T: ArtifactHandle>(
        &self,
        handle: &T,
    ) -> Option<&A>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LoadState {
    // Not loaded, and we haven't started trying to load it. Ref count > 0 implies we want to start
    // loading.
    Unloaded,
    // Metadata request is in flight
    WaitingForMetadata,
    // We've incremented ref counts for dependencies, but they aren't loaded yet
    WaitingForDependencies,
    // Dependencies are loaded, and we have requested the data required to load this artifact
    WaitingForData,
    // Data has been passed off to end-user's loader
    Loading,
    // The engine finished loading the artifact but it is not available to the game yet
    // When hot reloading, we delay commit until we have loaded new versions of all changed artifacts,
    // so engine never sees a partial reload
    Loaded,
    // The artifact has been committed and is visible to the game
    Committed,
}

// This allows a handle in hydrate_base to get information from the loader which may be in hydrate_loader
// or some other loader implementation
pub trait LoadStateProvider {
    fn load_state(
        &self,
        load_handle: &Arc<ResolvedLoadHandle>,
    ) -> LoadState;
    fn artifact_id(
        &self,
        load_handle: &Arc<ResolvedLoadHandle>,
    ) -> ArtifactId;
}

/// The contract of an artifact handle.
///
/// There are two types of artifact handles:
///
/// * **Typed -- `Handle<T>`:** When the artifact's type is known when loading.
/// * **Generic -- `GenericHandle`:** When only the artifact's UUID is known when loading.
pub trait ArtifactHandle {
    /// Returns the load status of the artifact.
    ///
    /// # Parameters
    ///
    /// * `loader`: Loader that is loading the artifact.
    ///
    /// # Type Parameters
    ///
    /// * `L`: Artifact loader type.
    fn load_state<T: LoadStateProvider>(
        &self,
        loader: &T,
    ) -> LoadState {
        loader.load_state(self.resolved_load_handle())
    }

    fn artifact_id<T: LoadStateProvider>(
        &self,
        loader: &T,
    ) -> ArtifactId {
        loader.artifact_id(self.resolved_load_handle())
    }

    /// Returns an immutable reference to the artifact if it is committed.
    ///
    /// # Parameters
    ///
    /// * `storage`: Artifact storage.
    fn artifact<'a, T, S: TypedArtifactStorage<T>>(
        &self,
        storage: &'a S,
    ) -> Option<&'a T>
    where
        Self: Sized,
    {
        storage.get(self)
    }

    /// Downgrades this handle into a `WeakHandle`.
    ///
    /// Be aware that if there are no longer any strong handles to the artifact, then the underlying
    /// artifact may be freed at any time.
    fn downgrade(&self) -> WeakHandle {
        WeakHandle::new(self.resolved_load_handle().clone())
    }

    fn resolved_load_handle(&self) -> &Arc<ResolvedLoadHandle>;

    /// Returns the `LoadHandle` of this artifact handle.
    fn load_handle(&self) -> LoadHandle {
        self.resolved_load_handle().id
    }

    fn direct_load_handle(&self) -> LoadHandle {
        self.resolved_load_handle().direct_load_handle()
    }
}

pub fn make_handle_within_serde_context<T>(uuid: ArtifactId) -> Handle<T> {
    SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
        let load_handle = loader_info_provider
            .load_handle(&ArtifactRef(uuid))
            .unwrap();
        Handle::<T>::new(ref_op_sender.clone(), load_handle)
    })
}

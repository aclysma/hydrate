use super::{JobId, JobTypeId};
use crate::build::{AssetArtifactIdPair, BuiltArtifact};
use crate::import::{ImportData, ImportJobs};
use crate::{BuildLogEvent, LogEventLevel, PipelineResult};
use hydrate_base::handle::DummySerdeContextHandle;
use hydrate_base::hashing::HashMap;
use hydrate_base::{ArtifactId, AssetId, BuiltArtifactHeaderData, Handle};
use hydrate_data::{
    DataContainerRef, DataSet, DataSetError, FieldRef, HashObjectMode, PropertyPath, Record,
    SchemaSet, SingleObject,
};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use std::cell::RefCell;
use std::hash::Hash;
use std::panic::RefUnwindSafe;
use std::rc::Rc;
use std::sync::Arc;
use type_uuid::{TypeUuid, TypeUuidDynamic};

pub trait ImportDataProvider {
    fn clone_import_data_metadata_hashes(&self) -> HashMap<AssetId, u64>;

    fn load_import_data(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
    ) -> PipelineResult<ImportData>;
}

impl ImportDataProvider for ImportJobs {
    fn clone_import_data_metadata_hashes(&self) -> HashMap<AssetId, u64> {
        self.clone_import_data_metadata_hashes()
    }

    fn load_import_data(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
    ) -> PipelineResult<ImportData> {
        crate::import::load_import_data(self.import_data_root_path(), schema_set, asset_id)
    }
}

pub struct NewJob {
    pub job_type: JobTypeId,
    pub input_hash: u128,
    pub input_data: Vec<u8>,
}

fn create_artifact_id<T: Hash>(
    asset_id: AssetId,
    artifact_key: Option<T>,
) -> ArtifactId {
    if let Some(artifact_key) = artifact_key {
        let mut hasher = siphasher::sip128::SipHasher::default();
        asset_id.hash(&mut hasher);
        artifact_key.hash(&mut hasher);
        let input_hash = hasher.finish128().as_u128();
        ArtifactId::from_u128(input_hash)
    } else {
        ArtifactId::from_uuid(asset_id.as_uuid())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum JobRequestor {
    Builder(AssetId),
    Job(JobId),
}

//
// API Design
//
pub trait JobApi: Send + Sync {
    fn enqueue_job(
        &self,
        job_requestor: JobRequestor,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job: NewJob,
        debug_name: String,
        log_events: &mut Vec<BuildLogEvent>,
    ) -> PipelineResult<JobId>;

    fn artifact_handle_created(
        &self,
        asset_id: AssetId,
        artifact_id: ArtifactId,
    );

    fn produce_artifact(
        &self,
        artifact: BuiltArtifact,
    );

    fn fetch_import_data(
        &self,
        asset_id: AssetId,
    ) -> PipelineResult<ImportData>;
}

//
// Job Traits
//
pub trait JobInput: Hash + Serialize + for<'a> Deserialize<'a> {}

pub trait JobOutput: Serialize + for<'a> Deserialize<'a> {}

#[derive(Default, Clone)]
pub struct JobEnumeratedDependencies {
    // The contents of assets can affect the output so we need to include a hash of the contents of
    // the asset. But assets can ref other assets, task needs to list all assets that are touched
    // (including prototypes of those assets).
    //
    // We could do it at asset type granularity? (i.e. if you change an asset of type X all jobs that
    // read an asset of type X have to rerun.
    //
    // What if we provide a data_set reader that keeps track of what was read? When we run the task
    // the first time we don't know what we will touch or how to hash it but we can store it. Second
    // build we can check if anything that was read last time was modified.
    //
    // Alternatively, jobs that read assets must always copy data out of the data set into a hashable
    // form and pass it as input to a job.
    //pub import_data: Vec<AssetId>,
    //pub built_data: Vec<ArtifactId>,
    pub upstream_jobs: Vec<JobId>,
}

pub(crate) trait JobProcessorAbstract: Send + Sync + RefUnwindSafe {
    fn version_inner(&self) -> u32;

    fn enumerate_dependencies_inner(
        &self,
        job_id: JobId,
        job_requestor: JobRequestor,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        log_events: &mut Vec<BuildLogEvent>,
    ) -> PipelineResult<JobEnumeratedDependencies>;

    fn run_inner(
        &self,
        job_id: JobId,
        input: &Vec<u8>,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
        fetched_asset_data: &mut HashMap<AssetId, FetchedAssetData>,
        fetched_import_data: &mut HashMap<AssetId, FetchedImportData>,
        log_events: &mut Vec<BuildLogEvent>,
    ) -> PipelineResult<Arc<Vec<u8>>>;
}

pub struct EnumerateDependenciesContext<'a, InputT> {
    pub job_id: JobId,
    pub(crate) job_requestor: JobRequestor,
    pub input: &'a InputT,
    pub data_set: &'a DataSet,
    pub schema_set: &'a SchemaSet,
    pub(crate) log_events: &'a Rc<RefCell<&'a mut Vec<BuildLogEvent>>>,
}

impl<'a, InputT> EnumerateDependenciesContext<'a, InputT> {
    pub fn warn<T: Into<String>>(
        &self,
        message: T,
    ) {
        let (asset_id, job_id) = match self.job_requestor {
            JobRequestor::Builder(asset_id) => (Some(asset_id), None),
            JobRequestor::Job(job_id) => (None, Some(job_id)),
        };

        let mut log_events = self.log_events.borrow_mut();
        let log_event = BuildLogEvent {
            asset_id,
            job_id,
            level: LogEventLevel::Warning,
            message: format!(
                "While enumerating dependencies for new job {}: {}",
                self.job_id.as_uuid(),
                message.into()
            ),
        };
        log::warn!("Build Warning: {:?}", log_event);
        log_events.push(log_event);
    }

    pub fn error<T: Into<String>>(
        &self,
        message: T,
    ) {
        let (asset_id, job_id) = match self.job_requestor {
            JobRequestor::Builder(asset_id) => (Some(asset_id), None),
            JobRequestor::Job(job_id) => (None, Some(job_id)),
        };

        let mut log_events = self.log_events.borrow_mut();
        let log_event = BuildLogEvent {
            asset_id,
            job_id,
            level: LogEventLevel::Error,
            message: format!(
                "While enumerating dependencies for new job {}: {}",
                self.job_id.as_uuid(),
                message.into()
            ),
        };
        log::error!("Build Error: {:?}", log_event);
        log_events.push(log_event);
    }
}

pub(crate) struct FetchedAssetData {
    pub(crate) _contents_hash: u64,
}

pub(crate) struct FetchedImportDataInfo {
    pub(crate) _contents_hash: u64,
    pub(crate) _metadata_hash: u64,
}

pub(crate) struct FetchedImportData {
    pub(crate) _info: FetchedImportDataInfo,
    pub(crate) import_data: Arc<SingleObject>,
}

#[derive(Copy, Clone)]
pub struct RunContext<'a, InputT> {
    pub job_id: JobId,
    pub input: &'a InputT,
    pub data_set: &'a DataSet,
    pub schema_set: &'a SchemaSet,
    pub(crate) fetched_asset_data: &'a Rc<RefCell<&'a mut HashMap<AssetId, FetchedAssetData>>>,
    pub(crate) fetched_import_data: &'a Rc<RefCell<&'a mut HashMap<AssetId, FetchedImportData>>>,
    pub(crate) job_api: &'a dyn JobApi,
    pub(crate) log_events: &'a Rc<RefCell<&'a mut Vec<BuildLogEvent>>>,
}

impl<'a, InputT> RunContext<'a, InputT> {
    pub fn warn<T: Into<String>>(
        &self,
        message: T,
    ) {
        let mut log_events = self.log_events.borrow_mut();
        let log_event = BuildLogEvent {
            asset_id: None,
            job_id: Some(self.job_id),
            level: LogEventLevel::Warning,
            message: message.into(),
        };
        log::warn!("Build Warning: {:?}", log_event);
        log_events.push(log_event);
    }

    pub fn error<T: Into<String>>(
        &self,
        message: T,
    ) {
        let mut log_events = self.log_events.borrow_mut();
        let log_event = BuildLogEvent {
            asset_id: None,
            job_id: Some(self.job_id),
            level: LogEventLevel::Error,
            message: message.into(),
        };
        log::error!("Build Error: {:?}", log_event);
        log_events.push(log_event);
    }

    pub fn asset<T: Record>(
        &'a self,
        asset_id: AssetId,
    ) -> PipelineResult<T::Reader<'a>> {
        if self
            .data_set
            .asset_schema(asset_id)
            .ok_or(DataSetError::AssetNotFound)?
            .name()
            != T::schema_name()
        {
            Err(DataSetError::InvalidSchema)?;
        }

        let mut fetched_asset_data = self.fetched_asset_data.borrow_mut();
        fetched_asset_data
            .entry(asset_id)
            .or_insert_with(|| FetchedAssetData {
                _contents_hash: self
                    .data_set
                    .hash_object(asset_id, HashObjectMode::PropertiesOnly)
                    .unwrap(),
            });

        Ok(<T as Record>::Reader::new(
            PropertyPath::default(),
            DataContainerRef::from_dataset(self.data_set, self.schema_set, asset_id),
        ))
    }

    pub fn imported_data<T: Record>(
        &'a self,
        asset_id: AssetId,
    ) -> PipelineResult<T::Reader<'a>> {
        let mut fetched_import_data = self.fetched_import_data.borrow_mut();
        let import_data = if let Some(fetched_import_data) = fetched_import_data.get(&asset_id) {
            fetched_import_data.import_data.clone()
        } else {
            let newly_fetched_import_data = self.job_api.fetch_import_data(asset_id)?;
            let import_data = Arc::new(newly_fetched_import_data.import_data);

            let old = fetched_import_data.insert(
                asset_id,
                FetchedImportData {
                    import_data: import_data.clone(),
                    _info: FetchedImportDataInfo {
                        _contents_hash: newly_fetched_import_data.contents_hash,
                        _metadata_hash: newly_fetched_import_data.metadata_hash,
                    },
                },
            );
            assert!(old.is_none());
            import_data
        };

        if import_data.schema().name() != T::schema_name() {
            Err(DataSetError::InvalidSchema)?;
        }

        return Ok(<T as Record>::Reader::new(
            PropertyPath::default(),
            DataContainerRef::from_single_object_arc(import_data.clone(), self.schema_set),
        ));
    }

    pub fn enqueue_job<JobProcessorT: JobProcessor>(
        &self,
        input: <JobProcessorT as JobProcessor>::InputT,
    ) -> PipelineResult<JobId> {
        enqueue_job::<JobProcessorT>(
            JobRequestor::Job(self.job_id),
            self.data_set,
            self.schema_set,
            self.job_api,
            input,
            &mut self.log_events.borrow_mut(),
        )
    }

    pub fn produce_artifact<KeyT: Hash + std::fmt::Display, ArtifactT: TypeUuid + Serialize>(
        &self,
        asset_id: AssetId,
        artifact_key: Option<KeyT>,
        asset: ArtifactT,
    ) -> PipelineResult<AssetArtifactIdPair> {
        produce_artifact(self.job_api, asset_id, artifact_key, asset)
    }

    pub fn produce_artifact_with_handles<
        KeyT: Hash + std::fmt::Display,
        ArtifactT: TypeUuid + Serialize,
        F: FnOnce(HandleFactory) -> PipelineResult<ArtifactT>,
    >(
        &self,
        asset_id: AssetId,
        artifact_key: Option<KeyT>,
        asset_fn: F,
    ) -> PipelineResult<ArtifactId> {
        produce_artifact_with_handles(self.job_api, asset_id, artifact_key, asset_fn)
    }

    pub fn produce_default_artifact<AssetT: TypeUuid + Serialize>(
        &self,
        asset_id: AssetId,
        asset: AssetT,
    ) -> PipelineResult<ArtifactId> {
        produce_default_artifact(self.job_api, asset_id, asset)
    }

    pub fn produce_default_artifact_with_handles<
        AssetT: TypeUuid + Serialize,
        F: FnOnce(HandleFactory) -> PipelineResult<AssetT>,
    >(
        &self,
        asset_id: AssetId,
        asset_fn: F,
    ) -> PipelineResult<ArtifactId> {
        produce_default_artifact_with_handles(self.job_api, asset_id, asset_fn)
    }
}

pub trait JobProcessor: TypeUuid {
    type InputT: JobInput + 'static;
    type OutputT: JobOutput + 'static;

    fn version(&self) -> u32;

    fn enumerate_dependencies(
        &self,
        _context: EnumerateDependenciesContext<Self::InputT>,
    ) -> PipelineResult<JobEnumeratedDependencies> {
        Ok(JobEnumeratedDependencies::default())
    }

    fn run<'a>(
        &'a self,
        context: &'a RunContext<'a, Self::InputT>,
    ) -> PipelineResult<Self::OutputT>;
}

pub(crate) fn enqueue_job<T: JobProcessor>(
    job_requestor: JobRequestor,
    data_set: &DataSet,
    schema_set: &SchemaSet,
    job_api: &dyn JobApi,
    input: <T as JobProcessor>::InputT,
    log_events: &mut Vec<BuildLogEvent>,
) -> PipelineResult<JobId> {
    let mut hasher = siphasher::sip128::SipHasher::default();
    input.hash(&mut hasher);
    let input_hash = hasher.finish128().as_u128();

    let input_data = bincode::serialize(&input).unwrap();

    let queued_job = NewJob {
        job_type: JobTypeId::from_bytes(T::UUID),
        input_hash,
        input_data,
    };

    let debug_name = format!("{}", std::any::type_name::<T>());
    job_api.enqueue_job(
        job_requestor,
        data_set,
        schema_set,
        queued_job,
        debug_name,
        log_events,
    )
}

fn produce_default_artifact<T: TypeUuid + Serialize>(
    job_api: &dyn JobApi,
    asset_id: AssetId,
    asset: T,
) -> PipelineResult<ArtifactId> {
    produce_artifact_with_handles(job_api, asset_id, None::<u32>, |_handle_factory| Ok(asset))
}

fn produce_default_artifact_with_handles<
    T: TypeUuid + Serialize,
    F: FnOnce(HandleFactory) -> PipelineResult<T>,
>(
    job_api: &dyn JobApi,
    asset_id: AssetId,
    asset_fn: F,
) -> PipelineResult<ArtifactId> {
    produce_artifact_with_handles(job_api, asset_id, None::<u32>, asset_fn)
}

fn produce_artifact<T: TypeUuid + Serialize, U: Hash + std::fmt::Display>(
    job_api: &dyn JobApi,
    asset_id: AssetId,
    artifact_key: Option<U>,
    asset: T,
) -> PipelineResult<AssetArtifactIdPair> {
    let artifact_id =
        produce_artifact_with_handles(job_api, asset_id, artifact_key, |_handle_factory| {
            Ok(asset)
        })?;
    Ok(AssetArtifactIdPair {
        asset_id,
        artifact_id,
    })
}

fn produce_artifact_with_handles<
    T: TypeUuid + Serialize,
    U: Hash + std::fmt::Display,
    F: FnOnce(HandleFactory) -> PipelineResult<T>,
>(
    job_api: &dyn JobApi,
    asset_id: AssetId,
    artifact_key: Option<U>,
    asset_fn: F,
) -> PipelineResult<ArtifactId> {
    let artifact_key_debug_name = artifact_key.as_ref().map(|x| format!("{}", x));
    let artifact_id = create_artifact_id(asset_id, artifact_key);

    let mut ctx = DummySerdeContextHandle::default();
    ctx.begin_serialize_artifact(artifact_id);

    let (built_data, asset_type) = ctx.scope(|| {
        let asset = (asset_fn)(HandleFactory { job_api });
        asset.map(|x| (bincode::serialize(&x), x.uuid()))
    })?;

    let referenced_assets = ctx.end_serialize_artifact(artifact_id);

    log::trace!(
        "produce_artifact {:?} {:?} {:?}",
        asset_id,
        artifact_id,
        artifact_key_debug_name
    );
    job_api.produce_artifact(BuiltArtifact {
        asset_id,
        artifact_id,
        metadata: BuiltArtifactHeaderData {
            dependencies: referenced_assets
                .into_iter()
                .map(|x| ArtifactId::from_uuid(x.0.as_uuid()))
                .collect(),
            asset_type: uuid::Uuid::from_bytes(asset_type),
        },
        data: built_data?,
        artifact_key_debug_name,
    });

    Ok(artifact_id)
}

#[derive(Copy, Clone)]
pub struct HandleFactory<'a> {
    job_api: &'a dyn JobApi,
}

impl<'a> HandleFactory<'a> {
    pub fn make_handle_to_default_artifact<T>(
        &self,
        asset_id: AssetId,
    ) -> Handle<T> {
        self.make_handle_to_artifact_key(asset_id, None::<u32>)
    }

    pub fn make_handle_to_artifact<T>(
        &self,
        asset_artifact_id_pair: AssetArtifactIdPair,
    ) -> Handle<T> {
        self.job_api.artifact_handle_created(
            asset_artifact_id_pair.asset_id,
            asset_artifact_id_pair.artifact_id,
        );
        hydrate_base::handle::make_handle_within_serde_context::<T>(
            asset_artifact_id_pair.artifact_id,
        )
    }

    pub fn make_handle_to_artifact_raw<T>(
        &self,
        asset_id: AssetId,
        artifact_id: ArtifactId,
    ) -> Handle<T> {
        self.job_api.artifact_handle_created(asset_id, artifact_id);
        hydrate_base::handle::make_handle_within_serde_context::<T>(artifact_id)
    }

    pub fn make_handle_to_artifact_key<T, K: Hash>(
        &self,
        asset_id: AssetId,
        artifact_key: Option<K>,
    ) -> Handle<T> {
        let artifact_id = create_artifact_id(asset_id, artifact_key);
        self.job_api.artifact_handle_created(asset_id, artifact_id);
        hydrate_base::handle::make_handle_within_serde_context::<T>(artifact_id)
    }
}

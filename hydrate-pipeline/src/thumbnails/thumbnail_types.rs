use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use type_uuid::TypeUuid;
use hydrate_base::{ArtifactId, AssetId};
use hydrate_base::hashing::HashMap;
use hydrate_data::{DataContainerRef, DataSet, FieldRef, PropertyPath, Record, SchemaSet};
use hydrate_schema::{DataSetError, HashSet};
use crate::build::{BuiltArtifact, FetchedImportData, FetchedImportDataInfo, NewJob};
use crate::import::ImportData;
use crate::{HydrateProjectConfiguration, JobId, PipelineResult};
use crate::thumbnails::thumbnail_system::ThumbnailImage;

// pub trait ThumbnailApi: Send + Sync {
//     // fn enqueue_job(
//     //     &self,
//     //     data_set: &DataSet,
//     //     schema_set: &SchemaSet,
//     //     job: NewJob,
//     //     debug_name: String,
//     // ) -> PipelineResult<JobId>;
//     //
//     // fn artifact_handle_created(
//     //     &self,
//     //     asset_id: AssetId,
//     //     artifact_id: ArtifactId,
//     // );
//     //
//     // fn produce_artifact(
//     //     &self,
//     //     artifact: BuiltArtifact,
//     // );
//
//     fn fetch_import_data(
//         &self,
//         asset_id: AssetId,
//     ) -> PipelineResult<ImportData>;
// }

struct ThumbnailApiInner {
    hydrate_config: HydrateProjectConfiguration,
    schema_set: SchemaSet,
}

#[derive(Clone)]
pub struct ThumbnailApi {
    inner: Arc<ThumbnailApiInner>,
}

impl ThumbnailApi {
    pub fn new(hydrate_config: &HydrateProjectConfiguration, schema_set: &SchemaSet) -> Self {
        let inner = ThumbnailApiInner {
            schema_set: schema_set.clone(),
            hydrate_config: hydrate_config.clone()
        };

        ThumbnailApi {
            inner: Arc::new(inner)
        }
    }

    pub fn fetch_import_data(&self, asset_id: AssetId) -> PipelineResult<ImportData> {
        crate::import::load_import_data(&self.inner.hydrate_config.import_data_path, &self.inner.schema_set, asset_id)
    }
}


pub struct ThumbnailProviderGatherContext<'a> {
    pub asset_id: AssetId,
    pub data_set: &'a DataSet,
    pub schema_set: &'a SchemaSet,
    pub(crate) import_data_dependencies: &'a Rc<RefCell<&'a mut HashSet<AssetId>>>,
}

impl<'a> ThumbnailProviderGatherContext<'a> {
    pub fn add_import_data_dependency(&self, asset_id: AssetId) {
        self.import_data_dependencies.borrow_mut().insert(asset_id);
    }
}

pub struct ThumbnailProviderRenderContext<'a> {
    pub asset_id: AssetId,
    pub schema_set: &'a SchemaSet,
    pub(crate) fetched_import_data: &'a Rc<RefCell<&'a mut HashMap<AssetId, FetchedImportData>>>,
    pub(crate) thumbnail_api: &'a ThumbnailApi
}

impl<'a> ThumbnailProviderRenderContext<'a> {
    pub fn imported_data<T: Record>(
        &'a self,
        asset_id: AssetId,
    ) -> PipelineResult<T::Reader<'a>> {
        let mut fetched_import_data = self.fetched_import_data.borrow_mut();
        let import_data = if let Some(fetched_import_data) = fetched_import_data.get(&asset_id) {
            fetched_import_data.import_data.clone()
        } else {

            let newly_fetched_import_data = self.thumbnail_api.fetch_import_data(asset_id)?;
            let import_data = Arc::new(newly_fetched_import_data.import_data);

            let old = fetched_import_data.insert(
                asset_id,
                FetchedImportData {
                    import_data: import_data.clone(),
                    info: FetchedImportDataInfo {
                        contents_hash: newly_fetched_import_data.contents_hash,
                        metadata_hash: newly_fetched_import_data.metadata_hash,
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
}


pub trait ThumbnailProvider {
    type GatheredDataT: Hash + 'static;

    fn asset_type(&self) -> &'static str;

    fn version(&self) -> u32;

    // Is given access to all data needed to produce the thumbnail. This call acts as the logic
    // that decides if a thumbnail needs to be refreshed.
    fn gather(&self, context: ThumbnailProviderGatherContext) -> Self::GatheredDataT;

    // Can only read gathered data provided by gather(), import data, and build data. The import/build
    // data has to match the hash of what was requested by gather()
    fn render<'a>(
        &'a self,
        context: &'a ThumbnailProviderRenderContext<'a>,
        gathered_data: Self::GatheredDataT
    ) -> PipelineResult<ThumbnailImage>;
}

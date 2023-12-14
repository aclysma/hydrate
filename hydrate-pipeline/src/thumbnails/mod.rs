mod thumbnail_provider_registry;
pub use thumbnail_provider_registry::*;

mod thumbnail_types;
pub use thumbnail_types::*;

mod thumbnail_system;
pub use thumbnail_system::ThumbnailSystem;
pub use thumbnail_system::ThumbnailSystemState;
pub use thumbnail_system::ThumbnailImage;

use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::TypeUuid;
use hydrate_base::AssetId;
use hydrate_data::{DataSet, SchemaSet};
use hydrate_schema::HashSet;
use crate::{JobOutput, JobProcessor, PipelineResult};
use crate::build::{JobApi, JobProcessorAbstract};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThumbnailProviderId(pub usize);

pub(crate) struct ThumbnailEnumeratedDependencies {
    pub(crate) thumbnail_input_hash: u128,
    pub(crate) gathered_data: Arc<Vec<u8>>,
    pub(crate) import_data: HashSet<AssetId>,
}

trait ThumbnailProviderAbstract {
    // The type of asset that this builder handles
    fn asset_type_inner(&self) -> &'static str;

    fn version_inner(&self) -> u32;

    fn gather_inner(
        &self,
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> PipelineResult<ThumbnailEnumeratedDependencies>;

    fn render_inner(
        &self,
        asset_id: AssetId,
        gathered_data: &Vec<u8>,
        // data_set: &DataSet,
        // schema_set: &SchemaSet,
        // job_api: &dyn JobApi,
        // fetched_asset_data: &mut HashMap<AssetId, FetchedAssetData>,
        // fetched_import_data: &mut HashMap<AssetId, FetchedImportData>,
    ) -> PipelineResult<ThumbnailImage>; // return something?
}

struct ThumbnailProviderWrapper<T: ThumbnailProvider>(T);

impl<T: ThumbnailProvider + Send + Sync> ThumbnailProviderAbstract for ThumbnailProviderWrapper<T>
    where
        <T as ThumbnailProvider>::GatheredDataT: Hash + for<'a> serde::Deserialize<'a> + serde::Serialize
{
    fn asset_type_inner(&self) -> &'static str {
        self.0.asset_type()
    }

    fn version_inner(&self) -> u32 {
        self.0.version()
    }

    fn gather_inner(
        &self,
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet
    ) -> PipelineResult<ThumbnailEnumeratedDependencies> {
        let mut import_data = HashSet::default();
        let gathered_data = self.0.gather(ThumbnailProviderGatherContext {
            asset_id,
            data_set,
            schema_set,
            import_data_dependencies: &Rc::new(RefCell::new(&mut import_data))
        });

        let mut hasher = siphasher::sip128::SipHasher::default();
        gathered_data.hash(&mut hasher);
        let mut thumbnail_input_hash = hasher.finish128().as_u128();

        // Has the import data with xor because we don't have deterministic ordering of asset IDs
        for asset_id in &import_data {
            let mut hasher_inner = siphasher::sip128::SipHasher::default();
            asset_id.hash(&mut hasher_inner);
            thumbnail_input_hash ^= hasher_inner.finish128().as_u128();
        }

        let gathered_data = Arc::new(bincode::serialize(&gathered_data)?);

        Ok(ThumbnailEnumeratedDependencies {
            thumbnail_input_hash,
            gathered_data,
            import_data
        })
    }

    fn render_inner(
        &self,
        asset_id: AssetId,
        gathered_data: &Vec<u8>,
        // input: &Vec<u8>,
        // data_set: &DataSet,
        // schema_set: &SchemaSet,
        // job_api: &dyn JobApi,
        // fetched_asset_data: &mut HashMap<AssetId, FetchedAssetData>,
        // fetched_import_data: &mut HashMap<AssetId, FetchedImportData>
    ) -> PipelineResult<ThumbnailImage> {
        let gathered_data: T::GatheredDataT = bincode::deserialize(&*gathered_data)?;
        let image = self.0.render(ThumbnailProviderRenderContext {
            asset_id,
        }, gathered_data).unwrap();

        Ok(image)
    }
}


use crate::build::JobExecutor;
use crate::thumbnails::thumbnail_thread_pool::{
    ThumbnailThreadPool, ThumbnailThreadPoolOutcome, ThumbnailThreadPoolRequest,
    ThumbnailThreadPoolRequestRunJob,
};
use crate::thumbnails::ThumbnailProviderRegistry;
use crate::{HydrateProjectConfiguration, ThumbnailApi, ThumbnailInputHash};
use crossbeam_channel::Receiver;
use hydrate_base::hashing::HashMap;
use hydrate_base::lru_cache::LruCache;
use hydrate_base::AssetId;
use hydrate_data::{DataSet, SchemaSet};
use hydrate_schema::{HashSet, SchemaFingerprint};
use std::sync::{Arc, Mutex};

// Thumbnail providers are implemented per asset type
// - Implement a gather method that runs in main thread and can see asset data
//   This method will indicate all input data to produce the thumbnail.
// - Implement a render method that will run off-main-thread. It can read import data and only
//   data gathered from assets in gather().
// These providers are placed in a registry, this handles the type erasure for us to operate on them as a set
// The system will keep track of requested thumbnails, thumbnail state (not created, pending, created, etc.)
// and the registry
// Thumbnails need to be invalidated, we will use metadata returned from gather() to determine this

const THUMBNAIL_CACHE_SIZE: u32 = 1024;
const STALENESS_CHECK_TIME_MILLISECONDS: u128 = 1000;

pub struct ThumbnailImage {
    pub width: u32,
    pub height: u32,
    // Basic 8-bit RBGA
    pub pixel_data: Vec<u8>,
}

#[derive(Default)]
pub struct ThumbnailState {
    // List of asset dependencies
    // List of import data dependencies
    image: Option<Arc<ThumbnailImage>>,
    // Set when the image is loaded
    current_input_hash: Option<ThumbnailInputHash>,
    // Set when the image request is queued and cleared when it completes
    queued_request_input_hash: Option<ThumbnailInputHash>,
    failed_to_load: bool,
    last_staleness_check: Option<std::time::Instant>,
}

struct ThumbnailSystemStateInner {
    cache: LruCache<AssetId, ThumbnailState>,
}

#[derive(Clone)]
pub struct ThumbnailSystemState {
    pub inner: Arc<Mutex<ThumbnailSystemStateInner>>,
}

impl Default for ThumbnailSystemState {
    fn default() -> Self {
        ThumbnailSystemState {
            inner: Arc::new(Mutex::new(ThumbnailSystemStateInner {
                cache: LruCache::new(THUMBNAIL_CACHE_SIZE),
            })),
        }
    }
}

impl ThumbnailSystemState {
    pub fn request(
        &self,
        asset_id: AssetId,
    ) -> Option<Arc<ThumbnailImage>> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(thumbnail_state) = inner.cache.get(&asset_id, true) {
            thumbnail_state.image.clone()
        } else {
            inner.cache.insert(asset_id, ThumbnailState::default());
            None
        }
    }

    pub fn forget(
        &self,
        asset_id: AssetId,
    ) {
    }

    pub fn forget_all(&self) {}
}

pub struct ThumbnailSystem {
    // Thumbnails that have been requested, created, etc.
    thumbnail_system_state: ThumbnailSystemState,
    thumbnail_provider_registry: ThumbnailProviderRegistry,
    default_image: Arc<ThumbnailImage>,
    thumbnail_api: ThumbnailApi,
    thread_pool: Option<ThumbnailThreadPool>,
    thread_pool_result_rx: Receiver<ThumbnailThreadPoolOutcome>,
    current_requests: HashSet<ThumbnailInputHash>,
}

impl Drop for ThumbnailSystem {
    fn drop(&mut self) {
        let thread_pool = self.thread_pool.take().unwrap();
        thread_pool.finish();
    }
}

impl ThumbnailSystem {
    pub fn system_state(&self) -> &ThumbnailSystemState {
        &self.thumbnail_system_state
    }

    pub fn thumbnail_provider_registry(&self) -> &ThumbnailProviderRegistry {
        &self.thumbnail_provider_registry
    }

    pub fn new(
        hydrate_config: &HydrateProjectConfiguration,
        thumbnail_provider_registry: ThumbnailProviderRegistry,
        schema_set: &SchemaSet,
    ) -> Self {
        let default_image = ThumbnailImage {
            width: 1,
            height: 1,
            pixel_data: vec![0, 0, 0, 255],
        };

        let thumbnail_api = ThumbnailApi::new(hydrate_config, schema_set);

        let (thread_pool_result_tx, thread_pool_result_rx) = crossbeam_channel::unbounded();
        let thread_pool = ThumbnailThreadPool::new(
            thumbnail_provider_registry.clone(),
            schema_set.clone(),
            thumbnail_api.clone(),
            4,
            thread_pool_result_tx,
        );

        ThumbnailSystem {
            thumbnail_system_state: ThumbnailSystemState::default(),
            thumbnail_provider_registry,
            default_image: Arc::new(default_image),
            thumbnail_api: ThumbnailApi::new(hydrate_config, schema_set),
            thread_pool: Some(thread_pool),
            thread_pool_result_rx,
            current_requests: Default::default(),
        }
    }

    pub fn update(
        &mut self,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) {
        let now = std::time::Instant::now();
        let mut state = self.thumbnail_system_state.inner.lock().unwrap();
        for (asset_id, thumbnail_state) in state
            .cache
            .pairs_mut()
            .iter_mut()
            .filter_map(|x| x.as_mut())
        {
            // No more than 50 requests in flight at a time
            if self.current_requests.len() > 50 {
                break;
            }

            // See if we already have a thumbnail loaded for the asset
            // if thumbnail_state.image.is_some() {
            //     continue;
            // }

            let asset_id = *asset_id;

            // See if is already queued to load
            if thumbnail_state.queued_request_input_hash.is_some() {
                continue;
            }

            if thumbnail_state.failed_to_load {
                continue;
            }

            if let Some(last_staleness_check) = thumbnail_state.last_staleness_check {
                if (now - last_staleness_check).as_millis() > STALENESS_CHECK_TIME_MILLISECONDS {
                    continue;
                }
            }

            // Try to find a registered provider
            let Some(asset_schema) = data_set.asset_schema(asset_id) else {
                thumbnail_state.failed_to_load = true;
                continue;
            };

            let Some(provider) = self
                .thumbnail_provider_registry
                .provider_for_asset(asset_schema.fingerprint())
            else {
                thumbnail_state.image = Some(self.default_image.clone());
                continue;
            };

            // Calculate the current input hash
            let dependencies = provider
                .gather_inner(asset_id, data_set, schema_set)
                .unwrap();
            if self
                .current_requests
                .contains(&dependencies.thumbnail_input_hash)
            {
                continue;
            }

            // Check if the image we loaded is stale
            if thumbnail_state.current_input_hash == Some(dependencies.thumbnail_input_hash) {
                continue;
            }

            // Kick off the request
            self.current_requests
                .insert(dependencies.thumbnail_input_hash);
            thumbnail_state.queued_request_input_hash = Some(dependencies.thumbnail_input_hash);
            self.thread_pool
                .as_ref()
                .unwrap()
                .add_request(ThumbnailThreadPoolRequest::RunJob(
                    ThumbnailThreadPoolRequestRunJob {
                        asset_id,
                        asset_type: asset_schema.fingerprint(),
                        dependencies: Arc::new(dependencies),
                    },
                ));
        }

        while let Ok(result) = self.thread_pool_result_rx.try_recv() {
            match result {
                ThumbnailThreadPoolOutcome::RunJobComplete(msg) => {
                    self.current_requests
                        .remove(&msg.request.dependencies.thumbnail_input_hash);
                    if let Some(thumbnail_state) = state.cache.get_mut(&msg.request.asset_id, false)
                    {
                        match msg.result {
                            Ok(image) => {
                                thumbnail_state.queued_request_input_hash = None;
                                thumbnail_state.current_input_hash =
                                    Some(msg.request.dependencies.thumbnail_input_hash);
                                thumbnail_state.image = Some(Arc::new(image));
                            }
                            Err(e) => {
                                thumbnail_state.failed_to_load = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

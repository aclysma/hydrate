use std::sync::{Arc, Mutex};
use hydrate_base::AssetId;
use hydrate_base::hashing::HashMap;
use hydrate_base::lru_cache::LruCache;
use hydrate_data::{DataSet, SchemaSet};
use crate::thumbnails::ThumbnailProviderRegistry;

// Thumbnail providers are implemented per asset type
// - Implement a gather method that runs in main thread and can see asset data
//   This method will indicate all input data to produce the thumbnail.
// - Implement a render method that will run off-main-thread. It can read import data and only
//   data gathered from assets in gather().
// These providers are placed in a registry, this handles the type erasure for us to operate on them as a set
// The system will keep track of requested thumbnails, thumbnail state (not created, pending, created, etc.)
// and the registry
// Thumbnails need to be invalidated, we will use metadata returned from gather() to determine this

const THUMBNAIL_CACHE_SIZE: u32 = 64;

pub struct ThumbnailImage {
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u8>,
}

#[derive(Default)]
pub struct ThumbnailState {
    // List of asset dependencies
    // List of import data dependencies
    image: Option<Arc<ThumbnailImage>>,
}

struct ThumbnailSystemStateInner {
    cache: LruCache<AssetId, ThumbnailState>,
}

#[derive(Clone)]
pub struct ThumbnailSystemState {
    pub inner: Arc<Mutex<ThumbnailSystemStateInner>>
}

impl Default for ThumbnailSystemState {
    fn default() -> Self {
        ThumbnailSystemState {
            inner: Arc::new(Mutex::new(ThumbnailSystemStateInner {
                cache: LruCache::new(THUMBNAIL_CACHE_SIZE)
            }))
        }
    }
}

impl ThumbnailSystemState {
    pub fn request(&self, asset_id: AssetId) -> Option<Arc<ThumbnailImage>> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(thumbnail_state) = inner.cache.get(&asset_id) {
            thumbnail_state.image.clone()
        } else {
            inner.cache.insert(asset_id, ThumbnailState::default());
            None
        }
    }

    pub fn forget(&self, asset_id: AssetId) {

    }

    pub fn forget_all(&self) {

    }
}



pub struct ThumbnailSystem {
    // Thumbnails that have been requested, created, etc.
    thumbnail_system_state: ThumbnailSystemState,
    thumbnail_provider_registry: ThumbnailProviderRegistry,
    default_image: Arc<ThumbnailImage>,
}

impl ThumbnailSystem {
    pub fn system_state(&self) -> &ThumbnailSystemState {
        &self.thumbnail_system_state
    }

    pub fn new(thumbnail_provider_registry: ThumbnailProviderRegistry) -> Self {
        let default_image = ThumbnailImage {
            width: 1,
            height: 1,
            pixel_data: vec![0, 0, 0, 255]
        };

        ThumbnailSystem {
            thumbnail_system_state: ThumbnailSystemState::default(),
            thumbnail_provider_registry,
            default_image: Arc::new(default_image)
        }
    }

    // pub fn set_requested_thumbnails(
    //     &mut self,
    //     requested_thumbnails: Vec<AssetId>
    // ) {
    //     //self.requested_thumbnails = requested_thumbnails;
    // }

    pub fn update(
        &mut self,
        data_set: &DataSet,
        schema_set: &SchemaSet
    ) {
        let mut state = self.thumbnail_system_state.inner.lock().unwrap();
        for (asset_id, thumbnail_state) in state.cache.pairs_mut().iter_mut().filter_map(|x| x.as_mut()) {
            let asset_id = *asset_id;
            if thumbnail_state.image.is_none() {
                let asset_schema = data_set.asset_schema(asset_id).unwrap();
                if let Some(provider) = self.thumbnail_provider_registry.provider_for_asset(asset_schema.fingerprint()) {
                    let dependencies = provider.gather_inner(asset_id, data_set, schema_set).unwrap();
                    let image = provider.render_inner(asset_id, &*dependencies.gathered_data).unwrap();

                    thumbnail_state.image = Some(Arc::new(image));
                } else {
                    thumbnail_state.image = Some(self.default_image.clone());
                }
            }
        }

        // for &asset_id in &self.requested_thumbnails {
        //     let thumbnail_state = self.thumbnail_state.entry(asset_id).or_insert_with(|| ThumbnailState::default());
        //     if !thumbnail_state.has_been_loaded {
        //         let asset_schema = data_set.asset_schema(asset_id).unwrap();
        //         if let Some(provider) = self.thumbnail_provider_registry.provider_for_asset(asset_schema.fingerprint()) {
        //             let dependencies = provider.gather_inner(asset_id, data_set, schema_set).unwrap();
        //             let image = provider.render_inner(asset_id, &*dependencies.gathered_data).unwrap();
        //
        //             thumbnail_state.image = image;
        //             thumbnail_state.has_been_loaded = true;
        //         }
        //     }
        // }





        // Consider all thumbnails that are requested
        // - Has it been loaded at all?
        // - Is it stale?
        // - Gather data for some thumbnails
        // - Dispatch render jobs for some of them

        // Is now the time to add some kind of revision ID?


    }
}

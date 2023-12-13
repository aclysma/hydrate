use std::sync::{Arc, Mutex};
use egui::{ColorImage, Context, SizeHint, TextureHandle, TextureOptions};
use egui::load::{ImageLoader, ImageLoadResult, ImagePoll, LoadError, SizedTexture, TextureLoader, TextureLoadResult, TexturePoll};
use hydrate_model::AssetId;
use crate::lru_cache::LruCache;

const THUMBNAIL_URI_PREFIX: &str = "thumbnail://";
const THUMBNAIL_CACHE_SIZE: u32 = 64;

#[derive(PartialEq)]
enum LoadState {
    Requesting,
    Loaded(Arc<ColorImage>)
}

struct ThumbnailInfo {
    asset_id: AssetId,
    count: usize,
    load_state: LoadState,
}


pub struct AssetThumbnailImageLoader {
    dummy_image: Arc<ColorImage>,
    cache: Mutex<LruCache<AssetId, ThumbnailInfo>>
}

impl AssetThumbnailImageLoader {
    pub fn new() -> Self {
        let dummy_image = ColorImage::example();
        AssetThumbnailImageLoader {
            dummy_image: Arc::new(dummy_image),
            cache: Mutex::new(LruCache::new(THUMBNAIL_CACHE_SIZE))
        }
    }

    pub fn update(&self) {
        let mut cache = self.cache.lock().unwrap();
        for (asset_id, thumbnail_info) in cache.pairs_mut().iter_mut().filter_map(|x| x.as_mut()) {
            if thumbnail_info.load_state == LoadState::Requesting {
                thumbnail_info.count += 1;
                if thumbnail_info.count > 100 {
                    thumbnail_info.load_state = LoadState::Loaded(self.dummy_image.clone());
                }
            }
        }
    }
}

impl ImageLoader for AssetThumbnailImageLoader {
    fn id(&self) -> &str {
        "hydrate_editor::AssetThumbnailImageLoader"
    }

    fn load(&self, ctx: &Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if uri.starts_with(THUMBNAIL_URI_PREFIX) {
            let asset_id = AssetId::parse_str(&uri[THUMBNAIL_URI_PREFIX.len()..]).unwrap();
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached_entry) = cache.get(&asset_id) {
                match &cached_entry.load_state {
                    LoadState::Requesting => Ok(ImagePoll::Pending {
                        size: None,
                    }),
                    LoadState::Loaded(image) => Ok(ImagePoll::Ready {
                        image: image.clone()
                    })
                }
            } else {
                cache.insert(asset_id, ThumbnailInfo {
                    asset_id,
                    count: 0,
                    load_state: LoadState::Requesting,
                });
                Ok(ImagePoll::Pending {
                    size: None,
                })
            }

        } else {
            Err(LoadError::NotSupported)
        }

    }

    fn forget(&self, uri: &str) {
        if uri.starts_with(THUMBNAIL_URI_PREFIX) {
            let asset_id = AssetId::parse_str(&uri[THUMBNAIL_URI_PREFIX.len()..]).unwrap();
            let mut cache = self.cache.lock().unwrap();
            cache.remove(&asset_id);
        }
    }

    fn forget_all(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = LruCache::new(THUMBNAIL_CACHE_SIZE)
    }

    fn byte_size(&self) -> usize {
        //TODO: Implement this
        0
    }
}




pub struct AssetThumbnailTextureLoader {
    cache: Mutex<LruCache<(String, TextureOptions), TextureHandle>>,
}

impl AssetThumbnailTextureLoader {
    pub fn new() -> Self {
        AssetThumbnailTextureLoader {
            cache: Mutex::new(LruCache::new(THUMBNAIL_CACHE_SIZE))
        }
    }
}

impl TextureLoader for AssetThumbnailTextureLoader {
    fn id(&self) -> &str {
        ""
    }

    fn load(
        &self,
        ctx: &Context,
        uri: &str,
        texture_options: TextureOptions,
        size_hint: SizeHint,
    ) -> TextureLoadResult {
        let mut cache = self.cache.lock().unwrap();
        if let Some(handle) = cache.get(&(uri.into(), texture_options)) {
            let texture = SizedTexture::from_handle(handle);
            Ok(TexturePoll::Ready { texture })
        } else {
            match ctx.try_load_image(uri, size_hint)? {
                ImagePoll::Pending { size } => Ok(TexturePoll::Pending { size }),
                ImagePoll::Ready { image } => {
                    let handle = ctx.load_texture(uri, image, texture_options);
                    let texture = SizedTexture::from_handle(&handle);
                    cache.insert((uri.into(), texture_options), handle);
                    Ok(TexturePoll::Ready { texture })
                }
            }
        }
    }

    fn forget(&self, uri: &str) {
        let mut pending_remove = Vec::default();

        let mut cache = self.cache.lock().unwrap();
        for (asset_id, thumbnail_info) in cache.pairs_mut().iter_mut().filter_map(|x| x.as_mut()) {
            if asset_id.0 == uri {
                pending_remove.push(asset_id.clone());
            }
        }

        for key in pending_remove {
            cache.remove(&key);
        }
    }

    fn forget_all(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = LruCache::new(THUMBNAIL_CACHE_SIZE)
    }

    fn end_frame(&self, _: usize) {}

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .unwrap()
            .pairs()
            .iter()
            .filter_map(|x| x.as_ref())
            .map(|(k, v)| v.byte_size())
            .sum()
    }
}

use eframe::epaint::Color32;
use egui::load::{
    ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizedTexture, TextureLoadResult,
    TextureLoader, TexturePoll,
};
use egui::ImageData::Color;
use egui::{ColorImage, Context, SizeHint, TextureHandle, TextureOptions};
use hydrate_base::lru_cache::LruCache;
use hydrate_model::edit_context::EditContext;
use hydrate_model::pipeline::{
    AssetEngine, ThumbnailImage, ThumbnailProviderRegistry, ThumbnailSystemState,
};
use hydrate_model::{AssetId, HashMap, SchemaFingerprint, SchemaSet};
use hydrate_pipeline::ThumbnailInputHash;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const THUMBNAIL_ASSET_URI_PREFIX: &str = "thumbnail-asset://";
const THUMBNAIL_ASSET_TYPE_URI_PREFIX: &str = "thumbnail-asset-type://";
const THUMBNAIL_SPECIAL_PREFIX: &str = "thumbnail-special://";
const THUMBNAIL_URI_NO_THUMBNAIL: &str = "thumbnail-special://no-thumbnail";
const THUMBNAIL_URI_NO_REFERENCE: &str = "thumbnail-special://no-reference";
const THUMBNAIL_CACHE_SIZE: u32 = 64;

#[derive(PartialEq)]
enum LoadState {
    Requesting,
    Loaded(Arc<ColorImage>),
}

struct CachedThumbnail {
    thumbnail_input_hash: ThumbnailInputHash,
    color_image: Arc<ColorImage>,
}

pub struct ThumbnailImageLoader {
    dummy_image: Arc<ColorImage>,
    thumbnail_cache: Mutex<LruCache<AssetId, CachedThumbnail>>,
    thumbnail_system_state: ThumbnailSystemState,
    thumbnail_provider_registry: ThumbnailProviderRegistry,
    default_thumbnails: HashMap<SchemaFingerprint, Arc<ColorImage>>,
    schema_set: SchemaSet,
    special_thumbnail_no_thumbnail: Arc<ColorImage>,
    special_thumbnail_no_reference: Arc<ColorImage>,
}

impl ThumbnailImageLoader {
    pub fn new(
        schema_set: &SchemaSet,
        thumbnail_provider_registry: &ThumbnailProviderRegistry,
        thumbnail_system_state: &ThumbnailSystemState,
    ) -> Self {
        let dummy_image = ColorImage::example();
        let mut loaded_images = HashMap::<PathBuf, Arc<ColorImage>>::default();
        let mut default_thumbnails = HashMap::default();

        let no_reference_image = image::load_from_memory_with_format(
            include_bytes!("../thumbnails/no-reference.png"),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8();
        let no_reference_image_size = [
            no_reference_image.width() as usize,
            no_reference_image.height() as usize,
        ];
        let no_reference = ColorImage::from_rgba_unmultiplied(
            no_reference_image_size,
            no_reference_image.as_raw(),
        );

        let no_thumbnail_image = image::load_from_memory_with_format(
            include_bytes!("../thumbnails/no-thumbnail.png"),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8();
        let no_thumbnail_image_size = [
            no_thumbnail_image.width() as usize,
            no_thumbnail_image.height() as usize,
        ];
        let no_thumbnail = ColorImage::from_rgba_unmultiplied(
            no_thumbnail_image_size,
            no_thumbnail_image.as_raw(),
        );

        for (k, v) in schema_set.schemas() {
            if let Some(record) = v.try_as_record() {
                if let Some(path) = &record.markup().default_thumbnail {
                    if let Some(loaded_image) = loaded_images.get(path) {
                        default_thumbnails.insert(*k, loaded_image.clone());
                    } else {
                        println!("open path {:?}", path);
                        let image = image::open(path).unwrap().into_rgba8();
                        let image = Arc::new(ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            image.as_raw(),
                        ));
                        loaded_images.insert(path.clone(), image.clone());
                        default_thumbnails.insert(*k, image);
                    }
                }
            }
        }

        ThumbnailImageLoader {
            schema_set: schema_set.clone(),
            dummy_image: Arc::new(dummy_image),
            thumbnail_cache: Mutex::new(LruCache::new(THUMBNAIL_CACHE_SIZE)),
            thumbnail_system_state: thumbnail_system_state.clone(),
            thumbnail_provider_registry: thumbnail_provider_registry.clone(),
            default_thumbnails,
            special_thumbnail_no_thumbnail: Arc::new(no_thumbnail),
            special_thumbnail_no_reference: Arc::new(no_reference),
        }
    }

    pub fn check_for_stale_thumbnails(
        &self,
        ctx: &egui::Context,
    ) {
        let refreshed_thumbnails = self.thumbnail_system_state.take_refreshed_thumbnails();
        for refreshed_thumbnail in refreshed_thumbnails {
            ctx.forget_image(&format!(
                "thumbnail-asset://{}",
                refreshed_thumbnail.as_uuid().to_string()
            ));
        }
    }

    pub fn thumbnail_uri_for_asset_with_fingerprint(
        &self,
        asset_id: AssetId,
        schema_fingerprint: SchemaFingerprint,
    ) -> String {
        if asset_id.is_null() {
            THUMBNAIL_URI_NO_REFERENCE.to_string()
        } else if self
            .thumbnail_provider_registry
            .has_provider_for_asset(schema_fingerprint)
        {
            format!("thumbnail-asset://{}", asset_id.as_uuid().to_string())
        } else if self.default_thumbnails.contains_key(&schema_fingerprint) {
            format!(
                "thumbnail-asset-type://{}",
                schema_fingerprint.as_uuid().to_string()
            )
        } else {
            THUMBNAIL_URI_NO_THUMBNAIL.to_string()
        }
    }

    pub fn thumbnail_uri_for_asset(
        &self,
        edit_context: &EditContext,
        asset_id: AssetId,
    ) -> String {
        if asset_id.is_null() {
            return THUMBNAIL_URI_NO_REFERENCE.to_string();
        };

        let schema_record = edit_context.asset_schema(asset_id);
        if let Some(schema_record) = schema_record {
            let schema_fingerprint = schema_record.fingerprint();
            if self
                .thumbnail_provider_registry
                .has_provider_for_asset(schema_fingerprint)
            {
                return format!("thumbnail-asset://{}", asset_id.as_uuid().to_string());
            } else if self.default_thumbnails.contains_key(&schema_fingerprint) {
                return format!(
                    "thumbnail-asset-type://{}",
                    schema_fingerprint.as_uuid().to_string()
                );
            }
        }

        THUMBNAIL_URI_NO_THUMBNAIL.to_string()
    }
}

impl ImageLoader for ThumbnailImageLoader {
    fn id(&self) -> &str {
        "hydrate_editor::AssetThumbnailImageLoader"
    }

    fn load(
        &self,
        ctx: &Context,
        uri: &str,
        size_hint: SizeHint,
    ) -> ImageLoadResult {
        if uri == THUMBNAIL_URI_NO_THUMBNAIL {
            Ok(ImagePoll::Ready {
                image: self.special_thumbnail_no_thumbnail.clone(),
            })
        } else if uri == THUMBNAIL_URI_NO_REFERENCE {
            Ok(ImagePoll::Ready {
                image: self.special_thumbnail_no_reference.clone(),
            })
        } else if uri.starts_with(THUMBNAIL_ASSET_TYPE_URI_PREFIX) {
            let schema_fingerprint = SchemaFingerprint::from_uuid(
                Uuid::parse_str(&uri[THUMBNAIL_ASSET_TYPE_URI_PREFIX.len()..]).unwrap(),
            );
            if let Some(default_thumbnail) = self.default_thumbnails.get(&schema_fingerprint) {
                Ok(ImagePoll::Ready {
                    image: default_thumbnail.clone(),
                })
            } else {
                Ok(ImagePoll::Ready {
                    image: self.dummy_image.clone(),
                })
            }
        } else if uri.starts_with(THUMBNAIL_ASSET_URI_PREFIX) {
            let asset_id = AssetId::parse_str(&uri[THUMBNAIL_ASSET_URI_PREFIX.len()..]).unwrap();
            let mut cache = self.thumbnail_cache.lock().unwrap();
            if let Some(image) = cache.get(&asset_id, true) {
                Ok(ImagePoll::Ready {
                    image: image.color_image.clone(),
                })
            } else if let Some(cached_entry) = self.thumbnail_system_state.request(asset_id) {
                let image = Arc::new(ColorImage::from_rgba_unmultiplied(
                    [
                        cached_entry.image.width as usize,
                        cached_entry.image.height as usize,
                    ],
                    &cached_entry.image.pixel_data,
                ));

                cache.insert(
                    asset_id,
                    CachedThumbnail {
                        thumbnail_input_hash: cached_entry.hash,
                        color_image: image.clone(),
                    },
                );

                Ok(ImagePoll::Ready { image })
            } else {
                Ok(ImagePoll::Pending { size: None })
            }
        } else {
            Err(LoadError::NotSupported)
        }
    }

    fn forget(
        &self,
        uri: &str,
    ) {
        if uri.starts_with(THUMBNAIL_ASSET_URI_PREFIX) {
            let asset_id = AssetId::parse_str(&uri[THUMBNAIL_ASSET_URI_PREFIX.len()..]).unwrap();
            self.thumbnail_system_state.forget(asset_id);
            let mut cache = self.thumbnail_cache.lock().unwrap();
            cache.remove(&asset_id);
        }
    }

    fn forget_all(&self) {
        self.thumbnail_system_state.forget_all();
        let mut cache = self.thumbnail_cache.lock().unwrap();
        *cache = LruCache::new(THUMBNAIL_CACHE_SIZE);
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
            cache: Mutex::new(LruCache::new(THUMBNAIL_CACHE_SIZE)),
        }
    }
}

impl TextureLoader for AssetThumbnailTextureLoader {
    fn id(&self) -> &str {
        "hydrate_editor::AssetThumbnailTextureLoader"
    }

    fn load(
        &self,
        ctx: &Context,
        uri: &str,
        texture_options: TextureOptions,
        size_hint: SizeHint,
    ) -> TextureLoadResult {
        let mut cache = self.cache.lock().unwrap();
        if let Some(handle) = cache.get(&(uri.into(), texture_options), true) {
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

    fn forget(
        &self,
        uri: &str,
    ) {
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

    fn end_frame(
        &self,
        _: usize,
    ) {
    }

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

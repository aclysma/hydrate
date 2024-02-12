use crate::thumbnails::{
    ThumbnailProvider, ThumbnailProviderAbstract, ThumbnailProviderId, ThumbnailProviderWrapper,
};
use crate::ThumbnailImage;
use hydrate_base::hashing::HashMap;
use hydrate_data::SchemaSet;
use hydrate_schema::SchemaFingerprint;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Default)]
pub struct ThumbnailProviderRegistryBuilder {
    thumbnail_providers: Vec<Arc<dyn ThumbnailProviderAbstract>>,
    default_thumbnails: HashMap<String, Arc<ThumbnailImage>>,
}

impl ThumbnailProviderRegistryBuilder {
    pub fn register_thumbnail_provider<T: ThumbnailProvider + Send + Sync + Default + 'static>(
        &mut self
    ) where
        T::GatheredDataT: Hash + for<'a> serde::Deserialize<'a> + serde::Serialize,
    {
        self.thumbnail_providers
            .push(Arc::new(ThumbnailProviderWrapper(T::default())));
    }

    pub fn register_thumbnail_provider_instance<T: ThumbnailProvider + Send + Sync + 'static>(
        &mut self,
        thumbnail_provider: T,
    ) where
        T::GatheredDataT: Hash + for<'a> serde::Deserialize<'a> + serde::Serialize,
    {
        self.thumbnail_providers
            .push(Arc::new(ThumbnailProviderWrapper(thumbnail_provider)));
    }

    pub fn build(
        self,
        schema_set: &SchemaSet,
    ) -> ThumbnailProviderRegistry {
        let mut asset_type_to_provider = HashMap::default();

        for (provider_index, provider) in self.thumbnail_providers.iter().enumerate() {
            let provider_id = ThumbnailProviderId(provider_index);
            let asset_type = schema_set
                .find_named_type(provider.asset_type_inner())
                .unwrap()
                .fingerprint();
            let insert_result = asset_type_to_provider.insert(asset_type, provider_id);
            // println!(
            //     "provider {} handles asset fingerprint {}",
            //     provider_id.0,
            //     asset_type.as_uuid()
            // );
            if insert_result.is_some() {
                panic!(
                    "Multiple handlers registered to handle the same asset {}",
                    provider.asset_type_inner()
                );
            }
        }

        let mut default_thumbnails = HashMap::default();
        for (k, v) in self.default_thumbnails {
            let named_type = schema_set.find_named_type(k).unwrap().fingerprint();
            default_thumbnails.insert(named_type, v);
        }

        let inner = ThumbnailProviderRegistryInner {
            asset_type_to_provider,
            thumbnail_providers: self.thumbnail_providers,
        };

        ThumbnailProviderRegistry {
            inner: Arc::new(inner),
        }
    }
}

pub struct ThumbnailProviderRegistryInner {
    thumbnail_providers: Vec<Arc<dyn ThumbnailProviderAbstract>>,
    asset_type_to_provider: HashMap<SchemaFingerprint, ThumbnailProviderId>,
}

#[derive(Clone)]
pub struct ThumbnailProviderRegistry {
    inner: Arc<ThumbnailProviderRegistryInner>,
}

impl ThumbnailProviderRegistry {
    pub fn has_provider_for_asset(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> bool {
        self.inner.asset_type_to_provider.contains_key(&fingerprint)
    }

    pub fn provider_for_asset(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&Arc<dyn ThumbnailProviderAbstract>> {
        self.inner
            .asset_type_to_provider
            .get(&fingerprint)
            .copied()
            .map(|x| &self.inner.thumbnail_providers[x.0])
    }
}

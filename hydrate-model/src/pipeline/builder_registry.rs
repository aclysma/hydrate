use crate::{BuilderId, HashMap, SchemaFingerprint, SchemaSet};
use std::sync::Arc;

use super::build_types::*;

// Keeps track of all known builders
pub struct BuilderRegistryInner {
    registered_builders: Vec<Box<dyn Builder>>,
    asset_type_to_builder: HashMap<SchemaFingerprint, BuilderId>,
}

#[derive(Clone)]
pub struct BuilderRegistry {
    inner: Arc<BuilderRegistryInner>
}

impl BuilderRegistry {
    pub fn builder_for_asset(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&Box<dyn Builder>> {
        self.inner.asset_type_to_builder
            .get(&fingerprint)
            .copied()
            .map(|x| &self.inner.registered_builders[x.0])
    }
}

// Keeps track of all known builders
#[derive(Default)]
pub struct BuilderRegistryBuilder {
    registered_builders: Vec<Box<dyn Builder>>,
}

impl BuilderRegistryBuilder {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: Builder + Default + 'static>(
        &mut self,
    ) {
        let handler = Box::new(T::default());
        self.registered_builders.push(handler);
    }

    pub fn register_handler_instance<T: Builder + 'static>(
        &mut self,
        instance: T,
    ) {
        let handler = Box::new(instance);
        self.registered_builders.push(handler);
    }

    //
    // Called after finished linking the schema so we can associate schema fingerprints with handlers
    //
    pub fn build(
        self,
        schema_set: &SchemaSet,
    ) -> BuilderRegistry {
        let mut asset_type_to_builder = HashMap::default();

        for (builder_index, builder) in self.registered_builders.iter().enumerate() {
            let builder_id = BuilderId(builder_index);
            let asset_type = schema_set
                .find_named_type(builder.asset_type())
                .unwrap()
                .fingerprint();
            let insert_result = asset_type_to_builder.insert(asset_type, builder_id);
            // println!(
            //     "builder {} handles asset fingerprint {}",
            //     builder_id.0,
            //     asset_type.as_uuid()
            // );
            if insert_result.is_some() {
                panic!("Multiple handlers registered to handle the same asset")
            }
        }

        let inner = BuilderRegistryInner {
            registered_builders: self.registered_builders,
            asset_type_to_builder,
        };

        BuilderRegistry {
            inner: Arc::new(inner)
        }
    }
}

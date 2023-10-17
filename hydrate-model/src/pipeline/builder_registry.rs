use crate::{BuildInfo, BuilderId, DataSet, DataSource, EditorModel, HashMap, HashMapKeys, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint, SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use hydrate_base::{BuiltObjectMetadata, AssetUuid};
use hydrate_base::handle::DummySerdeContextHandle;

use super::ImportJobs;

use hydrate_base::uuid_path::{path_to_uuid, uuid_and_hash_to_path, uuid_to_path};

use super::build_types::*;

// Keeps track of all known builders
pub struct BuilderRegistryInner {
    registered_builders: Vec<Box<dyn Builder>>,
    //file_extension_associations: HashMap<String, Vec<BuilderId>>,
    asset_type_to_builder: HashMap<SchemaFingerprint, BuilderId>,
}

#[derive(Clone)]
pub struct BuilderRegistry {
    inner: Arc<BuilderRegistryInner>
}

impl BuilderRegistry {
    // pub fn importers_for_file_extension(&self, extension: &str) -> &[BuilderId] {
    //     const EMPTY_LIST: &'static [BuilderId] = &[];
    //     self.file_extension_associations.get(extension).map(|x| x.as_slice()).unwrap_or(EMPTY_LIST)
    // }

    pub fn builder_for_asset(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&Box<dyn Builder>> {
        // if let Some(builder_id) = self.asset_type_to_builder.get(&fingerprint).copied() {
        //     Some(&self.registered_builders[builder_id.0])
        // } else {
        //     None
        // }
        self.inner.asset_type_to_builder
            .get(&fingerprint)
            .copied()
            .map(|x| &self.inner.registered_builders[x.0])
    }

    // pub fn builder(&self, builder_id: BuilderId) -> Option<&Box<Builder>> {
    //     self.registered_builders.get(&builder_id)
    // }
}

// Keeps track of all known builders
#[derive(Default)]
pub struct BuilderRegistryBuilder {
    registered_builders: Vec<Box<dyn Builder>>,
    //file_extension_associations: HashMap<String, Vec<BuilderId>>,
    asset_type_to_builder: HashMap<SchemaFingerprint, BuilderId>,
}

impl BuilderRegistryBuilder {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: Builder + Default + 'static>(
        &mut self,
        linker: &mut SchemaLinker,
    ) {
        let handler = Box::new(T::default());
        self.registered_builders.push(handler);
    }

    pub fn register_handler_instance<T: Builder + 'static>(
        &mut self,
        linker: &mut SchemaLinker,
        instance: T,
    ) {
        let handler = Box::new(instance);
        self.registered_builders.push(handler);
    }

    //
    // Called after finished linking the schema so we can associate schema fingerprints with handlers
    //
    pub fn finished_linking(
        &mut self,
        schema_set: &SchemaSet,
    ) {
        let mut asset_type_to_builder = HashMap::default();

        for (builder_index, builder) in self.registered_builders.iter().enumerate() {
            let builder_id = BuilderId(builder_index);
            let asset_type = schema_set
                .find_named_type(builder.asset_type())
                .unwrap()
                .fingerprint();
            let insert_result = asset_type_to_builder.insert(asset_type, builder_id);
            println!(
                "builder {} handles asset fingerprint {}",
                builder_id.0,
                asset_type.as_uuid()
            );
            if insert_result.is_some() {
                panic!("Multiple handlers registered to handle the same asset")
            }
        }

        self.asset_type_to_builder = asset_type_to_builder;
    }

    pub fn build(self) -> BuilderRegistry {
        let inner = BuilderRegistryInner {
            registered_builders: self.registered_builders,
            asset_type_to_builder: self.asset_type_to_builder,
        };

        BuilderRegistry {
            inner: Arc::new(inner)
        }
    }
}

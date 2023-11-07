use crate::{
    HashMap,
    ImporterId,
};
use std::sync::Arc;
use type_uuid::TypeUuid;
use uuid::Uuid;

use super::import_types::*;

// Keeps track of all known importers
pub struct ImporterRegistryInner {
    registered_importers: HashMap<ImporterId, Box<dyn Importer>>,
    file_extension_associations: HashMap<String, Vec<ImporterId>>,
}

#[derive(Clone)]
pub struct ImporterRegistry {
    inner: Arc<ImporterRegistryInner>
}

impl ImporterRegistry {
    pub fn importers_for_file_extension(
        &self,
        extension: &str,
    ) -> &[ImporterId] {
        const EMPTY_LIST: &'static [ImporterId] = &[];
        self.inner.file_extension_associations
            .get(extension)
            .map(|x| x.as_slice())
            .unwrap_or(EMPTY_LIST)
    }

    pub fn importer(
        &self,
        importer_id: ImporterId,
    ) -> Option<&Box<dyn Importer>> {
        self.inner.registered_importers.get(&importer_id)
    }
}

#[derive(Default)]
pub struct ImporterRegistryBuilder {
    registered_importers: HashMap<ImporterId, Box<dyn Importer>>,
    file_extension_associations: HashMap<String, Vec<ImporterId>>,
}

impl ImporterRegistryBuilder {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: TypeUuid + Importer + Default + 'static>(
        &mut self,
    ) {
        self.register_handler_instance(T::default())
    }

    pub fn register_handler_instance<T: TypeUuid + Importer + 'static>(
        &mut self,
        importer: T
    ) {
        let handler = Box::new(importer);
        let importer_id = ImporterId(Uuid::from_bytes(T::UUID));
        self.registered_importers.insert(importer_id, handler);

        for extension in self.registered_importers[&importer_id].supported_file_extensions() {
            self.file_extension_associations
                .entry(extension.to_string())
                .or_default()
                .push(importer_id);
        }
    }

    pub fn build(self) -> ImporterRegistry {
        let inner = ImporterRegistryInner {
            registered_importers: self.registered_importers,
            file_extension_associations: self.file_extension_associations,
        };

        ImporterRegistry {
            inner: Arc::new(inner)
        }
    }
}
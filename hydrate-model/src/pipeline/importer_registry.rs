use crate::{
    DataSet, DataSource, EditorModel, HashMap, HashMapKeys, ImportInfo,
    ImporterId, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint,
    SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use type_uuid::{TypeUuid, TypeUuidDynamic};
use uuid::Uuid;

use crate::edit_context::EditContext;
use crate::SingleObjectJson;
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};

use super::import_types::*;

// Keeps track of all known importers
#[derive(Default)]
pub struct ImporterRegistry {
    registered_importers: HashMap<ImporterId, Box<dyn Importer>>,
    file_extension_associations: HashMap<String, Vec<ImporterId>>,
    //asset_to_importer: HashMap<SchemaFingerprint, ImporterId>,
}

impl ImporterRegistry {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: TypeUuid + Importer + Default + 'static>(
        &mut self,
        linker: &mut SchemaLinker,
    ) {
        let handler = Box::new(T::default());
        //handler.register_schemas(linker);
        let importer_id = ImporterId(Uuid::from_bytes(T::UUID));
        self.registered_importers.insert(importer_id, handler);

        for extension in self.registered_importers[&importer_id].supported_file_extensions() {
            self.file_extension_associations
                .entry(extension.to_string())
                .or_default()
                .push(importer_id);
        }
    }

    //
    // Called after finished linking the schema so we can associate schema fingerprints with handlers
    //
    pub fn finished_linking(
        &mut self,
        schema_set: &SchemaSet,
    ) {
        // let mut asset_to_importer = HashMap::default();
        //
        // for (importer_id, importer) in &self.registered_importers {
        //     // for asset_type in importer.asset_types() {
        //     //     let asset_type = schema_set.find_named_type(asset_type).unwrap().fingerprint();
        //     //     let insert_result = asset_to_importer.insert(asset_type, *importer_id);
        //     //     if insert_result.is_some() {
        //     //         panic!("Multiple handlers registered to handle the same asset")
        //     //     }
        //     // }
        // }

        //self.asset_to_importer = asset_to_importer;
    }

    pub fn importers_for_file_extension(
        &self,
        extension: &str,
    ) -> &[ImporterId] {
        const EMPTY_LIST: &'static [ImporterId] = &[];
        self.file_extension_associations
            .get(extension)
            .map(|x| x.as_slice())
            .unwrap_or(EMPTY_LIST)
    }

    // pub fn handler_for_asset(&self, fingerprint: SchemaFingerprint) -> Option<ImporterId> {
    //     self.asset_to_importer.get(&fingerprint).copied()
    // }

    pub fn importer(
        &self,
        importer_id: ImporterId,
    ) -> Option<&Box<dyn Importer>> {
        self.registered_importers.get(&importer_id)
    }
}
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
use hydrate_base::hashing::HashSet;

use crate::edit_context::EditContext;
use crate::SingleObjectJson;
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};

use super::import_types::*;
use super::importer_registry::*;

fn hash_file_metadata(metadata: &std::fs::Metadata) -> u64 {
    let mut hasher = siphasher::sip::SipHasher::default();
    metadata.modified().unwrap().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    hasher.finish()
}

pub struct ImportData {
    pub import_data: SingleObject,
    pub metadata_hash: u64,
}

// An in-flight import operation we want to perform
struct ImportOp {
    object_ids: HashMap<Option<String>, ObjectId>,
    importer_id: ImporterId,
    path: PathBuf,
    assets_to_regenerate: HashSet<ObjectId>,
    //pub(crate) import_info: ImportInfo,
}

// A known import job, each existing asset that imports data will have an associated import job.
// It could be in a completed state, or there could be a problem with it and we need to re-run it.
struct ImportJob {
    object_id: ObjectId,
    import_data_exists: bool,
    asset_exists: bool,
    imported_data_stale: bool, // how to know it's stale? (we need timestamp/filesize stored along with import data, and paths to file it included) We may not know until we try to open it
    imported_data_invalid: bool, // how to know it's valid? (does it parse? does it have errors? we may not know until we try to open it)
    imported_data_hash: Option<u64>,
}

impl ImportJob {
    pub fn new(object_id: ObjectId) -> Self {
        ImportJob {
            object_id,
            import_data_exists: false,
            asset_exists: false,
            imported_data_stale: false,
            imported_data_invalid: false,
            imported_data_hash: None,
        }
    }
}

// Cache of all known import jobs. This includes imports that are complete, in progress, or not started.
// We find these by scanning existing assets and import data. We also inspect the asset and imported
// data to see if the job is complete, or is in a failed or stale state.
pub struct ImportJobs {
    //import_editor_model: EditorModel
    root_path: PathBuf,
    import_jobs: HashMap<ObjectId, ImportJob>,
    import_operations: Vec<ImportOp>,
}

impl ImportJobs {
    pub fn new(
        importer_registry: &ImporterRegistry,
        editor_model: &EditorModel,
        root_path: PathBuf,
    ) -> Self {
        let import_jobs = ImportJobs::find_all_jobs(importer_registry, editor_model, &root_path);

        ImportJobs {
            root_path,
            import_jobs,
            import_operations: Default::default(),
        }
    }

    pub fn queue_import_operation(
        &mut self,
        object_ids: HashMap<Option<String>, ObjectId>,
        importer_id: ImporterId,
        path: PathBuf,
        assets_to_regenerate: HashSet<ObjectId>,
    ) {
        self.import_operations.push(ImportOp {
            object_ids,
            importer_id,
            path,
            assets_to_regenerate,
            //import_info
        })
    }

    pub fn load_import_data(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
    ) -> ImportData {
        let path = uuid_to_path(&self.root_path, object_id.as_uuid(), "if");
        println!("LOAD DATA PATH {:?}", path);
        let str = std::fs::read_to_string(&path).unwrap();
        let metadata = path.metadata().unwrap();
        let metadata_hash = hash_file_metadata(&metadata);
        let import_data = SingleObjectJson::load_single_object_from_string(schema_set, &str);
        ImportData {
            import_data,
            metadata_hash,
        }
    }

    pub fn clone_import_data_metadata_hashes(&self) -> HashMap<ObjectId, u64> {
        let mut metadata_hashes = HashMap::default();
        for (k, v) in &self.import_jobs {
            if let Some(imported_data_hash) = v.imported_data_hash {
                metadata_hashes.insert(*k, imported_data_hash);
            }
        }

        metadata_hashes
    }

    // pub fn handle_file_updates(&mut self, file_updates: &[PathBuf]) {
    //     for file_update in file_updates {
    //         if let Ok(relative) = file_update.strip_prefix(&self.root_path) {
    //             if let Some(uuid) = path_to_uuid(&self.root_path, file_update) {
    //                 let object_id = ObjectId(uuid.as_u128());
    //
    //             }
    //         }
    //     }
    // }

    pub fn update(
        &mut self,
        importer_registry: &ImporterRegistry,
        editor_model: &mut EditorModel,
    ) {
        let schema_set = editor_model.clone_schema_set();
        for import_op in &self.import_operations {
            //let importer_id = editor_model.root_edit_context().import_info()
            let importer_id = import_op.importer_id;
            //let fingerprint = editor_model.root_edit_context().object_schema(import_op.import_info).unwrap().fingerprint();
            //let importer_id = importer_registry.asset_to_importer.get(&fingerprint).unwrap();
            let importer = importer_registry.importer(importer_id).unwrap();

            let mut importable_objects = HashMap::<Option<String>, ImportableObject>::default();
            for (name, object_id) in &import_op.object_ids {
                let referenced_paths = editor_model.root_edit_context().resolve_all_file_references(*object_id).unwrap_or_default();
                importable_objects.insert(name.clone(), ImportableObject {
                    id: *object_id,
                    referenced_paths
                });
            }

            let imported_objects = importer.import_file(
                &import_op.path,
                &importable_objects,
                editor_model.schema_set(),
            );

            //TODO: Validate that all requested importables exist?
            for (name, imported_object) in imported_objects {
                if let Some(object_id) = import_op.object_ids.get(&name) {
                    if import_op.assets_to_regenerate.contains(object_id) {
                        if let Some(default_asset) = &imported_object.default_asset {
                            editor_model.root_edit_context_mut().copy_from_single_object(&*schema_set, *object_id, default_asset).unwrap();
                        }
                    }

                    if let Some(import_data) = &imported_object.import_data {
                        let data =
                            SingleObjectJson::save_single_object_to_string(import_data);
                        let path = uuid_to_path(&self.root_path, object_id.as_uuid(), "if");

                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent).unwrap();
                        }

                        std::fs::write(&path, data).unwrap();
                        let metadata = path.metadata().unwrap();
                        let metadata_hash = hash_file_metadata(&metadata);
                        let mut import_job = self
                            .import_jobs
                            .entry(*object_id)
                            .or_insert_with(|| ImportJob::new(*object_id));
                        import_job.import_data_exists = true;
                        import_job.imported_data_hash = Some(metadata_hash);
                    }
                }
            }
        }

        self.import_operations.clear();

        // Send/mark for processing?
    }

    fn find_all_jobs(
        importer_registry: &ImporterRegistry,
        editor_model: &EditorModel,
        root_path: &Path,
    ) -> HashMap<ObjectId, ImportJob> {
        let mut import_jobs = HashMap::<ObjectId, ImportJob>::default();

        //
        // Scan import dir for known import data
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.if"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                println!("dir file {:?}", file);
                let dir_uuid = path_to_uuid(root_path, file.path()).unwrap();
                let object_id = ObjectId(dir_uuid.as_u128());
                let job = import_jobs
                    .entry(object_id)
                    .or_insert_with(|| ImportJob::new(object_id));

                let file_metadata = file.metadata().unwrap();
                let import_data_hash = hash_file_metadata(&file_metadata);

                job.import_data_exists = true;
                job.imported_data_hash = Some(import_data_hash);
            }
        }

        //
        // Scan assets to find any asset that has an associated importer
        //
        let data_set = editor_model.root_edit_context().data_set();
        for object_id in data_set.all_objects() {
            if let Some(import_info) = data_set.import_info(*object_id) {
                let importer_id = import_info.importer_id();
                let importer = importer_registry.importer(importer_id);
                if importer.is_some() {
                    let job = import_jobs
                        .entry(*object_id)
                        .or_insert_with(|| ImportJob::new(*object_id));
                    job.asset_exists = true;
                }
            }

            //let schema_fingerprint = data_set.object_schema(*object_id).unwrap().fingerprint();
            // let importer = importer_registry.handler_for_asset(schema_fingerprint);
            // if importer.is_some() {
            //     let job = import_jobs.entry(*object_id).or_insert_with(|| ImportJob::new(*object_id));
            //     job.asset_exists = true;
            // }
        }

        import_jobs

        // for (object_id, job) in import_jobs {
        //     if job.asset_exists && !job.import_data_exists {
        //         // We need to re-import the data
        //     }
        //
        //     if !job.asset_exists && job.import_data_exists {
        //         // We need to delete the import data that no longer has an associated asset
        //     }
        //
        //     if job.asset_exists && job.import_data_exists {
        //         // We may want to validate the import data and check that it is not stale
        //     }
        // }
    }
}


// struct ScanContext {
//     referenced_files: Vec<PathBuf>,
//     referenced_assets: Vec<ObjectId>,
// }
//
// impl ScanContext {
//     // Will read the file, and if we are live-reloading changes, trigger a re-import if the file changes
//     // This is used when the file is not referenced by another asset, or there is no desire to
//     // import it once and have several assets share it
//     pub fn read(path: &Path) -> Vec<u8> {
//         unimplemented!();
//     }
//
//     pub fn read_to_string(path: &Path) -> String {
//         unimplemented!();
//     }
//
//     // Will trigger an importer for a referenced file and return the imported asset ID
//     pub fn import_file(path: &Path) -> ObjectId {
//         unimplemented!();
//     }
// }
//
//
//
// struct ImportContext {
//     referenced_files: Vec<PathBuf>,
//     referenced_assets: Vec<ObjectId>,
// }
//
// impl ImportContext {
//     // Will read the file, and if we are live-reloading changes, trigger a re-import if the file changes
//     // This is used when the file is not referenced by another asset, or there is no desire to
//     // import it once and have several assets share it
//     pub fn read(path: &Path) -> Vec<u8> {
//         unimplemented!();
//     }
//
//     pub fn read_to_string(path: &Path) -> String {
//         unimplemented!();
//     }
//
//     // Will trigger an importer for a referenced file and return the imported asset ID
//     pub fn import_file(path: &Path) -> ObjectId {
//         unimplemented!();
//     }
// }
use ::image::{EncodableLayout, GenericImageView};
use nexdb::{DataSet, DataSource, EditorModel, FileSystemObjectDataSource, HashMap, HashMapKeys, ImporterId, ImportInfo, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint, SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use imnodes::EditorContext;
use rafx::api::objc::runtime::Object;
use uuid::Uuid;
use type_uuid::{TypeUuid, TypeUuidDynamic};

mod image_importer;
pub use image_importer::ImageImporter;

mod material_importer;
pub use material_importer::MaterialImporter;

use nexdb::dir_tree_blob_store::{path_to_uuid, uuid_to_path};
use nexdb::edit_context::EditContext;
use nexdb::json::SingleObjectJson;


// Create ImportJobs
// - It immediately scans assets to create jobs for recognized assets
// - Kick off job to do initial imports metadata scan
//



// ENQUEUE IF
// - asset exists, import data doesn't exist, source file is available
// - asset exists, import data exists but is stale, source file is available

struct ImportOp {
    object_ids: HashMap<Option<String>, ObjectId>,
    importer_id: ImporterId,
    path: PathBuf,
    //pub(crate) import_info: ImportInfo,
}

struct ImportJob {
    object_id: ObjectId,
    import_data_exists: bool,
    asset_exists: bool,
    imported_data_stale: bool, // how to know it's stale? (we need timestamp/filesize stored along with import data, and paths to file it included) We may not know until we try to open it
    imported_data_invalid: bool // how to know it's valid? (does it parse? does it have errors? we may not know until we try to open it)
}

impl ImportJob {
    pub fn new(object_id: ObjectId) -> Self {
        ImportJob {
            object_id,
            import_data_exists: false,
            asset_exists: false,
            imported_data_stale: false,
            imported_data_invalid: false,
        }
    }
}

// Cache of all import jobs. This includes imports that are complete, in progress, or not started
pub struct ImportJobs {
    //import_editor_model: EditorModel
    root_path: PathBuf,
    import_jobs: HashMap<ObjectId, ImportJob>,
    import_operations: Vec<ImportOp>
}

impl ImportJobs {
    pub fn new(importer_registry: &ImporterRegistry, editor_model: &EditorModel, root_path: PathBuf) -> Self {
        let import_jobs = ImportJobs::find_jobs_in_assets(importer_registry, editor_model, &root_path);

        ImportJobs {
            root_path,
            import_jobs,
            import_operations: Default::default()
        }
    }

    pub fn queue_import_operation(&mut self, object_ids: HashMap<Option<String>,ObjectId>, importer_id: ImporterId, path: PathBuf) {


        self.import_operations.push(ImportOp {
            object_ids,
            importer_id,
            path,
            //import_info
        })
    }

    pub fn update(&mut self, importer_registry: &ImporterRegistry, editor_model: &EditorModel) {
        for import_op in &self.import_operations {
            //let importer_id = editor_model.root_edit_context().import_info()
            let importer_id = import_op.importer_id;
            //let fingerprint = editor_model.root_edit_context().object_schema(import_op.import_info).unwrap().fingerprint();
            //let importer_id = importer_registry.asset_to_importer.get(&fingerprint).unwrap();
            let importer = importer_registry.importer(importer_id).unwrap();

            let mut referenced_source_file_paths = Vec::default();
            let imported_objects = importer.import_file(
                &import_op.path,
                &import_op.object_ids,
                editor_model.schema_set(),
                &mut referenced_source_file_paths
            );

            for (name, imported_object) in imported_objects {
                if let Some(object_id) = import_op.object_ids.get(&name) {
                    let data = SingleObjectJson::save_single_object_to_string(&imported_object);
                    let path = uuid_to_path(&self.root_path, object_id.as_uuid(), "af");

                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }

                    std::fs::write(&path, data).unwrap()
                }
            }
        }

        self.import_operations.clear();

        // Send/mark for processing?
    }

    fn find_jobs_in_assets(importer_registry: &ImporterRegistry, editor_model: &EditorModel, root_path: &Path) -> HashMap<ObjectId, ImportJob> {
        let mut import_jobs = HashMap::<ObjectId, ImportJob>::default();

        //
        // Scan import dir for known import data
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(    root_path, &["**.i"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                println!("dir file {:?}", file);
                let dir_uuid = path_to_uuid(root_path, file.path()).unwrap();
                let object_id = ObjectId(dir_uuid.as_u128());
                let job = import_jobs.entry(object_id).or_insert_with(|| ImportJob::new(object_id));
                job.import_data_exists = true;
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
                    let job = import_jobs.entry(*object_id).or_insert_with(|| ImportJob::new(*object_id));
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











#[derive(Default)]
pub struct ImporterRegistry {
    registered_importers: HashMap<ImporterId, Box<Importer>>,
    file_extension_associations: HashMap<String, Vec<ImporterId>>,
    //asset_to_importer: HashMap<SchemaFingerprint, ImporterId>,
}

impl ImporterRegistry {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: TypeUuid + Importer + Default + 'static>(&mut self, linker: &mut SchemaLinker) {
        let handler = Box::new(T::default());
        handler.register_schemas(linker);
        let importer_id = ImporterId(Uuid::from_bytes(T::UUID));
        self.registered_importers.insert(importer_id, handler);

        for extension in self.registered_importers[&importer_id].supported_file_extensions() {
            self.file_extension_associations.entry(extension.to_string()).or_default().push(importer_id);
        }
    }

    //
    // Called after finished linking the schema so we can associate schema fingerprints with handlers
    //
    pub fn finished_linking(&mut self, schema_set: &SchemaSet) {
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

    pub fn importers_for_file_extension(&self, extension: &str) -> &[ImporterId] {
        const EMPTY_LIST: &'static [ImporterId] = &[];
        self.file_extension_associations.get(extension).map(|x| x.as_slice()).unwrap_or(EMPTY_LIST)
    }

    // pub fn handler_for_asset(&self, fingerprint: SchemaFingerprint) -> Option<ImporterId> {
    //     self.asset_to_importer.get(&fingerprint).copied()
    // }

    pub fn importer(&self, importer_id: ImporterId) -> Option<&Box<Importer>> {
        self.registered_importers.get(&importer_id)
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

pub struct ScannedImportable {
    pub name: Option<String>,
    pub asset_type: SchemaRecord,
    pub referenced_source_file_paths: PathBuf,
}

// ID?
pub trait Importer : TypeUuidDynamic {
    fn importer_id(&self) -> ImporterId {
        ImporterId(Uuid::from_bytes(self.uuid()))
    }

    fn register_schemas(&self, schema_linker: &mut SchemaLinker);

    //fn asset_types(&self) -> &[&'static str];

    fn supported_file_extensions(&self) -> &[&'static str];

    // fn create_default_asset(&self, path: &Path, editor_model: &mut EditorModel, location: ObjectLocation) {
    //     let name = if let Some(name) = path.file_name() {
    //         ObjectName::new(name.to_string_lossy())
    //     } else {
    //         ObjectName::empty()
    //     };
    //
    //     let schema_record = editor_model.root_edit_context_mut().schema_set().find_named_type().unwrap().as_record().unwrap();
    //
    //     let new_object = editor_model.root_edit_context_mut().new_object(&name, &location, schema_record);
    // }

    //fn create_default_import_options(&self, schema_set: &SchemaSet) -> SingleObject;


    //
    // Open the file and determine what assets exist in it that can be imported
    //
    fn scan_file(&self, path: &Path, schema_set: &SchemaSet) -> Vec<ScannedImportable>;

    fn create_default_asset(&self, editor_model: &mut EditorModel, object_name: ObjectName, object_location: ObjectLocation) -> ObjectId;

    // fn scan_for_referenced_source_file_paths(
    //     &self,
    //     //scan_context: &mut ScanContext,
    //     path: &Path,
    // );

    fn import_file(
        &self,
        path: &Path,
        object_ids: &HashMap<Option<String>, ObjectId>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
        referenced_source_file_paths: &mut Vec<PathBuf>,
    ) -> HashMap<Option<String>, SingleObject>;
}

// ID?
trait Processor {
    fn process_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Vec<u8>;
}

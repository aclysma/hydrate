use hydrate_base::hashing::{HashMap, HashSet};
use hydrate_base::AssetId;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::import_thread_pool::{
    ImportThreadOutcome, ImportThreadRequest, ImportThreadRequestImport, ImportWorkerThreadPool,
};
use crate::{DynEditorModel, PipelineResult};
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};
use hydrate_data::json_storage::SingleObjectJson;
use hydrate_data::{ImporterId, SchemaSet, SingleObject};

use super::import_types::*;
use super::importer_registry::*;

pub fn load_import_data(
    import_data_root_path: &Path,
    schema_set: &SchemaSet,
    asset_id: AssetId,
) -> ImportData {
    profiling::scope!(&format!("Load asset import data {:?}", asset_id));
    let path = uuid_to_path(import_data_root_path, asset_id.as_uuid(), "if");

    // json format
    //let str = std::fs::read_to_string(&path).unwrap();
    //let import_data = SingleObjectJson::load_single_object_from_string(schema_set, &str);

    // b3f format
    let bytes = std::fs::read(&path).unwrap();
    let import_data = SingleObjectJson::load_single_object_from_b3f(schema_set, &bytes);

    let metadata = path.metadata().unwrap();
    let metadata_hash = hash_file_metadata(&metadata);

    ImportData {
        import_data: import_data.single_object,
        contents_hash: import_data.contents_hash,
        metadata_hash,
    }
}

pub(super) fn hash_file_metadata(metadata: &std::fs::Metadata) -> u64 {
    let mut hasher = siphasher::sip::SipHasher::default();
    metadata.modified().unwrap().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    hasher.finish()
}

pub struct ImportDataMetadataHash {
    pub metadata_hash: u64,
}

pub struct ImportData {
    pub import_data: SingleObject,
    pub contents_hash: u64,
    pub metadata_hash: u64,
}

// An in-flight import operation we want to perform
#[derive(Clone)]
pub struct ImportOp {
    // The string is a key is an importable name
    pub asset_ids: HashMap<Option<String>, AssetId>,
    pub importer_id: ImporterId,
    pub path: PathBuf,
    pub assets_to_regenerate: HashSet<AssetId>,
    //pub(crate) import_info: ImportInfo,
}

// A known import job, each existing asset that imports data will have an associated import job.
// It could be in a completed state, or there could be a problem with it and we need to re-run it.
struct ImportJob {
    import_data_exists: bool,
    asset_exists: bool,
    //imported_data_stale: bool, // how to know it's stale? (we need timestamp/filesize stored along with import data, and paths to file it included) We may not know until we try to open it
    //imported_data_invalid: bool, // how to know it's valid? (does it parse? does it have errors? we may not know until we try to open it)
    imported_data_hash: Option<u64>,
}

impl ImportJob {
    pub fn new() -> Self {
        ImportJob {
            import_data_exists: false,
            asset_exists: false,
            //imported_data_stale: false,
            //imported_data_invalid: false,
            imported_data_hash: None,
        }
    }
}

// Cache of all known import jobs. This includes imports that are complete, in progress, or not started.
// We find these by scanning existing assets and import data. We also inspect the asset and imported
// data to see if the job is complete, or is in a failed or stale state.
pub struct ImportJobs {
    //import_editor_model: EditorModel
    import_data_root_path: PathBuf,
    import_jobs: HashMap<AssetId, ImportJob>,
    import_operations: Vec<ImportOp>,
}

impl ImportJobs {
    pub fn import_data_root_path(&self) -> &Path {
        &self.import_data_root_path
    }

    pub fn new(
        importer_registry: &ImporterRegistry,
        editor_model: &dyn DynEditorModel,
        import_data_root_path: &Path,
    ) -> Self {
        let import_jobs =
            ImportJobs::find_all_jobs(importer_registry, editor_model, import_data_root_path);

        ImportJobs {
            import_data_root_path: import_data_root_path.to_path_buf(),
            import_jobs,
            import_operations: Default::default(),
        }
    }

    pub fn queue_import_operation(
        &mut self,
        asset_ids: HashMap<Option<String>, AssetId>,
        importer_id: ImporterId,
        path: PathBuf,
        assets_to_regenerate: HashSet<AssetId>,
    ) {
        self.import_operations.push(ImportOp {
            asset_ids,
            importer_id,
            path,
            assets_to_regenerate,
            //import_info
        })
    }

    pub fn load_import_data_hash(
        &self,
        asset_id: AssetId,
    ) -> ImportDataMetadataHash {
        let path = uuid_to_path(&self.import_data_root_path, asset_id.as_uuid(), "if");
        //println!("LOAD DATA HASH PATH {:?}", path);
        let metadata = path.metadata().unwrap();
        let metadata_hash = hash_file_metadata(&metadata);
        ImportDataMetadataHash { metadata_hash }
    }

    // We do a clone because we want to allow background processing of this data and detecting if
    // import data changed at end of the build - which would invalidate it
    pub fn clone_import_data_metadata_hashes(&self) -> HashMap<AssetId, u64> {
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
    //         if let Ok(relative) = file_update.strip_prefix(&self.import_data_root_path) {
    //             if let Some(uuid) = path_to_uuid(&self.import_data_root_path, file_update) {
    //                 let asset_id = AssetId(uuid.as_u128());
    //
    //             }
    //         }
    //     }
    // }

    #[profiling::function]
    pub fn update(
        &mut self,
        importer_registry: &ImporterRegistry,
        editor_model: &mut dyn DynEditorModel,
    ) -> PipelineResult<()> {
        profiling::scope!("Process Import Operations");
        if self.import_operations.is_empty() {
            return Ok(());
        }

        //
        // Take the import operations
        //
        let mut import_operations = Vec::default();
        std::mem::swap(&mut self.import_operations, &mut import_operations);

        //
        // Create the thread pool
        //
        let thread_count = num_cpus::get();
        //let thread_count = 1;

        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        let thread_pool = ImportWorkerThreadPool::new(
            importer_registry,
            editor_model.schema_set(),
            &self.import_data_root_path,
            thread_count,
            result_tx,
        );

        //
        // Queue the import operations
        //
        let mut total_jobs = 0;
        for import_op in import_operations {
            let mut importable_assets = HashMap::<Option<String>, ImportableAsset>::default();
            for (name, asset_id) in &import_op.asset_ids {
                let referenced_paths = editor_model
                    .data_set()
                    .resolve_all_file_references(*asset_id)
                    .unwrap_or_default();
                importable_assets.insert(
                    name.clone(),
                    ImportableAsset {
                        id: *asset_id,
                        referenced_paths,
                    },
                );
            }

            total_jobs += 1;
            thread_pool.add_request(ImportThreadRequest::RequestImport(
                ImportThreadRequestImport {
                    import_op,
                    importable_assets,
                },
            ));
        }

        //
        // Wait for the thread pool to finish
        //
        let mut last_job_print_time = None;
        while !thread_pool.is_idle() {
            std::thread::sleep(std::time::Duration::from_millis(50));

            let now = std::time::Instant::now();
            let mut print_progress = true;
            if let Some(last_job_print_time) = last_job_print_time {
                if (now - last_job_print_time) < std::time::Duration::from_millis(500) {
                    print_progress = false;
                }
            }

            if print_progress {
                log::info!(
                    "Import jobs: {}/{}",
                    total_jobs - thread_pool.active_request_count(),
                    total_jobs
                );
                last_job_print_time = Some(now);
            }
        }

        thread_pool.finish();

        //
        // Commit the imports
        //
        for outcome in result_rx.try_iter() {
            match outcome {
                ImportThreadOutcome::Complete(msg) => {
                    for (name, imported_asset) in msg.result? {
                        if let Some(asset_id) = msg.request.import_op.asset_ids.get(&name) {
                            if msg
                                .request
                                .import_op
                                .assets_to_regenerate
                                .contains(asset_id)
                            {
                                editor_model
                                    .init_from_single_object(*asset_id, &imported_asset.default_asset)
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
        /*
        for import_op in &self.import_operations {
            profiling::scope!(&format!("Import {:?}", import_op.path.to_string_lossy()));
            //let importer_id = editor_model.root_edit_context().import_info()
            let importer_id = import_op.importer_id;
            //let fingerprint = editor_model.root_edit_context().asset_schema(import_op.import_info).unwrap().fingerprint();
            //let importer_id = importer_registry.asset_to_importer.get(&fingerprint).unwrap();
            let importer = importer_registry.importer(importer_id).unwrap();

            let mut importable_assets = HashMap::<Option<String>, ImportableAsset>::default();
            for (name, asset_id) in &import_op.asset_ids {
                let referenced_paths = editor_model
                    .root_edit_context()
                    .resolve_all_file_references(*asset_id)
                    .unwrap_or_default();
                importable_assets.insert(
                    name.clone(),
                    ImportableAsset {
                        id: *asset_id,
                        referenced_paths,
                    },
                );
            }

            let imported_assets = {
                profiling::scope!("Importer::import_file");
                importer.import_file(
                    &import_op.path,
                    &importable_assets,
                    editor_model.schema_set(),
                )
            };

            //TODO: Validate that all requested importables exist?
            for (name, imported_asset) in imported_assets {
                if let Some(asset_id) = import_op.asset_ids.get(&name) {
                    let type_name = editor_model
                        .root_edit_context()
                        .data_set()
                        .asset_schema(*asset_id)
                        .unwrap()
                        .name();

                    profiling::scope!(&format!("Importable {:?} {}", name, type_name));

                    if import_op.assets_to_regenerate.contains(asset_id) {
                        if let Some(default_asset) = &imported_asset.default_asset {
                            editor_model
                                .root_edit_context_mut()
                                .init_from_single_object(*asset_id, default_asset)
                                .unwrap();
                        }
                    }

                    if let Some(import_data) = &imported_asset.import_data {
                        // Json-only format
                        // let data = SingleObjectJson::save_single_object_to_string(import_data)
                        //     .into_bytes();

                        // b3f format
                        let mut buf_writer = BufWriter::new(Vec::default());
                        SingleObjectJson::save_single_object_to_b3f(&mut buf_writer, import_data);
                        let data = buf_writer.into_inner().unwrap();

                        let path = uuid_to_path(&self.import_data_root_path, asset_id.as_uuid(), "if");

                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent).unwrap();
                        }

                        let mut file_needs_write = true;
                        if path.exists() {
                            let data_on_disk = std::fs::read(&path).unwrap();

                            let mut data_hasher = siphasher::sip::SipHasher::default();
                            data_on_disk.hash(&mut data_hasher);
                            let data_on_disk_hash = data_hasher.finish();

                            let mut data_hasher = siphasher::sip::SipHasher::default();
                            data.hash(&mut data_hasher);
                            let data_hash = data_hasher.finish();

                            if data_on_disk_hash == data_hash {
                                file_needs_write = false;
                            }
                        }

                        if file_needs_write {
                            // Avoid unnecessary writes, they mutate the last modified date of the
                            // file and trigger unnecessary rebuilds
                            std::fs::write(&path, data).unwrap();
                        }

                        let metadata = path.metadata().unwrap();
                        let metadata_hash = hash_file_metadata(&metadata);
                        let import_job = self
                            .import_jobs
                            .entry(*asset_id)
                            .or_insert_with(|| ImportJob::new());
                        import_job.import_data_exists = true;
                        import_job.imported_data_hash = Some(metadata_hash);
                    }
                }
            }
        }
        */

        //self.import_operations.clear();

        // Send/mark for processing?
    }

    fn find_all_jobs(
        importer_registry: &ImporterRegistry,
        editor_model: &dyn DynEditorModel,
        import_data_root_path: &Path,
    ) -> HashMap<AssetId, ImportJob> {
        let mut import_jobs = HashMap::<AssetId, ImportJob>::default();

        //
        // Scan import dir for known import data
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(import_data_root_path, &["**.if"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                let file = dunce::canonicalize(&file.path()).unwrap();
                //println!("import file {:?}", file);
                let import_file_uuid = path_to_uuid(import_data_root_path, &file).unwrap();
                let asset_id = AssetId::from_uuid(import_file_uuid);
                let job = import_jobs
                    .entry(asset_id)
                    .or_insert_with(|| ImportJob::new());

                let file_metadata = file.metadata().unwrap();
                let import_data_hash = hash_file_metadata(&file_metadata);

                job.import_data_exists = true;
                job.imported_data_hash = Some(import_data_hash);
            }
        }

        //
        // Scan assets to find any asset that has an associated importer
        //
        for (asset_id, _) in editor_model.data_set().assets() {
            if let Some(import_info) = editor_model.data_set().import_info(*asset_id) {
                let importer_id = import_info.importer_id();
                let importer = importer_registry.importer(importer_id);
                if importer.is_some() {
                    let job = import_jobs
                        .entry(*asset_id)
                        .or_insert_with(|| ImportJob::new());
                    job.asset_exists = true;
                }
            }
        }

        import_jobs

        // for (asset_id, job) in import_jobs {
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

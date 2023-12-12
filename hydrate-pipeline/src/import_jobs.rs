use hydrate_base::hashing::HashMap;
use hydrate_base::AssetId;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crossbeam_channel::Receiver;

use crate::import_storage::ImportDataMetadata;
use crate::import_thread_pool::{
    ImportThreadOutcome, ImportThreadRequest, ImportThreadRequestImport, ImportWorkerThreadPool,
};
use crate::import_util::RequestedImportable;
use crate::{DynEditorModel, HydrateProjectConfiguration, PipelineResult};
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};
use hydrate_data::ImportableName;
use hydrate_data::{ImporterId, SchemaSet, SingleObject};

use super::import_types::*;
use super::importer_registry::*;

pub fn load_import_data(
    import_data_root_path: &Path,
    schema_set: &SchemaSet,
    asset_id: AssetId,
) -> PipelineResult<ImportData> {
    profiling::scope!(&format!("Load asset import data {:?}", asset_id));
    let path = uuid_to_path(import_data_root_path, asset_id.as_uuid(), "if");

    // b3f format
    let file = std::fs::File::open(&path)?;
    let mut buf_reader = BufReader::new(file);
    let import_data =
        super::import_storage::load_import_data_from_b3f(schema_set, &mut buf_reader)?;

    let metadata = path.metadata()?;
    let metadata_hash = hash_file_metadata(&metadata);

    Ok(ImportData {
        import_data: import_data.single_object,
        contents_hash: import_data.metadata.import_data_contents_hash,
        metadata_hash,
    })
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ImportType {
    // Used when the asset doesn't exist
    ImportAlways,
    // Used if the asset already exists
    ImportIfImportDataStale,
}

// An in-flight import operation we want to perform
#[derive(Clone, Debug)]
pub struct ImportOp {
    // The string is a key is an importable name
    pub requested_importables: HashMap<ImportableName, RequestedImportable>,
    pub importer_id: ImporterId,
    pub path: PathBuf,
    pub import_type: ImportType,
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

pub struct ImportStatusImporting {
    pub total_job_count: usize,
    pub completed_job_count: usize
}

pub enum ImportStatus {
    Idle,
    Importing(ImportStatusImporting)
}

struct ImportTask {
    thread_pool: ImportWorkerThreadPool,
    job_count: usize,
    result_rx: Receiver<ImportThreadOutcome>,
}

// Cache of all known import jobs. This includes imports that are complete, in progress, or not started.
// We find these by scanning existing assets and import data. We also inspect the asset and imported
// data to see if the job is complete, or is in a failed or stale state.
pub struct ImportJobs {
    //import_editor_model: EditorModel
    project_config: HydrateProjectConfiguration,
    import_data_root_path: PathBuf,
    import_jobs: HashMap<AssetId, ImportJob>,
    import_operations: Vec<ImportOp>,
    current_import_task: Option<ImportTask>,
}

impl ImportJobs {
    pub fn import_data_root_path(&self) -> &Path {
        &self.import_data_root_path
    }

    pub fn new(
        project_config: &HydrateProjectConfiguration,
        importer_registry: &ImporterRegistry,
        editor_model: &dyn DynEditorModel,
        import_data_root_path: &Path,
    ) -> Self {
        let import_jobs =
            ImportJobs::find_all_jobs(importer_registry, editor_model, import_data_root_path);

        ImportJobs {
            project_config: project_config.clone(),
            import_data_root_path: import_data_root_path.to_path_buf(),
            import_jobs,
            import_operations: Default::default(),
            current_import_task: None,
        }
    }

    pub fn queue_import_operation(
        &mut self,
        asset_ids: HashMap<ImportableName, RequestedImportable>,
        importer_id: ImporterId,
        path: PathBuf,
        import_type: ImportType,
    ) {
        self.import_operations.push(ImportOp {
            requested_importables: asset_ids,
            importer_id,
            path,
            import_type,
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

    #[profiling::function]
    pub fn start_import_task(
        &mut self,
        importer_registry: &ImporterRegistry,
        editor_model: &mut dyn DynEditorModel,
    ) -> PipelineResult<ImportTask> {
        //
        // Take the import operations
        //
        let mut import_operations = Vec::default();
        std::mem::swap(&mut self.import_operations, &mut import_operations);

        //
        // Cache the import info for all assets
        //
        let mut existing_asset_import_state = HashMap::default();
        for (asset_id, asset_info) in editor_model.data_set().assets() {
            if let Some(import_info) = asset_info.import_info() {
                let import_metadata = ImportDataMetadata {
                    source_file_size: import_info.source_file_size(),
                    source_file_modified_timestamp: import_info.source_file_modified_timestamp(),
                    import_data_contents_hash: import_info.import_data_contents_hash(),
                };
                existing_asset_import_state.insert(*asset_id, import_metadata);
            }
        }
        let existing_asset_import_state = Arc::new(existing_asset_import_state);

        //
        // Create the thread pool
        //
        let thread_count = num_cpus::get();
        //let thread_count = 1;

        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        let thread_pool = ImportWorkerThreadPool::new(
            &self.project_config,
            importer_registry,
            editor_model.schema_set(),
            &existing_asset_import_state,
            &self.import_data_root_path,
            thread_count,
            result_tx,
        );

        //
        // Queue the import operations
        //
        let mut job_count = 0;
        for import_op in import_operations {
            let mut importable_assets = HashMap::<ImportableName, ImportableAsset>::default();
            for (name, requested_importable) in &import_op.requested_importables {
                let canonical_path_references = requested_importable.canonical_path_references.clone();
                let path_references = requested_importable.path_references.clone();

                // We could merge in any paths that were already configured in the asset DB. However
                // for now we rely on the code queueing the update to determine if it wants to do that
                // or not.
                // if !requested_importable.replace_with_default_asset {
                //     let asset_referenced_paths = editor_model
                //         .data_set()
                //         .resolve_all_path_references(requested_importable.asset_id)
                //         .unwrap_or_default();
                //
                //     for (k, v) in asset_referenced_paths {
                //         path_references.insert(k, v);
                //     }
                // }

                importable_assets.insert(
                    name.clone(),
                    ImportableAsset {
                        id: requested_importable.asset_id,
                        canonical_path_references,
                        path_references,
                    },
                );
            }

            job_count += 1;
            thread_pool.add_request(ImportThreadRequest::RequestImport(
                ImportThreadRequestImport {
                    import_op,
                    importable_assets,
                },
            ));
        }

        Ok(ImportTask {
            thread_pool,
            job_count,
            result_rx
        })
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        importer_registry: &ImporterRegistry,
        editor_model: &mut dyn DynEditorModel,
    ) -> PipelineResult<ImportStatus> {
        profiling::scope!("Process Import Operations");

        //
        // If we already have an import task running, report progress
        //
        if let Some(current_import_task) = &self.current_import_task {
            if !current_import_task.thread_pool.is_idle() {
                return Ok(ImportStatus::Importing(ImportStatusImporting {
                    total_job_count: current_import_task.job_count,
                    completed_job_count: current_import_task.job_count - current_import_task.thread_pool.active_request_count()
                }));
            }
        }

        //
        // If we have a completed import task, merge results back into the editor model
        //
        if let Some(finished_import_task) = self.current_import_task.take() {
            finished_import_task.thread_pool.finish();

            //
            // Commit the imports
            //
            for outcome in finished_import_task.result_rx.try_iter() {
                match outcome {
                    ImportThreadOutcome::Complete(msg) => {
                        for (name, imported_asset) in msg.result? {
                            if let Some(requested_importable) =
                                msg.request.import_op.requested_importables.get(&name)
                            {
                                editor_model.handle_import_complete(
                                    requested_importable.asset_id,
                                    requested_importable.asset_name.clone(),
                                    requested_importable.asset_location.clone(),
                                    &imported_asset.default_asset,
                                    requested_importable.replace_with_default_asset,
                                    imported_asset.import_info,
                                    &requested_importable.canonical_path_references,
                                    &requested_importable.path_references,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        //
        // Check if we have pending imports/should start a new import task
        //
        if self.import_operations.is_empty() {
            // Nothing is pending import
            return Ok(ImportStatus::Idle);
        }

        //
        // Start a new import task with all pending imports
        //
        let import_task = self.start_import_task(importer_registry, editor_model)?;
        let status = ImportStatus::Importing(ImportStatusImporting {
            total_job_count: import_task.job_count,
            completed_job_count: 0
        });

        assert!(self.current_import_task.is_none());
        self.current_import_task = Some(import_task);

        Ok(status)
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


use crate::import_jobs::ImportOp;
use crate::{ImportContext, ImportableAsset, ImporterRegistry, PipelineResult, ImportType};
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::uuid_path::uuid_to_path;
use hydrate_data::{ImportableName, ImportInfo, PathReference, SchemaSet, SingleObject};
use std::hash::{Hash, Hasher};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::SystemTime;
use hydrate_base::AssetId;
use crate::import_storage::{ImportDataMetadata};

// Ask the thread to gather import data from the asset
pub struct ImportThreadRequestImport {
    // pub asset_ids: HashMap<ImportableName, AssetId>,
    // pub importer_id: ImporterId,
    // pub path: PathBuf,
    pub import_op: ImportOp,
    pub importable_assets: HashMap<ImportableName, ImportableAsset>,
}

pub enum ImportThreadRequest {
    RequestImport(ImportThreadRequestImport),
}

// ImportedImportable with anything not needed for main thread to commit the work removed
pub struct ImportThreadImportedImportable {
    pub default_asset: SingleObject,
    pub import_info: ImportInfo,
}

// Results from successful import
pub struct ImportThreadOutcomeComplete {
    pub request: ImportThreadRequestImport,
    pub result: PipelineResult<HashMap<ImportableName, ImportThreadImportedImportable>>,
    //asset: SingleObject,
    //import_data: SingleObject,
}

pub enum ImportThreadOutcome {
    Complete(ImportThreadOutcomeComplete),
}

// Thread that tries to take jobs out of the request channel and ends when the finish channel is signalled
struct ImportWorkerThread {
    finish_tx: Sender<()>,
    join_handle: JoinHandle<()>,
}

fn do_import(
    importer_registry: &ImporterRegistry,
    schema_set: &SchemaSet,
    existing_asset_import_state: &HashMap<AssetId, ImportDataMetadata>,
    import_data_root_path: &Path,
    msg: &ImportThreadRequestImport,
) -> PipelineResult<HashMap<ImportableName, ImportThreadImportedImportable>> {
    //
    // Get metadata for the source file (i.e. length, last modified time)
    //
    let source_file_metadata = msg.import_op.path.metadata()?;
    let source_file_size = source_file_metadata.len();
    let source_file_modified_timestamp = source_file_metadata.modified()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Error getting duration since unix epoch: {:?}", e))?
        .as_secs();

    //
    // Compare the existing import data to the source file and see if we can skip importing this file
    //
    if msg.import_op.import_type == ImportType::ImportIfImportDataStale {
        let mut any_asset_has_stale_import_data = false;
        let mut any_asset_has_stale_asset_data = false;

        //
        // Determine if any asset has stale asset or import data.
        //
        for (_, asset) in &msg.importable_assets {
            let import_data_path = uuid_to_path(import_data_root_path, asset.id.as_uuid(), "if");
            if !import_data_path.exists() {
                //
                // Import data file is missing, we cannot reuse the data. We have to run the full import.
                //
                any_asset_has_stale_import_data = true;
                any_asset_has_stale_asset_data = true;
                break;
            }

            let mut import_data_file = std::fs::File::open(import_data_path)?;
            let metadata = super::import_storage::load_import_metadata_from_b3f(&mut import_data_file)?;
            if metadata.source_file_size != source_file_size || metadata.source_file_modified_timestamp != source_file_modified_timestamp {
                //
                // Force re-import if the import data does not match the source file size/timestamp. We can stop
                // as soon as we find stale import data because we will have to import.
                //
                any_asset_has_stale_import_data = true;
                any_asset_has_stale_asset_data = true;
                break;
            }

            // let Some(asset_import_state) = existing_asset_import_state.get(&asset.id) else {
            //     //
            //     // The asset doesn't exist or has never been imported. (Eventually we want to avoid
            //     // creating assets until the import runs so that the asset doesn't exist but that's
            //     // needs addressing.)
            //     //
            //     any_asset_has_stale_asset_data = true;
            // };
            //
            // if asset_import_state.import_data_contents_hash != metadata.import_data_contents_hash ||
            //     asset_import_state.source_file_size != metadata.source_file_size ||
            //     asset_import_state.source_file_modified_timestamp != metadata.source_file_modified_timestamp {
            //     //
            //     // The asset data does not match the source file size/timestamp. Even if import data is not
            //     // stale we still at least need to update the asset metadata.
            //     //
            //     // This is not really an expected case, like the user copies an asset file, re-imports,
            //     // then overwrites the asset file with the old asset file from before the import
            //     //
            //     any_asset_has_stale_asset_data = true;
            // }
        }

        // any stale import data = full re-import
        if !any_asset_has_stale_import_data {
            // depending on if we have stale asset data
            /*if !any_asset_has_stale_asset_data {
                //
                // Our state matches source file state, do nothing
                //
                return Ok(Default::default())
            } else*/ {
                //
                // Just the asset data is stale, we can recover it from the import data that isn't stale
                //
                let mut cached_importables = HashMap::default();

                for (name, asset) in &msg.importable_assets {
                    //
                    // Load the metadata and default asset from disk
                    //
                    let import_data_path = uuid_to_path(import_data_root_path, asset.id.as_uuid(), "if");
                    let mut import_data_file = std::fs::File::open(import_data_path)?;
                    let metadata = super::import_storage::load_import_metadata_from_b3f(&mut import_data_file)?;
                    let default_asset = super::import_storage::load_default_asset_from_b3f(schema_set, &mut import_data_file)?;

                    let metadata = ImportDataMetadata {
                        source_file_modified_timestamp,
                        source_file_size,
                        import_data_contents_hash: metadata.import_data_contents_hash,
                    };
                    let import_info = create_import_info(msg, &name, metadata);

                    let old = cached_importables.insert(
                        name.clone(),
                        ImportThreadImportedImportable {
                            default_asset,
                            import_info,
                        },
                    );
                    assert!(old.is_none());
                }

                return Ok(cached_importables);
            }
        }
    }

    let importer_id = msg.import_op.importer_id;
    let importer = importer_registry.importer(importer_id).unwrap();
    let mut imported_importables = HashMap::default();

    //
    // Do the import
    //
    {
        profiling::scope!("Importer::import_file");
        importer.import_file(ImportContext::new(
            &msg.import_op.path,
            &msg.importable_assets,
            schema_set,
            &mut imported_importables,
        ))?
    }

    //
    // Write import data for each imported asset to disk
    //
    let mut written_importables = HashMap::default();

    for (name, imported_asset) in imported_importables {
        if let Some(requested_importable) = msg.import_op.requested_importables.get(&name) {
            let default_asset = &imported_asset.default_asset;
            let type_name = default_asset.schema().name();

            profiling::scope!(&format!("Importable {:?} {}", name, type_name));

            let mut import_data_metadata = ImportDataMetadata {
                source_file_modified_timestamp,
                source_file_size,
                import_data_contents_hash: 0,
            };

            //
            // Write the import file to disk
            //
            {
                let mut buf_writer = BufWriter::new(Vec::default());

                if let Some(import_data) = &imported_asset.import_data {
                    let mut contents_hasher = siphasher::sip::SipHasher::default();
                    import_data.hash(&mut contents_hasher);
                    import_data_metadata.import_data_contents_hash = contents_hasher.finish();
                }

                super::import_storage::save_single_object_to_b3f(
                    &mut buf_writer,
                    imported_asset.import_data.as_ref(),
                    &import_data_metadata,
                    &imported_asset.default_asset
                );

                let data_to_write = buf_writer
                    .into_inner()
                    .map_err(|e| format!("Error converting bufwriter to Vec<u8>: {:?}", e))?;

                let path = uuid_to_path(import_data_root_path, requested_importable.asset_id.as_uuid(), "if");

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
                    data_to_write.hash(&mut data_hasher);
                    let data_hash = data_hasher.finish();

                    if data_on_disk_hash == data_hash {
                        file_needs_write = false;
                    }
                }

                if file_needs_write {
                    // Avoid unnecessary writes, they mutate the last modified date of the
                    // file and trigger unnecessary rebuilds
                    std::fs::write(&path, data_to_write).unwrap();
                }
            }

            let import_info = create_import_info(msg, &name, import_data_metadata);
            let old = written_importables.insert(
                name,
                ImportThreadImportedImportable {
                    default_asset: imported_asset.default_asset,
                    import_info,
                },
            );
            assert!(old.is_none());
        } else {
            unimplemented!()
        }
    }

    Ok(written_importables)
}

fn create_import_info(msg: &ImportThreadRequestImport, name: &ImportableName, import_data_metadata: ImportDataMetadata) -> ImportInfo {
    let source_file = PathReference::new(msg.import_op.path.to_string_lossy().to_string(), name.clone());
    let import_info = ImportInfo::new(
        msg.import_op.importer_id,
        source_file,
        msg.importable_assets[&name].referenced_paths.keys().cloned().collect(),
        import_data_metadata.source_file_modified_timestamp,
        import_data_metadata.source_file_size,
        import_data_metadata.import_data_contents_hash
    );
    import_info
}

impl ImportWorkerThread {
    fn new(
        importer_registry: ImporterRegistry,
        schema_set: SchemaSet,
        existing_asset_import_state: Arc<HashMap<AssetId, ImportDataMetadata>>,
        import_data_root_path: Arc<PathBuf>,
        request_rx: Receiver<ImportThreadRequest>,
        outcome_tx: Sender<ImportThreadOutcome>,
        active_request_count: Arc<AtomicUsize>,
        thread_index: usize,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new()
            .name("IO Thread".into())
            .spawn(move || {
                profiling::register_thread!(&format!("ImportWorkerThread {}", thread_index));
                loop {
                    crossbeam_channel::select! {
                        recv(request_rx) -> msg => {
                            match msg.unwrap() {
                                ImportThreadRequest::RequestImport(msg) => {
                                    profiling::scope!("ImportThreadRequest::RequestImport");
                                    let result = do_import(
                                        &importer_registry,
                                        &schema_set,
                                        &*existing_asset_import_state,
                                        &*import_data_root_path,
                                        &msg,
                                    );

                                    outcome_tx.send(ImportThreadOutcome::Complete(ImportThreadOutcomeComplete {
                                        request: msg,
                                        result,
                                    })).unwrap();
                                    active_request_count.fetch_sub(1, Ordering::Release);
                                },
                            }
                        },
                        recv(finish_rx) -> _msg => {
                            return;
                        }
                    }
                }
            })
            .unwrap();

        ImportWorkerThread {
            finish_tx,
            join_handle,
        }
    }
}

// Spans N threads, proxies messages to/from them, and kills the threads when the pool is dropped
pub struct ImportWorkerThreadPool {
    worker_threads: Vec<ImportWorkerThread>,
    request_tx: Sender<ImportThreadRequest>,
    active_request_count: Arc<AtomicUsize>,
}

impl ImportWorkerThreadPool {
    pub fn new(
        importer_registry: &ImporterRegistry,
        schema_set: &SchemaSet,
        existing_asset_import_state: &Arc<HashMap<AssetId, ImportDataMetadata>>,
        import_data_root_path: &Path,
        max_requests_in_flight: usize,
        result_tx: Sender<ImportThreadOutcome>,
    ) -> Self {
        let import_data_root_path = Arc::new(import_data_root_path.to_path_buf());
        let (request_tx, request_rx) = crossbeam_channel::unbounded::<ImportThreadRequest>();
        let active_request_count = Arc::new(AtomicUsize::new(0));

        let mut worker_threads = Vec::with_capacity(max_requests_in_flight);
        for thread_index in 0..max_requests_in_flight {
            let worker = ImportWorkerThread::new(
                importer_registry.clone(),
                schema_set.clone(),
                existing_asset_import_state.clone(),
                import_data_root_path.clone(),
                request_rx.clone(),
                result_tx.clone(),
                active_request_count.clone(),
                thread_index,
            );
            worker_threads.push(worker);
        }

        ImportWorkerThreadPool {
            request_tx,
            worker_threads,
            active_request_count,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.active_request_count() == 0
    }

    pub fn active_request_count(&self) -> usize {
        self.active_request_count.load(Ordering::Relaxed)
    }

    pub fn add_request(
        &self,
        request: ImportThreadRequest,
    ) {
        self.active_request_count.fetch_add(1, Ordering::Release);
        self.request_tx.send(request).unwrap();
    }

    pub fn finish(self) {
        for worker_thread in &self.worker_threads {
            worker_thread.finish_tx.send(()).unwrap();
        }

        for worker_thread in self.worker_threads {
            worker_thread.join_handle.join().unwrap();
        }
    }
}

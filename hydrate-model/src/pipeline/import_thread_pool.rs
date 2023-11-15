use std::hash::{Hash, Hasher};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::uuid_path::uuid_to_path;
use hydrate_data::{SchemaSet, SingleObject};
use crate::{ImportableAsset, ImporterRegistry, SingleObjectJson};
use crate::pipeline::import_jobs::ImportOp;

// Ask the thread to gather import data from the asset
pub struct ImportThreadRequestImport {
    // pub asset_ids: HashMap<Option<String>, AssetId>,
    // pub importer_id: ImporterId,
    // pub path: PathBuf,
    // pub assets_to_regenerate: HashSet<AssetId>,
    pub import_op: ImportOp,
    pub importable_assets: HashMap<Option<String>, ImportableAsset>
}

pub enum ImportThreadRequest {
    RequestImport(ImportThreadRequestImport),
}

// ImportedImportable with anything not needed for main thread to commit the work removed
pub struct ImportThreadImportedImportable {
    pub default_asset: Option<SingleObject>,
}

// Results from successful import
pub struct ImportThreadOutcomeComplete {
    pub request: ImportThreadRequestImport,
    pub imported_importables: HashMap<Option<String>, ImportThreadImportedImportable>,

    //asset: SingleObject,
    //import_data: SingleObject,
}

// Results from failed import
pub struct ImportThreadOutcomeFailed {
    pub failure: String,
}

pub enum ImportThreadOutcome {
    Complete(ImportThreadOutcomeComplete),
    Failed(ImportThreadOutcomeFailed)
}

// Thread that tries to take jobs out of the request channel and ends when the finish channel is signalled
struct ImportWorkerThread {
    finish_tx: Sender<()>,
    join_handle: JoinHandle<()>,
}

fn do_import(
    importer_registry: &ImporterRegistry,
    schema_set: &SchemaSet,
    import_data_root_path: &Path,
    msg: ImportThreadRequestImport
) -> ImportThreadOutcome {
    let importer_id = msg.import_op.importer_id;
    let importer = importer_registry.importer(importer_id).unwrap();

    let imported_assets = {
        profiling::scope!("Importer::import_file");
        importer.import_file(
            &msg.import_op.path,
            &msg.importable_assets,
            schema_set,
        )
    };

    let mut imported_importables = HashMap::default();

    for (name, imported_asset) in imported_assets {
        if let Some(asset_id) = msg.import_op.asset_ids.get(&name) {
            let default_asset = imported_asset.default_asset.as_ref().unwrap();
            let type_name = default_asset.schema().name();

            profiling::scope!(&format!("Importable {:?} {}", name, type_name));

            if let Some(import_data) = &imported_asset.import_data {
                // Json-only format
                // let data = SingleObjectJson::save_single_object_to_string(import_data)
                //     .into_bytes();

                // b3f format
                let mut buf_writer = BufWriter::new(Vec::default());
                SingleObjectJson::save_single_object_to_b3f(&mut buf_writer, import_data);
                let data = buf_writer.into_inner().unwrap();

                let path = uuid_to_path(import_data_root_path, asset_id.as_uuid(), "if");

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
                let metadata_hash = super::import_jobs::hash_file_metadata(&metadata);
            }
        }

        imported_importables.insert(name, ImportThreadImportedImportable {
            default_asset: imported_asset.default_asset
        });
    }

    ImportThreadOutcome::Complete(ImportThreadOutcomeComplete {
        request: msg,
        imported_importables,
    })
}

impl ImportWorkerThread {
    fn new(
        importer_registry: ImporterRegistry,
        schema_set: SchemaSet,
        import_data_root_path: Arc<PathBuf>,
        request_rx: Receiver<ImportThreadRequest>,
        outcome_tx: Sender<ImportThreadOutcome>,
        active_request_count: Arc<AtomicUsize>,
        thread_index: usize,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new().name("IO Thread".into()).spawn(move || {
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
                                    &*import_data_root_path,
                                    msg,
                                );

                                outcome_tx.send(result).unwrap();
                                active_request_count.fetch_sub(1, Ordering::Release);
                            },
                        }
                    },
                    recv(finish_rx) -> _msg => {
                        return;
                    }
                }
            }
        }).unwrap();

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

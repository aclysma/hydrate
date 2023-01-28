use crate::distill_core::AssetTypeId;
use crate::distill_loader::LoadHandle;
use crate::loader::ObjectData;
use crate::loader::{
    CombinedBuildHash, Loader, LoaderEvent, LoaderIO, ObjectMetadata, RequestDataResult,
    RequestMetadataResult,
};
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_model::ObjectId;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

struct DiskAssetIORequestMetadata {
    object_id: ObjectId,
    load_handle: LoadHandle,
    version: u32,
    hash: u64,
}

struct DiskAssetIORequestData {
    object_id: ObjectId,
    load_handle: LoadHandle,
    hash: u64,
    version: u32,
    subresource: Option<u32>,
}

enum DiskAssetIORequest {
    Metadata(DiskAssetIORequestMetadata),
    Data(DiskAssetIORequestData),
}

// Thread that tries to take jobs out of the request channel and ends when the finish channel is signalled
struct DiskAssetIOWorkerThread {
    finish_tx: Sender<()>,
    join_handle: JoinHandle<()>,
}

impl DiskAssetIOWorkerThread {
    fn new(
        root_path: Arc<PathBuf>,
        request_rx: Receiver<DiskAssetIORequest>,
        result_tx: Sender<LoaderEvent>,
        active_request_count: Arc<AtomicUsize>,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new().name("IO Thread".into()).spawn(move || {
            loop {
                crossbeam_channel::select! {
                    recv(request_rx) -> msg => {
                        match msg.unwrap() {
                            DiskAssetIORequest::Metadata(msg) => {
                                //let path = hydrate_model::uuid_path::uuid_and_hash_to_path(&*root_path, msg.object_id.as_uuid(), msg.hash, "bf");
                                log::trace!("Start metadata read {:?}", msg.object_id);
                                //let t0 = std::time::Instant::now();
                                // let result = std::fs::read(&path);
                                // //let t1 = std::time::Instant::now();
                                // match &result {
                                //     Ok(data) => {
                                //         //log::debug!("Read {:?} {:?} {} bytes in {}ms", object_id, subresource, data.len(), (t1 - t0).as_secs_f32() * 1000.0);
                                //     },
                                //     Err(e) => {
                                //         log::warn!("Failed to read metadata {:?} at path {:?}", msg.object_id, path);
                                //     }
                                // }

                                let metadata = ObjectMetadata {
                                    dependencies: vec![],
                                    subresource_count: 0,
                                    asset_type: AssetTypeId(*uuid::Uuid::parse_str("1a4dde10-5e60-483d-88fa-4f59752e4524").unwrap().as_bytes()),
                                    hash: msg.hash,
                                };

                                result_tx.send(LoaderEvent::MetadataRequestComplete( RequestMetadataResult {
                                    object_id: msg.object_id,
                                    load_handle: msg.load_handle,
                                    //subresource: msg.subresource,
                                    //hash: msg.hash,
                                    version: msg.version,
                                    result: Ok(metadata)
                                })).unwrap();
                                active_request_count.fetch_sub(1, Ordering::Release);
                            },
                            DiskAssetIORequest::Data(msg) => {
                                let path = hydrate_model::uuid_path::uuid_and_hash_to_path(&*root_path, msg.object_id.as_uuid(), msg.hash, "bf");
                                log::trace!("Start read {:?} {:?}", msg.object_id, msg.subresource);
                                //let t0 = std::time::Instant::now();
                                let result = std::fs::read(&path);
                                //let t1 = std::time::Instant::now();
                                match result {
                                    Ok(data) => {
                                        //log::debug!("Read {:?} {:?} {} bytes in {}ms", object_id, subresource, data.len(), (t1 - t0).as_secs_f32() * 1000.0);

                                        result_tx.send(LoaderEvent::DataRequestComplete(RequestDataResult {
                                            object_id: msg.object_id,
                                            load_handle: msg.load_handle,
                                            subresource: msg.subresource,
                                            version: msg.version,
                                            //hash: msg.hash,
                                            result: Ok(ObjectData {
                                                data
                                            })
                                        })).unwrap();
                                    },
                                    Err(e) => {
                                        log::warn!("Failed to read {:?} {:?} at path {:?}", msg.object_id, msg.subresource, path);
                                    }
                                }

                                active_request_count.fetch_sub(1, Ordering::Release);
                            }
                        }
                    },
                    recv(finish_rx) -> msg => {
                        return;
                    }
                }
            }
        }).unwrap();

        DiskAssetIOWorkerThread {
            finish_tx,
            join_handle,
        }
    }
}

// Spans N threads, proxies messages to/from them, and kills the threads when the pool is dropped
struct DiskAssetIOThreadPool {
    worker_threads: Vec<DiskAssetIOWorkerThread>,
    request_tx: Sender<DiskAssetIORequest>,
    // result_tx: Sender<DiskAssetIOResult>,
    // result_rx: Receiver<DiskAssetIOResult>,
    active_request_count: Arc<AtomicUsize>,
}

impl DiskAssetIOThreadPool {
    fn new(
        root_path: Arc<PathBuf>,
        max_requests_in_flight: usize,
        result_tx: Sender<LoaderEvent>,
    ) -> Self {
        let (request_tx, request_rx) = crossbeam_channel::unbounded::<DiskAssetIORequest>();
        //let (result_tx, result_rx) = crossbeam_channel::unbounded();
        let active_request_count = Arc::new(AtomicUsize::new(0));

        let mut worker_threads = Vec::with_capacity(max_requests_in_flight);
        for _ in 0..max_requests_in_flight {
            let worker = DiskAssetIOWorkerThread::new(
                root_path.clone(),
                request_rx.clone(),
                result_tx.clone(),
                active_request_count.clone(),
            );
            worker_threads.push(worker);
        }

        DiskAssetIOThreadPool {
            request_tx,
            // result_tx,
            // result_rx,
            worker_threads,
            active_request_count,
        }
    }

    fn add_request(
        &self,
        request: DiskAssetIORequest,
    ) {
        self.active_request_count.fetch_add(1, Ordering::Release);
        self.request_tx.send(request).unwrap();
    }

    // fn add_result(&self, result: DiskAssetIOResult) {
    //     self.result_tx.send(result).unwrap();
    // }
    //
    // fn results_rx(&self) -> &Receiver<DiskAssetIOResult> {
    //     &self.result_rx
    // }

    fn finish(mut self) {
        for worker_thread in &self.worker_threads {
            worker_thread.finish_tx.send(()).unwrap();
        }

        for worker_thread in self.worker_threads {
            worker_thread.join_handle.join().unwrap();
        }
    }

    fn active_request_count(&self) -> usize {
        self.active_request_count.load(Ordering::Acquire)
    }
}

// struct DiskAssetIORequest {
//     object_id: ObjectId,
//     load_handle: LoadHandle,
//     hash: u64,
//     subresource: Option<u32>,
// }

// pub struct DiskAssetIOResult {
//     pub object_id: ObjectId,
//     pub load_handle: LoadHandle,
//     pub hash: u64,
//     pub subresource: Option<u32>,
//     pub result: std::io::Result<Vec<u8>>
// }

pub struct BuildManifest {
    pub asset_build_hashes: HashMap<ObjectId, u64>,
}

impl BuildManifest {
    fn load_from_file(
        manifest_dir_path: &Path,
        build_hash: CombinedBuildHash,
    ) -> BuildManifest {
        let mut asset_build_hashes = HashMap::default();

        let file_name = format!("{:0>16x}.manifest", build_hash.0);
        let file_path = manifest_dir_path.join(file_name);
        let file = std::fs::File::open(file_path).unwrap();
        let buf_reader = std::io::BufReader::new(file);
        for line in buf_reader.lines() {
            let line_str = line.unwrap().to_string();
            if line_str.is_empty() {
                continue;
            }

            let separator = line_str.find(",").unwrap();
            let left = &line_str[..separator];
            let right = &line_str[(separator + 1)..];

            let asset_id = u128::from_str_radix(left, 16).unwrap();
            let build_hash = u64::from_str_radix(right, 16).unwrap();

            asset_build_hashes.insert(ObjectId(asset_id), build_hash);
        }

        BuildManifest { asset_build_hashes }
    }
}

// Given a folder, finds the TOC that is "latest" (has highest timestamp)
fn find_latest_toc(toc_dir_path: &Path) -> Option<PathBuf> {
    let mut max_timestamp = 0;
    let mut max_timestamp_path = None;

    log::info!("find latest toc from {:?}", toc_dir_path);
    let files = std::fs::read_dir(toc_dir_path).unwrap();
    for file in files {
        let path = file.unwrap().path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        if let Some(file_name) = file_name.strip_suffix(".toc") {
            if let Ok(timestamp) = u64::from_str_radix(file_name, 16) {
                if timestamp > max_timestamp {
                    max_timestamp = timestamp;
                    max_timestamp_path = Some(path);
                }
            }
        }
    }

    max_timestamp_path
}

struct BuildToc {
    build_hash: CombinedBuildHash,
}

// Opens a TOC file and reads contents
fn read_toc(path: &Path) -> BuildToc {
    let data = std::fs::read_to_string(path).unwrap();
    let build_hash = u64::from_str_radix(&data, 16).unwrap();
    BuildToc {
        build_hash: CombinedBuildHash(build_hash),
    }
}

pub struct DiskAssetIO {
    thread_pool: Option<DiskAssetIOThreadPool>,
    manifest: BuildManifest,
    build_hash: CombinedBuildHash,
    tx: Sender<LoaderEvent>,
}

impl Drop for DiskAssetIO {
    fn drop(&mut self) {
        self.thread_pool.take().unwrap().finish();
    }
}

impl DiskAssetIO {
    pub fn new(
        build_data_root_path: PathBuf,
        tx: Sender<LoaderEvent>,
    ) -> Result<Self, String> {
        let max_toc_path = find_latest_toc(&build_data_root_path.join("toc"));
        let max_toc_path = max_toc_path.ok_or_else(|| "Could not find TOC file".to_string())?;
        let build_toc = read_toc(&max_toc_path);
        let build_hash = build_toc.build_hash;

        let manifest =
            BuildManifest::load_from_file(&build_data_root_path.join("manifests"), build_hash);
        let thread_pool = Some(DiskAssetIOThreadPool::new(
            Arc::new(build_data_root_path),
            4,
            tx.clone(),
        ));

        Ok(DiskAssetIO {
            thread_pool,
            manifest,
            build_hash,
            tx,
        })
    }

    pub fn manifest(&self) -> &BuildManifest {
        &self.manifest
    }

    pub fn request_data(
        &self,
        load_handle: LoadHandle,
        object_id: ObjectId,
        subresource: Option<u32>,
        version: u32,
        hash: u64,
    ) {
        log::debug!("Request {:?} {:?}", object_id, subresource);
        let hash = self.manifest.asset_build_hashes.get(&object_id);
        if let Some(&hash) = hash {
            self.thread_pool
                .as_ref()
                .unwrap()
                .add_request(DiskAssetIORequest::Data(DiskAssetIORequestData {
                    object_id,
                    load_handle,
                    //hash,
                    version,
                    subresource,
                    hash,
                }));
        } else {
            self.tx
                .send(LoaderEvent::DataRequestComplete(RequestDataResult {
                    object_id,
                    load_handle,
                    version,
                    subresource,
                    //hash: 0,
                    result: Err(std::io::ErrorKind::NotFound.into()),
                }))
                .unwrap();
        }
    }

    // pub fn results(&self) -> &Receiver<DiskAssetIOResult> {
    //     self.thread_pool.as_ref().map(|x| &x.result_rx).unwrap()
    // }

    pub fn active_request_count(&self) -> usize {
        self.thread_pool.as_ref().unwrap().active_request_count()
    }
}

impl LoaderIO for DiskAssetIO {
    fn latest_build_hash(&self) -> CombinedBuildHash {
        self.build_hash
    }

    fn request_metadata(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        object_id: ObjectId,
        version: u32,
    ) {
        log::debug!("request_metadata {:?}", load_handle);

        let hash = self.manifest.asset_build_hashes.get(&object_id);
        if let Some(&hash) = hash {
            self.thread_pool
                .as_ref()
                .unwrap()
                .request_tx
                .send(DiskAssetIORequest::Metadata(DiskAssetIORequestMetadata {
                    load_handle,
                    object_id,
                    version,
                    hash,
                }))
                .unwrap();
        } else {
            self.tx
                .send(LoaderEvent::MetadataRequestComplete(
                    RequestMetadataResult {
                        object_id,
                        load_handle,
                        version,
                        //hash: 0,
                        result: Err(std::io::ErrorKind::NotFound.into()),
                    },
                ))
                .unwrap();
        }

        // self.tx.send(LoaderEvent::MetadataRequestComplete(RequestMetadataResult {
        //     load_handle,
        //     object_id,
        //     version,
        //     result: Ok(ObjectMetadata {
        //         dependencies: vec![],
        //         subresource_count: 0,
        //         asset_type: AssetTypeId(*uuid::Uuid::parse_str("1a4dde10-5e60-483d-88fa-4f59752e4524").unwrap().as_bytes())
        //     })
        // })).unwrap();
    }

    fn request_data(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        object_id: ObjectId,
        hash: u64,
        subresource: Option<u32>,
        version: u32,
    ) {
        log::debug!("request_data {:?}", load_handle);

        self.thread_pool
            .as_ref()
            .unwrap()
            .request_tx
            .send(DiskAssetIORequest::Data(DiskAssetIORequestData {
                object_id,
                load_handle,
                hash,
                version,
                subresource,
            }))
            .unwrap();

        // self.tx.send(LoaderEvent::DataRequestComplete(RequestDataResult {
        //     load_handle,
        //     object_id,
        //     subresource,
        //     version,
        //     result: Ok(ObjectData {
        //         data: vec![]
        //     })
        // })).unwrap();
    }
}

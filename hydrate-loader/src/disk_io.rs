use crate::loader::ArtifactData;
use crate::loader::{
    CombinedBuildHash, LoaderEvent, LoaderIO, ArtifactMetadata, RequestDataResult,
    RequestMetadataResult,
};
use crate::storage::IndirectIdentifier;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::LoadHandle;
use hydrate_base::{ArtifactId, AssetTypeId, ManifestFileEntry, ManifestFileJson};
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

struct DiskAssetIORequestMetadata {
    artifact_id: ArtifactId,
    load_handle: LoadHandle,
    version: u32,
    hash: u64,
}

struct DiskAssetIORequestData {
    artifact_id: ArtifactId,
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

                                // Read the data?

                                let path = hydrate_base::uuid_path::uuid_and_hash_to_path(&*root_path, msg.artifact_id.as_uuid(), msg.hash, "bf");
                                let mut reader = std::fs::File::open(path).unwrap();
                                let metadata = hydrate_base::BuiltArtifactMetadata::read_header(&mut reader).unwrap();


                                //let path = hydrate_model::uuid_path::uuid_and_hash_to_path(&*root_path, msg.artifact_id.as_uuid(), msg.hash, "bf");
                                log::trace!("Start metadata read {:?}", msg.artifact_id);
                                //let t0 = std::time::Instant::now();
                                // let result = std::fs::read(&path);
                                // //let t1 = std::time::Instant::now();
                                // match &result {
                                //     Ok(data) => {
                                //         //log::debug!("Read {:?} {:?} {} bytes in {}ms", artifact_id, subresource, data.len(), (t1 - t0).as_secs_f32() * 1000.0);
                                //     },
                                //     Err(e) => {
                                //         log::warn!("Failed to read metadata {:?} at path {:?}", msg.artifact_id, path);
                                //     }
                                // }

                                let metadata = ArtifactMetadata {
                                    dependencies: metadata.dependencies,
                                    asset_type: AssetTypeId(*metadata.asset_type.as_bytes()), //AssetTypeId(*uuid::Uuid::parse_str("1a4dde10-5e60-483d-88fa-4f59752e4524").unwrap().as_bytes()),
                                    hash: msg.hash,
                                };

                                log::trace!("read metadata {:?}", metadata);

                                result_tx.send(LoaderEvent::MetadataRequestComplete( RequestMetadataResult {
                                    artifact_id: msg.artifact_id,
                                    load_handle: msg.load_handle,
                                    //subresource: msg.subresource,
                                    //hash: msg.hash,
                                    version: msg.version,
                                    result: Ok(metadata)
                                })).unwrap();
                                active_request_count.fetch_sub(1, Ordering::Release);
                            },
                            DiskAssetIORequest::Data(msg) => {
                                let path = hydrate_base::uuid_path::uuid_and_hash_to_path(&*root_path, msg.artifact_id.as_uuid(), msg.hash, "bf");
                                let mut reader = std::fs::File::open(&path).unwrap();
                                let _metadata = hydrate_base::BuiltArtifactMetadata::read_header(&mut reader).unwrap();

                                let mut bytes = Vec::new();
                                use std::io::Read;
                                reader.read_to_end(&mut bytes).unwrap();

                                log::trace!("Start read {:?} {:?}", msg.artifact_id, msg.subresource);
                                //let t0 = std::time::Instant::now();
                                //let result = std::fs::read(&path);

                                let mut reader = std::fs::File::open(path).unwrap();
                                let mut length_bytes = [0u8; 8];
                                reader.read(&mut length_bytes).unwrap();
                                use std::io::Seek;
                                reader.seek(SeekFrom::Current(u64::from_le_bytes(length_bytes) as i64)).unwrap();
                                let mut data = Vec::default();
                                reader.read_to_end(&mut data).unwrap();

                                // This needs to skip the header?

                                //let t1 = std::time::Instant::now();
                                //log::debug!("Read {:?} {:?} {} bytes in {}ms", artifact_id, subresource, data.len(), (t1 - t0).as_secs_f32() * 1000.0);

                                result_tx.send(LoaderEvent::DataRequestComplete(RequestDataResult {
                                    artifact_id: msg.artifact_id,
                                    load_handle: msg.load_handle,
                                    subresource: msg.subresource,
                                    version: msg.version,
                                    //hash: msg.hash,
                                    result: Ok(ArtifactData {
                                        data
                                    })
                                })).unwrap();

                                active_request_count.fetch_sub(1, Ordering::Release);
                            }
                        }
                    },
                    recv(finish_rx) -> _msg => {
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
    active_request_count: Arc<AtomicUsize>,
}

impl DiskAssetIOThreadPool {
    fn new(
        root_path: Arc<PathBuf>,
        max_requests_in_flight: usize,
        result_tx: Sender<LoaderEvent>,
    ) -> Self {
        let (request_tx, request_rx) = crossbeam_channel::unbounded::<DiskAssetIORequest>();
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

    fn finish(self) {
        for worker_thread in &self.worker_threads {
            worker_thread.finish_tx.send(()).unwrap();
        }

        for worker_thread in self.worker_threads {
            worker_thread.join_handle.join().unwrap();
        }
    }
}

pub struct BuildManifest {
    pub artifact_lookup: HashMap<ArtifactId, ManifestFileEntry>,
    pub symbol_lookup: HashMap<String, ArtifactId>,
}

impl BuildManifest {
    fn load_from_file(
        manifest_dir_path: &Path,
        build_hash: CombinedBuildHash,
    ) -> BuildManifest {
        let file_name = format!("{:0>16x}.manifest", build_hash.0);
        let file_path = manifest_dir_path.join(file_name);
        let json_str = std::fs::read_to_string(file_path).unwrap();
        let manifest_file: ManifestFileJson = serde_json::from_str(&json_str).unwrap();

        let mut artifact_lookup = HashMap::default();
        let mut symbol_lookup = HashMap::default();
        for artifact in manifest_file.artifacts {
            if !artifact.symbol_name.is_empty() {
                let old = symbol_lookup.insert(artifact.symbol_name.clone(), artifact.artifact_id);
                assert!(old.is_none());
            }
            let old = artifact_lookup.insert(
                artifact.artifact_id,
                ManifestFileEntry {
                    artifact_id: artifact.artifact_id,
                    build_hash: u64::from_str_radix(&artifact.build_hash, 16).unwrap(),
                    symbol_name: artifact.symbol_name,
                    artifact_type: artifact.artifact_type,
                },
            );
            assert!(old.is_none());
        }

        BuildManifest {
            artifact_lookup,
            symbol_lookup,
        }

        // let mut asset_build_hashes = HashMap::default();
        //
        // let file_name = format!("{:0>16x}.manifest", build_hash.0);
        // let file_path = manifest_dir_path.join(file_name);
        // let file = std::fs::File::open(file_path).unwrap();
        // let buf_reader = std::io::BufReader::new(file);
        // for line in buf_reader.lines() {
        //     let line_str = line.unwrap().to_string();
        //     if line_str.is_empty() {
        //         continue;
        //     }
        //
        //     let separator = line_str.find(",").unwrap();
        //     let left = &line_str[..separator];
        //     let right = &line_str[(separator + 1)..];
        //
        //     let asset_id = u128::from_str_radix(left, 16).unwrap();
        //     let build_hash = u64::from_str_radix(right, 16).unwrap();
        //
        //
        //     asset_build_hashes.insert(ArtifactId(asset_id), build_hash);
        // }
        //
        // BuildManifest { asset_build_hashes }
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
}

impl LoaderIO for DiskAssetIO {
    fn latest_build_hash(&self) -> CombinedBuildHash {
        self.build_hash
    }

    fn resolve_indirect(
        &self,
        indirect_identifier: &IndirectIdentifier,
    ) -> Option<(ArtifactId, u64)> {
        match indirect_identifier {
            IndirectIdentifier::PathWithType(asset_path, asset_type) => {
                let artifact_id = self.manifest.symbol_lookup.get(asset_path)?;
                let metadata = self.manifest.artifact_lookup.get(&artifact_id)?;
                if *metadata.artifact_type.as_bytes() == asset_type.0 {
                    Some((metadata.artifact_id, metadata.build_hash))
                } else {
                    panic!(
                        "Tried to resolve artifact {:?} but it was an unexpected type {:?}",
                        indirect_identifier, metadata.artifact_type
                    );
                    //None
                }
            }
        }
    }

    fn request_metadata(
        &self,
        build_hash: CombinedBuildHash,
        load_handle: LoadHandle,
        artifact_id: ArtifactId,
        version: u32,
    ) {
        log::debug!("request_metadata {:?}", load_handle);
        assert_eq!(self.build_hash, build_hash);

        let hash = self
            .manifest
            .artifact_lookup
            .get(&artifact_id)
            .map(|x| x.build_hash);
        if let Some(hash) = hash {
            self.thread_pool
                .as_ref()
                .unwrap()
                .add_request(DiskAssetIORequest::Metadata(DiskAssetIORequestMetadata {
                    load_handle,
                    artifact_id,
                    version,
                    hash,
                }));
        } else {
            self.tx
                .send(LoaderEvent::MetadataRequestComplete(
                    RequestMetadataResult {
                        artifact_id,
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
        //     artifact_id,
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
        artifact_id: ArtifactId,
        hash: u64,
        subresource: Option<u32>,
        version: u32,
    ) {
        log::debug!("request_data {:?}", load_handle);
        assert_eq!(self.build_hash, build_hash);

        self.thread_pool
            .as_ref()
            .unwrap()
            .add_request(DiskAssetIORequest::Data(DiskAssetIORequestData {
                artifact_id,
                load_handle,
                hash,
                version,
                subresource,
            }));

        // self.tx.send(LoaderEvent::DataRequestComplete(RequestDataResult {
        //     load_handle,
        //     artifact_id,
        //     subresource,
        //     version,
        //     result: Ok(ArtifactData {
        //         data: vec![]
        //     })
        // })).unwrap();
    }
}

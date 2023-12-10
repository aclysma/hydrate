use crate::loader::ArtifactData;
use crate::loader::{
    ArtifactMetadata, CombinedBuildHash, LoaderEvent, LoaderIO, RequestDataResult,
    RequestMetadataResult,
};
use crate::storage::IndirectIdentifier;
use crate::ArtifactTypeId;
use crossbeam_channel::{Receiver, Sender};
use hydrate_base::hashing::HashMap;
use hydrate_base::{ArtifactId, ArtifactManifestData, DebugManifestFileJson};
use hydrate_base::{LoadHandle, StringHash};
use std::io::{BufRead, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use uuid::Uuid;

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
        thread_index: usize,
    ) -> Self {
        let (finish_tx, finish_rx) = crossbeam_channel::bounded(1);
        let join_handle = std::thread::Builder::new().name("IO Thread".into()).spawn(move || {
            profiling::register_thread!(&format!("DiskAssetIOWorkerThread {}", thread_index));
            loop {
                crossbeam_channel::select! {
                    recv(request_rx) -> msg => {
                        match msg.unwrap() {
                            DiskAssetIORequest::Metadata(msg) => {
                                profiling::scope!("DiskAssetIORequest::Metadata");
                                log::trace!("Start metadata read {:?}", msg.artifact_id);
                                let path = hydrate_base::uuid_path::uuid_and_hash_to_path(&*root_path, msg.artifact_id.as_uuid(), msg.hash, "bf");
                                let mut reader = std::fs::File::open(path).unwrap();
                                let header_data = hydrate_base::BuiltArtifactHeaderData::read_header(&mut reader).unwrap();

                                let metadata = ArtifactMetadata {
                                    dependencies: header_data.dependencies,
                                    asset_type: ArtifactTypeId::from_uuid(header_data.asset_type), //AssetTypeId(*uuid::Uuid::parse_str("1a4dde10-5e60-483d-88fa-4f59752e4524").unwrap().as_bytes()),
                                    hash: msg.hash,
                                };

                                log::trace!("read metadata {:?}", metadata);

                                result_tx.send(LoaderEvent::MetadataRequestComplete( RequestMetadataResult {
                                    artifact_id: msg.artifact_id,
                                    load_handle: msg.load_handle,
                                    version: msg.version,
                                    result: Ok(metadata)
                                })).unwrap();
                                active_request_count.fetch_sub(1, Ordering::Release);
                            },
                            DiskAssetIORequest::Data(msg) => {
                                profiling::scope!("DiskAssetIORequest::Data");
                                log::trace!("Start read {:?} {:?}", msg.artifact_id, msg.subresource);

                                let path = hydrate_base::uuid_path::uuid_and_hash_to_path(&*root_path, msg.artifact_id.as_uuid(), msg.hash, "bf");
                                let mut reader = std::fs::File::open(&path).unwrap();
                                let _header_data = hydrate_base::BuiltArtifactHeaderData::read_header(&mut reader).unwrap();

                                use std::io::Read;

                                let mut reader = std::fs::File::open(path).unwrap();
                                let mut length_bytes = [0u8; 8];
                                reader.read(&mut length_bytes).unwrap();
                                use std::io::Seek;
                                reader.seek(SeekFrom::Current(u64::from_le_bytes(length_bytes) as i64)).unwrap();
                                let mut data = Vec::default();
                                {
                                    profiling::scope!("std::fs::File::read_to_end");
                                    reader.read_to_end(&mut data).unwrap();
                                }

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
        for thread_index in 0..max_requests_in_flight {
            let worker = DiskAssetIOWorkerThread::new(
                root_path.clone(),
                request_rx.clone(),
                result_tx.clone(),
                active_request_count.clone(),
                thread_index,
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
    pub artifact_lookup: HashMap<ArtifactId, ArtifactManifestData>,
    pub symbol_lookup: HashMap<u128, ArtifactId>,
}

impl BuildManifest {
    fn load_from_file(
        manifest_dir_path: &Path,
        build_hash: CombinedBuildHash,
    ) -> BuildManifest {
        //
        // Load release manifest data, this must exist and load correctly
        //
        let mut build_manifest = {
            profiling::scope!("Load release manifest data");

            let mut asset_build_hashes = HashMap::default();

            let file_name = format!("{:0>16x}.manifest_release", build_hash.0);
            let file_path = manifest_dir_path.join(file_name);
            let file = std::fs::File::open(file_path).unwrap();
            let buf_reader = std::io::BufReader::new(file);

            let mut artifact_lookup = HashMap::default();
            let mut symbol_lookup = HashMap::default();

            for line in buf_reader.lines() {
                let line_str = line.unwrap().to_string();
                if line_str.is_empty() {
                    continue;
                }

                let fragments: Vec<_> = line_str.split(",").into_iter().collect();

                let artifact_id =
                    ArtifactId::from_u128(u128::from_str_radix(fragments[0], 16).unwrap());
                let build_hash = u64::from_str_radix(fragments[1], 16).unwrap();
                let artifact_type =
                    Uuid::from_u128(u128::from_str_radix(fragments[2], 16).unwrap());
                let symbol_hash_u128 = u128::from_str_radix(fragments[3], 16).unwrap();

                let symbol_hash = if symbol_hash_u128 != 0 {
                    let old = symbol_lookup.insert(symbol_hash_u128, artifact_id);
                    assert!(old.is_none());
                    Some(StringHash::from_hash(symbol_hash_u128))
                } else {
                    None
                };

                let old = artifact_lookup.insert(
                    artifact_id,
                    ArtifactManifestData {
                        artifact_id,
                        build_hash,
                        symbol_hash,
                        artifact_type,
                        debug_name: Default::default(),
                    },
                );
                assert!(old.is_none());

                asset_build_hashes.insert(artifact_id, build_hash);
            }

            BuildManifest {
                artifact_lookup,
                symbol_lookup,
            }
        };

        //
        // Load manifest debug data, it's ok if these files don't exist. This is just additive to
        // the critical data provided by the release manifest
        //
        {
            let file_name = format!("{:0>16x}.manifest_debug", build_hash.0);
            let file_path = manifest_dir_path.join(file_name);
            if file_path.exists() {
                profiling::scope!("Load debug manifest data");
                log::info!("Manifest debug data found");
                let json_str = std::fs::read_to_string(file_path).unwrap();
                let manifest_file: DebugManifestFileJson = {
                    profiling::scope!("serde_json::from_str");
                    serde_json::from_str(&json_str).unwrap()
                };

                for debug_manifest_entry in manifest_file.artifacts {
                    let manifest_entry = build_manifest
                        .artifact_lookup
                        .get_mut(&debug_manifest_entry.artifact_id)
                        .unwrap();

                    assert_eq!(manifest_entry.artifact_id, debug_manifest_entry.artifact_id);
                    assert_eq!(
                        manifest_entry.artifact_type,
                        debug_manifest_entry.artifact_type
                    );
                    let debug_manifest_build_hash =
                        u64::from_str_radix(&debug_manifest_entry.build_hash, 16).unwrap();
                    assert_eq!(manifest_entry.build_hash, debug_manifest_build_hash);

                    if debug_manifest_entry.symbol_name.is_empty() {
                        assert_eq!(manifest_entry.symbol_hash, None);
                    } else {
                        let debug_manifest_symbol_hash =
                            StringHash::from_runtime_str(&debug_manifest_entry.symbol_name);
                        assert_eq!(
                            manifest_entry.symbol_hash.as_ref().unwrap().hash(),
                            debug_manifest_symbol_hash.hash()
                        );
                        manifest_entry.symbol_hash = Some(debug_manifest_symbol_hash);
                    }

                    manifest_entry.debug_name = Some(Arc::new(debug_manifest_entry.debug_name));
                }
            } else {
                log::info!(
                    "No manifest debug data found, less debug info will be available at runtime"
                )
            }
        }

        build_manifest
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

    fn manifest_entry(
        &self,
        artifact_id: ArtifactId,
    ) -> Option<&ArtifactManifestData> {
        self.manifest.artifact_lookup.get(&artifact_id)
    }

    fn resolve_indirect(
        &self,
        indirect_identifier: &IndirectIdentifier,
    ) -> Option<&ArtifactManifestData> {
        let (artifact_id, asset_type) = match indirect_identifier {
            IndirectIdentifier::SymbolWithType(asset_path, asset_type) => {
                let artifact_id = self.manifest.symbol_lookup.get(&asset_path.hash())?;
                (*artifact_id, *asset_type)
            }
        };

        let metadata = self.manifest.artifact_lookup.get(&artifact_id)?;
        if metadata.artifact_type == asset_type.0 {
            Some(metadata)
        } else {
            panic!(
                "Tried to resolve artifact {:?} but it was an unexpected type {:?}",
                indirect_identifier, metadata.artifact_type
            );
            //None
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
            // Queue up the work
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
            // Return the failure immediately
            self.tx
                .send(LoaderEvent::MetadataRequestComplete(
                    RequestMetadataResult {
                        artifact_id,
                        load_handle,
                        version,
                        result: Err(std::io::ErrorKind::NotFound.into()),
                    },
                ))
                .unwrap();
        }
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

        // Queue up the work
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
    }
}

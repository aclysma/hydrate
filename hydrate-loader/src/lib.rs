
mod disk_io;


//States from distill's loader
// /// Indeterminate state - may transition into a load, or result in removal if ref count is == 0
// None = 0,
// /// The load operation needs metadata to progress
// WaitingForMetadata = 1,
// /// Metadata is being fetched for the load operation
// RequestingMetadata = 2,
// /// Dependencies are requested for loading
// RequestDependencies = 3,
// /// Waiting for dependencies to complete loading
// WaitingForDependencies = 4,
// /// Waiting for asset data to be fetched
// WaitingForData = 5,
// /// Asset data is being fetched
// RequestingData = 6,
// /// Engine systems are loading asset
// LoadingAsset = 7,
// /// Engine systems have loaded asset, but the asset is not committed.
// /// This state is only reached when AssetVersionLoad.auto_commit == false.
// LoadedUncommitted = 8,
// /// Asset is loaded and ready to use
// Loaded = 9,
// /// Asset should be unloaded
// UnloadRequested = 10,
// /// Asset is being unloaded by engine systems
// Unloading = 11,

// metadata

// /// Serializable metadata for an asset.
// /// Stored in .meta files and metadata DB.
// #[derive(Debug, Clone, Hash, Default)]
// #[cfg_attr(feature = "serde-1", derive(Serialize, Deserialize))]
// pub struct AssetMetadata {
//     /// UUID for the asset to uniquely identify it
//     pub id: AssetUuid,
//     /// Search tags are used by asset tooling to search for the imported asset
//     pub search_tags: Vec<(String, Option<String>)>,
//     /// The referenced build pipeline is invoked when a build artifact is requested for the imported asset
//     pub build_pipeline: Option<AssetUuid>,
//     /// The latest artifact produced when importing this asset
//     pub artifact: Option<ArtifactMetadata>,
// }
//
// /// 64-bit hash of the inputs that would produce a given asset artifact
// #[derive(Debug, Copy, Clone, Hash, Default)]
// #[cfg_attr(feature = "serde-1", derive(Serialize, Deserialize))]
// #[cfg_attr(feature = "serde-1", serde(transparent))]
// pub struct ArtifactId(pub u64);
//
// /// Serializable metadata for an artifact.
// /// Stored in .meta files and metadata DB.
// #[derive(Debug, Clone, Hash, Default)]
// #[cfg_attr(feature = "serde-1", derive(Serialize, Deserialize))]
// pub struct ArtifactMetadata {
//     /// Hash that identifies this artifact
//     pub id: ArtifactId,
//     /// UUID for this artifact's asset
//     pub asset_id: AssetUuid,
//     /// Build dependencies will be included in the Builder arguments when building an asset
//     pub build_deps: Vec<AssetRef>,
//     /// Load dependencies are guaranteed to load before this asset by the Loader
//     pub load_deps: Vec<AssetRef>,
//     /// Type of compression used to compress this artifact
//     pub compression: CompressionType,
//     /// Size of this artifact in bytes when compressed
//     pub compressed_size: Option<u64>,
//     /// Size of this artifact in bytes when serialized and uncompressed
//     pub uncompressed_size: Option<u64>,
//     /// The UUID of the artifact's Rust type
//     pub type_id: AssetTypeId,
// }


use std::cmp::max;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use hydrate_base::hashing::HashMap;
use hydrate_model::{HashSet, ObjectId};
use crate::disk_io::DiskAssetIO;


// Based on distill's AssetStorage
trait AssetStorage {
    // prepare asset
    // - we get a callback that gives us the data for the asset, we prepare it and notify when it is
    //   prepared. It is "uncommitted" at this point - meaning most game logic won't see the the
    //   effects of this call yet (just game logic involved in preparing the data)

    // commit asset
    // - we get a callback that tells us to "activate" the prepared asset, such that future requests
    //   for the asset return the most-recently prepared data

    // free
    // - we can unload the prepared and committed data for this asset
}

trait AssetProvider {
    // get
    // get_version
    // get_asset_with_version
}





struct IOCommandUpdateAsset {
    asset_id: ObjectId,
    version: u64,
    data: Vec<u8>,
}

struct IOCommandCommitAsset {
    asset_id: ObjectId,
    version: u64,
}

enum IOCommand {
    Update(IOCommandUpdateAsset),
    Commit(IOCommandCommitAsset),
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
    build_hash: u64
}

// Opens a TOC file and reads contents
fn read_toc(path: &Path) -> BuildToc {
    let data = std::fs::read_to_string(path).unwrap();
    let build_hash = u64::from_str_radix(&data, 16).unwrap();
    BuildToc {
        build_hash
    }
}
//
// struct BuildManifest {
//     asset_build_hashes: HashMap<ObjectId, u64>
// }
//
// fn load_manifest(manifest_dir_path: &Path, build_hash: u64) -> BuildManifest {
//     let mut asset_build_hashes = HashMap::default();
//
//     let file_name = format!("{:0>16x}.manifest", build_hash);
//     let file_path = manifest_dir_path.join(file_name);
//     let file = std::fs::File::open(file_path).unwrap();
//     let buf_reader = std::io::BufReader::new(file);
//     for line in buf_reader.lines() {
//         let line_str = line.unwrap().to_string();
//         if line_str.is_empty() {
//             continue;
//         }
//
//         let separator = line_str.find(",").unwrap();
//         let left = &line_str[..separator];
//         let right = &line_str[(separator+1)..];
//
//         let asset_id = u128::from_str_radix(left, 16).unwrap();
//         let build_hash = u64::from_str_radix(right, 16).unwrap();
//
//         asset_build_hashes.insert(ObjectId(asset_id), build_hash);
//     }
//
//     BuildManifest {
//         asset_build_hashes
//     }
// }

// Asset states can be:
// Unloaded, not subscribed
// Unloaded, subscribed, not requested yet
// Unloaded, subscribed, request in flight
// Unloaded, subscribed, request ready to load
// Loaded, unsubscribed

// States:
// Loaded | Unloaded
// Subscribed | Not Subscribed
// No request in flight | Request in flight | Request ready to load
//
// Unloaded, Not Subscribed, No Request in Flight -> Do nothing
// Unloaded, Not Subscribed, Request in Flight -> We could cancel the request, otherwise wait until ready to load
// Unloaded, Not Subscribed, Request Ready to Load -> Drop the data
// Unloaded, Subscribed, No Request in Flight -> Kick off the request
// Unloaded, Subscribed, Request in Flight -> Wait until ready to load
// Unloaded, Subscribed, Request Ready to Load -> Load the data
// Loaded, Subscribed, No Request in Flight -> Do nothing
// Loaded, Subscribed, Request in Flight -> Invalid
// Loaded, Subscribed, Request Ready to Load -> Invalid
// Loaded, Not Subscribed, No Request in Flight -> Drop the data
// Loaded, Not Subscribed, Request in Flight ->  Invalid
// Loaded, Not Subscribed, Request Ready to Load -> Invalid
//
// Request can only be in flight if we are not loaded
//
// Unloaded, Unsubscribed, No Request in flight
// Unloaded, Subscribed, No Request
// - potentially bail back to unloaded/unsubscribed
// Unloaded, Subscribed, Request in Flight
// - potentially bail back to unloaded, but the request needs to be cancelled/completed
// Unloaded, Subscribed, Request Ready to Load
// - potentially bail back to unloaded
// Loaded, Subscribed, No Request in flight
// Loaded, Unsubscribed, No Request in flight
// Unloaded, Unsubscribed, No Request in flight
//
// how to handle updates?
// (disk) <-> (request queue) <-> (version handling state machine?) <-> (streaming priority manager) <-> (asset handle tracker)
//
// streaming...
// - list of things we want to load, with score of value in having loaded
// - list of thigns that are loaded, with score of value in having loaded
// - by default, load requests are mandatory (max score?)
// - requests can be both assets and asset sub-resources
// -
//
// how to handle updates
// - we have some code that works ignoring the updates
// - then another thing that is lower priority that tracks the additional thing to be loaded
// - injects the differences to the main version handling state machine
// - how to handle handles being allocated while streaming in updates?
// - how to handle an update arriving faster than the original asset version?
// - treat different versions as different objects?


/*

trait AssetIO {
    fn subscribe(&mut self, object_id: ObjectId);
    fn unsubscribe(&mut self, object_id: ObjectId);

    // A stream of steps that, as long as we drain the queue, will leave us in a valid state
    fn take_load_command(&mut self, object_id: ObjectId) -> IOCommand;
}

struct AssetIOState {
    loaded_hash: u64,
}

struct DiskAssetIO {
    subscribed: HashSet<ObjectId>,
    path: PathBuf,
}

impl DiskAssetIO {
    pub fn new(path: PathBuf) -> Self {
        DiskAssetIO {
            subscribed,
            path
        }
    }
}

impl AssetIO for DiskAssetIO {
    fn subscribe(&mut self, object_id: ObjectId) {
        let was_subscribed = !self.subscribed.insert(object_id);

        // do we push some work into a queue?

        assert!(!was_subscribed);
    }

    fn unsubscribe(&mut self, object_id: ObjectId) {
        let was_subscribed = self.subscribed.remove(&object_id);

        assert!(was_subscribed);
    }

    fn take_load_command(&mut self, object_id: ObjectId) -> IOCommand {
        todo!()
    }
}


*/


// Create an Asset handle
// Ref-count tracking causes us to call subscribe/unsubscribe for the asset
// When we subscribe, we look up all the things we need to load
// Keep track of what we have subscribed to
// Provide all the assets that the subscriptions imply we need
// The loaded data for the asset will be built up in the IO and then taken by the game loader
//


pub struct Loader {
    //build_root_path: PathBuf,
    asset_io: DiskAssetIO,
}

impl Loader {
    pub fn new(build_data_root_path: PathBuf) -> Result<Self, String> {
        //let asset_io = DiskAssetIO::new(build_data_root_path.clone());

        let max_toc_path = find_latest_toc(&build_data_root_path.join("toc"));
        let max_toc_path = max_toc_path.ok_or_else(|| "Could not find TOC file".to_string())?;
        let build_toc = read_toc(&max_toc_path);
        let asset_io = DiskAssetIO::new(build_data_root_path, build_toc.build_hash);

        let t0 = std::time::Instant::now();
        for (k, v) in &asset_io.manifest().asset_build_hashes {
            asset_io.request_data(*k, None);
        }

        while asset_io.active_request_count() > 0 {
            //std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let t1 = std::time::Instant::now();
        log::info!("Loaded everything in {}ms", (t1 - t0).as_secs_f32() * 1000.0);

        Ok(Loader {
            //build_root_path,
            asset_io
        })
    }

    pub fn load_asset(&self, object_id: ObjectId) {
        self.asset_io.request_data(object_id, None);


        // Figure out what objects need to be loaded (i.e. dependerncies)
        // Issue disk IO requests
        // Wait until they are completed
        // Possibly some extra on-load-complete stuff
    }
}

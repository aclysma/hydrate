use std::path::PathBuf;
use uuid::Uuid;

mod error;
mod file_watcher;

struct AssetImportState {
    // file watcher
    // dirty/renamed file queues
    //
}

struct AssetPackDef {
    // local path
    // remote server
    // source control
    id: uuid::Uuid,
    source_root_paths: Vec<PathBuf>,
    db_path: PathBuf,
}

struct AssetEngineConfig {}

struct AssetEngine {
    // config
    config: AssetEngineConfig,
    // assset sources
    // importing
    // processing
    // serving
    // loading
}

impl AssetEngine {
    fn new(config: AssetEngineConfig) -> Self {
        AssetEngine { config }
    }

    fn mount_asset_pack(
        &mut self,
        asset_pack_def: AssetPackDef,
    ) {
    }
}

pub fn demo() {
    let mut asset_engine = AssetEngine::new(AssetEngineConfig {});

    asset_engine.mount_asset_pack(AssetPackDef {
        id: Uuid::parse_str("86d5fd3f-3159-4295-8a90-9ef8077675bf").unwrap(),
        db_path: PathBuf::default(),
        source_root_paths: Vec::default(),
    });
}

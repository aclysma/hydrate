use std::hash::{Hash, Hasher};
use crate::{ArtifactId, StringHash};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Used to store debug manifest data. It's not needed for the game to function but can be used in
/// addition to the release manifest to get helpful debug info
#[derive(Serialize, Deserialize)]
pub struct DebugArtifactManifestDataJson {
    pub artifact_id: ArtifactId,
    // stored as a string so we can encoded as hex
    pub build_hash: String,
    pub combined_build_hash: String,
    pub symbol_name: String,
    // stored as a string so we can encoded as hex. The hash isn't really needed but it's nice
    // to have in the file for looking up a hash while debugging
    pub symbol_hash: String,
    pub artifact_type: Uuid,
    pub debug_name: String,
}

/// Used to store debug manifest data. It's not needed for the game to function but can be used in
/// addition to the release manifest to get helpful debug info
#[derive(Serialize, Deserialize, Default)]
pub struct DebugManifestFileJson {
    pub artifacts: Vec<DebugArtifactManifestDataJson>,
}

/// Metadata about the asset that is loaded in memory at all times. May include extra debug data.
/// This is just enough information to know if an asset exists and know where to get more info
/// about it. Some data needed for load is encoded in the asset itself and not in memory until the
/// asset is requested and must be fetched from disk.
pub struct ArtifactManifestData {
    pub artifact_id: ArtifactId,
    pub simple_build_hash: u64,
    pub combined_build_hash: u64,
    // If the artifact cannot be addressed by symbol, this will be None
    // Even if the symbol is Some, the string in the string hash might not be present. It's only
    // a debugging aid
    pub symbol_hash: Option<StringHash>,
    pub artifact_type: Uuid,
    // The debug name might not be available
    pub debug_name: Option<Arc<String>>,
}

// No real reason this limit needs to exist, just don't want to read corrupt data and try to
// allocate or load based on corrupt data. This is larger than a header is actually expected
// to be.
const MAX_HEADER_SIZE: usize = 1024 * 1024;

/// Data encoded into the asset. This is necessary for loading but is not available in memory at
/// all times. The load process will fetch this from the top of the built artifact data.
/// This is specifically designed to read the minimum amount of info out of the file.

//TODO: Could use B3F here, but this is working fine for now.
//TODO: Probably don't strictly need bincode either
#[derive(Debug, Serialize, Deserialize)]
pub struct BuiltArtifactHeaderData {
    pub dependencies: Vec<ArtifactId>,
    pub asset_type: Uuid, // size?
}

impl Hash for BuiltArtifactHeaderData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut dependencies_hash = 0;
        for dependency in &self.dependencies {
            dependencies_hash ^= dependency.0.as_u128();
        }

        dependencies_hash.hash(state);
        self.asset_type.hash(state);
    }
}

impl BuiltArtifactHeaderData {
    pub fn write_header<T: std::io::Write>(
        &self,
        writer: &mut T,
    ) -> std::io::Result<()> {
        let serialized = bincode::serialize(self).unwrap();
        let bytes = serialized.len();
        // Just
        assert!(bytes <= MAX_HEADER_SIZE);
        writer.write(&bytes.to_le_bytes())?;
        writer.write(&serialized)?;

        Ok(())
    }

    pub fn read_header<T: std::io::Read>(
        reader: &mut T
    ) -> std::io::Result<BuiltArtifactHeaderData> {
        let mut length_bytes = [0u8; 8];
        reader.read(&mut length_bytes)?;
        let length = usize::from_le_bytes(length_bytes);
        assert!(length <= MAX_HEADER_SIZE);

        let mut read_buffer = vec![0u8; length];
        reader.read_exact(&mut read_buffer).unwrap();

        let metadata = bincode::deserialize(&read_buffer).unwrap();
        Ok(metadata)
    }
}

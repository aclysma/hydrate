use crate::{ArtifactId, StringHash};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO: We can split up the debug/non-debug data. The debug format could be something like json
// that's debug-friendly and contains all the data. The runtime-only could be a mean-and-lean bincode
// or other binary format
#[derive(Serialize, Deserialize)]
pub struct DebugManifestFileEntryJson {
    pub artifact_id: ArtifactId,
    // stored as a string so we can encoded as hex
    pub build_hash: String,
    pub symbol_name: String,
    // stored as a string so we can encoded as hex. The hash isn't really needed but it's nice
    // to have in the file for looking up a hash while debugging
    pub symbol_hash: String,
    pub artifact_type: Uuid,
    pub debug_name: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DebugManifestFileJson {
    pub artifacts: Vec<DebugManifestFileEntryJson>,
}

pub struct ManifestFileEntry {
    pub artifact_id: ArtifactId,
    pub build_hash: u64,
    pub symbol_hash: StringHash,
    pub artifact_type: Uuid,
    pub debug_name: String,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct BuiltArtifactMetadata {
    pub dependencies: Vec<ArtifactId>,
    pub asset_type: Uuid, // size?
}

impl BuiltArtifactMetadata {
    pub fn write_header<T: std::io::Write>(
        &self,
        writer: &mut T,
    ) -> std::io::Result<()> {
        // writer.write(&(self.dependencies.len() as u32).to_le_bytes())?;
        // for dependency in &self.dependencies {
        //     writer.write(&dependency.0.to_le_bytes())?;
        // }
        //
        // writer.write(&self.subresource_count.to_le_bytes())?;
        // writer.write(&self.asset_type.as_u128().to_le_bytes())?;

        let serialized = bincode::serialize(self).unwrap();
        let bytes = serialized.len();
        writer.write(&bytes.to_le_bytes())?;
        writer.write(&serialized)?;

        Ok(())
    }

    pub fn read_header<T: std::io::Read>(reader: &mut T) -> std::io::Result<BuiltArtifactMetadata> {
        // let mut buffer = [0; 16];
        // reader.read(&mut buffer[0..4])?;
        // let count = u32::from_le_bytes(&buffer[0..4]);
        // let mut dependencies = Vec::with_capacity(count as usize);
        // for _ in 0..count {
        //     dependencies.push(ArtifactId(reader.read_u128()?));
        // }
        //
        // let subresource_count = reader.read_u32()?;
        // let asset_type = Uuid::from_u128(reader.read_u128()?);

        let mut length_bytes = [0u8; 8];
        reader.read(&mut length_bytes)?;
        //let length = usize::from_le_bytes(length_bytes);

        let metadata = bincode::deserialize_from(reader).unwrap();
        Ok(metadata)

        // Ok(BuiltArtifactMetadata {
        //     dependencies,
        //     subresource_count,
        //     asset_type
        // })
    }
}

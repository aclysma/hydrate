use std::hash::{Hash, Hasher};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use hydrate_base::b3f;
use hydrate_data::json_storage::{SingleObjectJson};
use hydrate_data::{SchemaSet, SingleObject};
use crate::PipelineResult;

// No real reason this limit needs to exist, just don't want to read corrupt data and try to
// allocate or load based on corrupt data. This is larger than a header is actually expected
// to be.
const MAX_HEADER_SIZE: usize = 256;

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct ImportDataMetadata {
    pub source_file_modified_timestamp: u64,
    pub source_file_size: u64,
    pub import_data_contents_hash: u64,
}

impl ImportDataMetadata {
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

    pub fn read_header<T: std::io::Read>(reader: &mut T) -> std::io::Result<ImportDataMetadata> {
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

pub struct SingleObjectWithMetadata {
    pub single_object: SingleObject,
    pub metadata: ImportDataMetadata,
}

#[profiling::function]
pub fn load_import_metadata_from_b3f<T: std::io::Read + std::io::Seek>(
    data: &mut T,
) -> PipelineResult<ImportDataMetadata> {
    // First check that the file has the expected headers
    let b3f = hydrate_base::b3f::B3FReader::new(data)?.ok_or("Not a B3F file")?;
    assert_eq!(b3f.file_tag_as_u8(), b"HYIF");
    assert_eq!(b3f.version(), 1);

    let metadata = b3f.read_block(data, 0)?;
    let mut buf_reader = std::io::BufReader::new(metadata.as_slice());
    let metadata = ImportDataMetadata::read_header(&mut buf_reader)?;

    Ok(metadata)
}

#[profiling::function]
pub fn load_import_data_from_b3f<T: std::io::Read + std::io::Seek>(
    schema_set: &SchemaSet,
    data: &mut T,
) -> PipelineResult<SingleObjectWithMetadata> {
    // First check that the file has the expected headers
    let b3f = hydrate_base::b3f::B3FReader::new(data)?.ok_or("Not a B3F file")?;
    assert_eq!(b3f.file_tag_as_u8(), b"HYIF");
    assert_eq!(b3f.version(), 1);

    //
    // The first block is import file metadata in binary
    //
    let metadata = b3f.read_block(data, 0)?;
    //TODO: contents hash here? modified timestamp here?
    let mut buf_reader = std::io::BufReader::new(metadata.as_slice());
    let metadata = ImportDataMetadata::read_header(&mut buf_reader)?;

    //
    // The second block is UTF-8 json
    //
    let json_block = &b3f.read_block(data, 1)?;
    let json = std::str::from_utf8(&json_block).unwrap();

    //
    // Append remaining blocks to the buffers list. Put a placeholder buffer in for index 0
    // as that is where the json was stored
    //
    let mut buffers = vec![Arc::new(Vec::default())];
    for i in 1..b3f.block_count() {
        buffers.push(Arc::new(b3f.read_block(data, i)?));
    }

    // Parse the json to reconstruct the property data
    let stored_object: SingleObjectJson = {
        profiling::scope!("serde_json::from_str");
        serde_json::from_str(json).unwrap()
    };

    let single_object = {
        profiling::scope!("SingleObjectJson::to_single_object");
        stored_object.to_single_object(schema_set, &mut Some(buffers))
    };

    //let contents_hash = stored_object.contents_hash;
    Ok(SingleObjectWithMetadata {
        single_object,
        metadata,
    })
}

#[profiling::function]
pub fn save_single_object_to_b3f<W: std::io::Write>(
    write: W,
    object: &SingleObject,
    metadata: &ImportDataMetadata,
) {
    // Fill with two empty buffers that are placeholders for the metadata and the json portion of the
    // import data
    let mut buffers = Some(vec![Arc::new(Vec::default()), Arc::new(Vec::default())]);
    let single_object = SingleObjectJson::new(object, &mut buffers);

    // Encode the object to json
    let single_object_json = {
        profiling::scope!("serde_json::to_string_pretty");
        serde_json::to_string_pretty(&single_object).unwrap()
    };

    let mut buffers = buffers.unwrap();

    //
    // Store the binary header in buffer index 0
    //
    let mut hasher = siphasher::sip::SipHasher::default();
    // This includes schema, all other contents of the asset
    object.hash(&mut hasher);
    let contents_hash = hasher.finish();
    //TODO: Write this + import file size + import file modified time
    let mut data = Vec::default();
    metadata.write_header(&mut data).unwrap();
    buffers[0] = Arc::new(data);

    //
    // Store the json in buffer index 1
    //
    buffers[1] = Arc::new(single_object_json.into_bytes());

    //
    // Store the remaining import data buffers into subsequent blocks
    //
    let mut b3f_writer = b3f::B3FWriter::new_from_u8_tag(*b"HYIF", 1);
    for buffer in &buffers {
        b3f_writer.add_block(buffer);
    }

    b3f_writer.write(write)
}
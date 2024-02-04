use crate::PipelineResult;
use hydrate_base::b3f;
use hydrate_base::b3f::B3FReader;
use hydrate_data::json_storage::SingleObjectJson;
use hydrate_data::{SchemaSet, SingleObject};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;

// No real reason this limit needs to exist, just don't want to read corrupt data and try to
// allocate or load based on corrupt data. This is larger than a header is actually expected
// to be.
const MAX_HEADER_SIZE: usize = 256;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct ImportDataMetadata {
    pub source_file_modified_timestamp: u64,
    pub source_file_size: u64,
    pub import_data_contents_hash: u64,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct ImportDataHeader {
    metadata: ImportDataMetadata,
}

impl ImportDataHeader {
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

    pub fn read_header<T: std::io::Read>(reader: &mut T) -> std::io::Result<ImportDataHeader> {
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

//
// The first block is import file metadata in binary
//
fn read_header<T: std::io::Read + std::io::Seek>(
    b3f: &B3FReader,
    data: &mut T,
) -> PipelineResult<ImportDataHeader> {
    let metadata = b3f.read_block(data, 0)?;
    let mut buf_reader = std::io::BufReader::new(metadata.as_slice());
    let metadata = ImportDataHeader::read_header(&mut buf_reader)?;
    Ok(metadata)
}
//
// The second block is the default asset as UTF-8 json
//
fn read_default_asset<T: std::io::Read + std::io::Seek>(
    b3f: &B3FReader,
    data: &mut T,
    schema_set: &SchemaSet,
) -> PipelineResult<SingleObject> {
    let default_asset_block = &b3f.read_block(data, 1)?;
    let default_asset_str = std::str::from_utf8(&default_asset_block).unwrap();

    let default_asset_object_json: SingleObjectJson = {
        profiling::scope!("serde_json::from_str");
        serde_json::from_str(default_asset_str).unwrap()
    };

    let default_asset = {
        profiling::scope!("SingleObjectJson::to_single_object");
        default_asset_object_json.to_single_object(schema_set, &mut None)
    };
    Ok(default_asset)
}

#[profiling::function]
pub fn load_import_metadata_from_b3f<T: std::io::Read + std::io::Seek>(
    data: &mut T
) -> PipelineResult<ImportDataMetadata> {
    // First check that the file has the expected headers
    let b3f = B3FReader::new(data)?.ok_or("Not a B3F file")?;
    assert_eq!(b3f.file_tag_as_u8(), b"HYIF");
    assert_eq!(b3f.version(), 1);

    let header = read_header(&b3f, data)?;
    Ok(header.metadata)
}

#[profiling::function]
pub fn load_default_asset_from_b3f<T: std::io::Read + std::io::Seek>(
    schema_set: &SchemaSet,
    data: &mut T,
) -> PipelineResult<SingleObject> {
    // First check that the file has the expected headers
    let b3f = B3FReader::new(data)?.ok_or("Not a B3F file")?;
    assert_eq!(b3f.file_tag_as_u8(), b"HYIF");
    assert_eq!(b3f.version(), 1);

    read_default_asset(&b3f, data, schema_set)
}

#[profiling::function]
pub fn load_import_data_from_b3f<T: std::io::Read + std::io::Seek>(
    schema_set: &SchemaSet,
    data: &mut T,
) -> PipelineResult<SingleObjectWithMetadata> {
    // First check that the file has the expected headers
    let b3f = B3FReader::new(data)?.ok_or("Not a B3F file")?;
    assert_eq!(b3f.file_tag_as_u8(), b"HYIF");
    assert_eq!(b3f.version(), 1);

    // An import data file with block count of 2 does not have import data
    assert!(b3f.block_count() > 2);

    //
    // The first block is import file metadata in binary
    //
    let header = read_header(&b3f, data)?;

    //
    // The second block is the default asset as UTF-8 json
    //
    //let default_asset = read_default_asset(&b3f, data, schema_set)?;

    //
    // The third block is UTF-8 json import data
    //
    let import_data_json_block = &b3f.read_block(data, 2)?;
    let import_data_json_str = std::str::from_utf8(&import_data_json_block).unwrap();

    // Parse the json to reconstruct the property data
    let stored_object_json: SingleObjectJson = {
        profiling::scope!("serde_json::from_str");
        serde_json::from_str(import_data_json_str).unwrap()
    };

    //
    // Use remaining blocks to create a buffer list so we can re-create the json object
    //
    let mut buffers = vec![];
    for i in 3..b3f.block_count() {
        buffers.push(Arc::new(b3f.read_block(data, i)?));
    }

    let single_object = {
        profiling::scope!("SingleObjectJson::to_single_object");
        stored_object_json.to_single_object(schema_set, &mut Some(buffers))
    };

    Ok(SingleObjectWithMetadata {
        single_object,
        //default_asset,
        metadata: header.metadata,
    })
}

#[profiling::function]
pub fn save_single_object_to_b3f<W: std::io::Write>(
    write: W,
    import_data: Option<&SingleObject>,
    metadata: &ImportDataMetadata,
    schema_set: &SchemaSet,
    default_asset: &SingleObject,
) {
    let mut b3f_writer = b3f::B3FWriter::new_from_u8_tag(*b"HYIF", 1);

    //
    // Store the binary header in block index 0
    //
    //TODO: Write this + import file size + import file modified time
    let mut data = Vec::default();
    let header = ImportDataHeader {
        metadata: *metadata,
    };
    header.write_header(&mut data).unwrap();
    b3f_writer.add_block(&data);

    //
    // Store the default asset in block index 1
    //
    let default_asset_json_object = SingleObjectJson::new(schema_set, default_asset, &mut None);
    let default_asset_json = {
        profiling::scope!("serde_json::to_string_pretty");
        serde_json::to_string_pretty(&default_asset_json_object).unwrap()
    };
    let default_asset_bytes = default_asset_json.into_bytes();
    b3f_writer.add_block(&default_asset_bytes);

    let Some(import_data) = import_data else {
        return b3f_writer.write(write);
    };

    //
    // Store the import data starting at block index 2
    //

    // Encode the object as a json object + binary buffers
    let mut buffers = Some(Vec::default());
    let import_data_object_json = SingleObjectJson::new(schema_set, import_data, &mut buffers);
    let buffers = buffers.unwrap();

    // Encode the json object to string
    let single_object_json = {
        profiling::scope!("serde_json::to_string_pretty");
        serde_json::to_string_pretty(&import_data_object_json).unwrap()
    };

    // Store string to block index 2
    let single_object_bytes = single_object_json.into_bytes();
    b3f_writer.add_block(&single_object_bytes);

    // Buffers to into subsequent blocks
    for buffer in &buffers {
        b3f_writer.add_block(buffer.as_slice());
    }

    //
    // Write the b3f
    //
    b3f_writer.write(write)
}

//! Basic Binary Block Format (B3F)
//!
//! File Format
//! [4] magic number encoded as u32 (0xBB33FF00)
//! [4] file tag (arbitrary 4 bytes for user)
//! [4] version (arbitrary meaning for user, encoded as u32)
//! [4] block count (encoded as u32)
//! [8] bytes indicating 0 (0x00)
//! [8*n] ending offset of block
//! [x] pad to 16 byte offset
//! [n*len(n)] data (format/encoding/semantics would be implied by file tag). Each block begins at
//! [x] pad to 16 byte offset
//!
//! Endianness is undefined. Use the magic number to detect if endianness is different between
//! writer/reader
//!
//! This format can be encoded into a block, making this structure hierarchical. In this
//! case, omit the magic number, and use the file tag to optionally indicate the contents
//! of the block. (So it becomes a "block tag")
//!
//! if you c-cast the range memory from byte 16 to block count * 8, you have an array of u64 of n+1
//! length where n is number of blocks. Offset for block n is given by array[n]. End of block n is
//! given by array[n+1]. Size of block n in bytes is given by array[n+1] - array[n]
//!
//! Alignment of blocks to 16 bytes promotes reinterpreting bytes i.e. u8 to u64 or __m128 without
//! tripping over undefined behavior

use std::convert::TryInto;
use std::io::{Cursor, SeekFrom};
use std::ops::Range;

const HEADER_SIZE_IN_BYTES: usize = 16;
const BLOCK_LENGTH_SIZE_IN_BYTES: usize = 8;
const BLOCK_ALIGNMENT_IN_BYTES: usize = 16;

/// Used to encode data into B3F format
pub struct B3FWriter<'a> {
    file_tag: u32,
    version: u32,
    blocks: Vec<&'a [u8]>,
}

impl<'a> B3FWriter<'a> {
    pub fn new_from_u8_tag(
        file_tag: [u8; 4],
        version: u32,
    ) -> Self {
        B3FWriter {
            file_tag: u32::from_ne_bytes(file_tag),
            version,
            blocks: Vec::default(),
        }
    }

    pub fn new_from_u32_tag(
        file_tag: u32,
        version: u32,
    ) -> Self {
        B3FWriter {
            file_tag,
            version,
            blocks: Vec::default(),
        }
    }

    pub fn add_block(
        &mut self,
        data: &'a [u8],
    ) {
        self.blocks.push(data);
    }

    pub fn write<W: std::io::Write>(
        &self,
        mut writer: W,
    ) {
        //
        // 16 byte header
        //
        writer.write(&0xBB33FF00u32.to_ne_bytes()).unwrap();
        writer.write(&self.file_tag.to_ne_bytes()).unwrap();
        writer.write(&self.version.to_ne_bytes()).unwrap();
        let block_count = self.blocks.len() as u32;
        writer.write(&block_count.to_ne_bytes()).unwrap();

        //
        // A single u64 zero + N u64 block end positions
        //
        writer.write(&0u64.to_ne_bytes()).unwrap();

        let mut block_begin = 0;
        for block in &self.blocks {
            // Determine where the block ends
            let block_end = block_begin + block.len();

            // Write the ending of the previous block (or 0 for first block)
            writer.write(&(block_end as u64).to_ne_bytes()).unwrap();

            // Realign to 16 bytes, this is where the next block begins
            block_begin = ((block_end + BLOCK_ALIGNMENT_IN_BYTES - 1) / BLOCK_ALIGNMENT_IN_BYTES)
                * BLOCK_ALIGNMENT_IN_BYTES;
        }

        //
        // Pad block 0 to start at a 16 byte offset
        //
        let data_offset =
            HEADER_SIZE_IN_BYTES + ((self.blocks.len() + 1) * BLOCK_LENGTH_SIZE_IN_BYTES);
        if data_offset % 16 == 8 {
            writer.write(&0u64.to_ne_bytes()).unwrap();
        } else {
            assert!(data_offset % 16 == 0);
        }

        //
        // Write the blocks
        //
        for block in &self.blocks {
            writer.write(*block).unwrap();
            if block.len() % 16 != 0 {
                let required_padding = 16 - block.len() % 16;
                for _ in 0..required_padding {
                    writer.write(&0u8.to_ne_bytes()).unwrap();
                }
            }
        }
    }
}

pub struct B3FReader {
    file_tag: [u8; 4],
    version: u32,
    block_count: u32,
}

impl B3FReader {
    pub fn file_tag_as_u32(&self) -> u32 {
        u32::from_ne_bytes(self.file_tag.try_into().unwrap())
    }

    pub fn file_tag_as_u8(&self) -> &[u8] {
        &self.file_tag
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn block_count(&self) -> usize {
        self.block_count as usize
    }

    pub fn new<T: std::io::Read + std::io::Seek>(reader: &mut T) -> std::io::Result<Option<Self>> {
        reader.seek(SeekFrom::Start(0))?;
        let mut bytes = [0u8; 4];
        reader.read(&mut bytes)?;
        let magic_number = u32::from_ne_bytes(bytes);
        if magic_number != 0xBB33FF00 {
            return Ok(None);
        }

        reader.read(&mut bytes)?;
        let file_tag = bytes;

        reader.read(&mut bytes)?;
        let version = u32::from_ne_bytes(bytes);

        reader.read(&mut bytes)?;
        let block_count = u32::from_ne_bytes(bytes);

        Ok(Some(B3FReader {
            file_tag,
            version,
            block_count,
        }))
    }

    pub fn get_block_location<T: std::io::Read + std::io::Seek>(
        &self,
        reader: &mut T,
        index: usize,
    ) -> std::io::Result<Range<usize>> {
        // assumed by some implementation details here
        debug_assert_eq!(BLOCK_LENGTH_SIZE_IN_BYTES, 8);
        let begin_size_offset = HEADER_SIZE_IN_BYTES + (index * BLOCK_LENGTH_SIZE_IN_BYTES);
        reader.seek(SeekFrom::Start(begin_size_offset as u64))?;

        let mut bytes = [0u8; 8];
        reader.read(&mut bytes)?;
        let mut begin = u64::from_ne_bytes(bytes.try_into().unwrap()) as usize;
        reader.read(&mut bytes)?;
        let end = u64::from_ne_bytes(bytes.try_into().unwrap()) as usize;

        // Begin position needs to be rounded up to 16-byte offset
        begin = ((begin + BLOCK_ALIGNMENT_IN_BYTES - 1) / BLOCK_ALIGNMENT_IN_BYTES)
            * BLOCK_ALIGNMENT_IN_BYTES;

        let mut data_offset =
            HEADER_SIZE_IN_BYTES + ((self.block_count as usize + 1) * BLOCK_LENGTH_SIZE_IN_BYTES);
        data_offset = ((data_offset + BLOCK_ALIGNMENT_IN_BYTES - 1) / BLOCK_ALIGNMENT_IN_BYTES)
            * BLOCK_ALIGNMENT_IN_BYTES;

        Ok((data_offset + begin)..(data_offset + end))
    }

    pub fn read_block<T: std::io::Read + std::io::Seek>(
        &self,
        reader: &mut T,
        index: usize,
    ) -> std::io::Result<Vec<u8>> {
        let block_location = self.get_block_location(reader, index)?;
        reader.seek(SeekFrom::Start(block_location.start as u64))?;
        let mut bytes = vec![0u8; block_location.end - block_location.start];
        reader.read(bytes.as_mut_slice())?;
        Ok(bytes)
    }

    pub fn read_block_from_slice<'a>(
        &self,
        data: &'a [u8],
        index: usize,
    ) -> std::io::Result<&'a [u8]> {
        let mut cursor = Cursor::new(data);
        //let buf_reader = BufReader::new(data);
        let block_location = self.get_block_location(&mut cursor, index)?;
        Ok(&data[block_location])
    }
}

// Basic Binary Block Format (B3F)
//
// File Format
// [4] magic number encoded as u32 (0xBB33FF00)
// [4] file tag (arbitrary 4 bytes for user)
// [4] version (arbitrary meaning for user, encoded as u32)
// [4] block count (encoded as u32)
// [8] bytes indicating 0 (0x00)
// [8*n] ending offset of block
// [x] pad to 16 byte offset
// [n*len(n)] data (format/encoding/semantics would be implied by file tag). Each block begins at
// [x] pad to 16 byte offset
//
// Endianness is undefined. Use the magic number to detect if endianness is different between
// writer/reader
//
// This format can be encoded into a block, making this structure hierarchical. In this
// case, omit the magic number, and use the file tag to optionally indicate the contents
// of the block. (So it becomes a "block tag")
//
// if you c-cast the range memory from byte 16 to block count * 8, you have an array of u64 of n+1
// length where n is number of blocks. Offset for block n is given by array[n]. End of block n is
// given by array[n+1]. Size of block n in bytes is given by array[n+1] - array[n]
//
// Alignment of blocks to 16 bytes promotes reinterpreting bytes i.e. u8 to u64 or __m128 without
// tripping over undefined behavior

use std::convert::TryInto;
use std::io::{BufWriter, Write};
use std::path::Path;

const HEADER_SIZE_IN_BYTES: usize = 16;
const BLOCK_LENGTH_SIZE_IN_BYTES: usize = 8;
const BLOCK_ALIGNMENT_IN_BYTES: usize = 16;

pub struct B3FWriter<'a> {
    file_tag: u32,
    version: u32,
    blocks: Vec<&'a [u8]>
}

impl<'a> B3FWriter<'a> {
    pub fn new_from_u8_tag(file_tag: [u8; 4], version: u32) -> Self {
        B3FWriter {
            file_tag: u32::from_ne_bytes(file_tag),
            version,
            blocks: Vec::default()
        }
    }

    pub fn new_from_u32_tag(file_tag: u32, version: u32) -> Self {
        B3FWriter {
            file_tag,
            version,
            blocks: Vec::default()
        }
    }

    pub fn add_block(&mut self, data: &'a [u8]) {
        self.blocks.push(data);
    }

    pub fn write<W: std::io::Write>(&self, mut writer: W) {
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
        let data_offset = HEADER_SIZE_IN_BYTES + ((self.blocks.len() + 1) * BLOCK_LENGTH_SIZE_IN_BYTES);
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
                for i in 0..required_padding {
                    writer.write(&0u8.to_ne_bytes()).unwrap();
                }
            }
        }
    }
}

pub struct B3FReader<'a> {
    data: &'a [u8],
}

impl<'a> B3FReader<'a> {
    pub fn new(data: &'a [u8]) -> Option<B3FReader<'a>> {
        if data.len() < 16 {
            return None;
        }

        let magic_number = u32::from_ne_bytes(data[0..4].try_into().ok()?);
        if magic_number != 0xBB33FF00 {
            return None;
        }

        let reader = B3FReader { data };

        Some(reader)
    }

    pub fn file_tag_as_u32(&self) -> u32 {
        u32::from_ne_bytes(self.data[4..8].try_into().unwrap())
    }

    pub fn file_tag_as_u8(&self) -> &[u8] {
        &self.data[4..8]
    }

    pub fn version(&self) -> u32 {
        u32::from_ne_bytes(self.data[8..12].try_into().unwrap())
    }

    pub fn block_count(&self) -> usize {
        u32::from_ne_bytes(self.data[12..16].try_into().unwrap()) as usize
    }

    pub fn get_block(
        &self,
        index: usize,
    ) -> &'a [u8] {
        // assumed by some implementation details here
        debug_assert_eq!(BLOCK_LENGTH_SIZE_IN_BYTES, 8);
        let begin_size_offset = HEADER_SIZE_IN_BYTES + (index * BLOCK_LENGTH_SIZE_IN_BYTES);
        let size_data = &self.data[begin_size_offset..];
        let mut begin = u64::from_ne_bytes(size_data[0..8].try_into().unwrap()) as usize;
        let end = u64::from_ne_bytes(size_data[8..16].try_into().unwrap()) as usize;

        // Begin position needs to be rounded up to 16-byte offset
        begin = ((begin + BLOCK_ALIGNMENT_IN_BYTES - 1) / BLOCK_ALIGNMENT_IN_BYTES)
            * BLOCK_ALIGNMENT_IN_BYTES;

        let mut data_offset =
            HEADER_SIZE_IN_BYTES + ((self.block_count() + 1) * BLOCK_LENGTH_SIZE_IN_BYTES);
        data_offset = ((data_offset + BLOCK_ALIGNMENT_IN_BYTES - 1) / BLOCK_ALIGNMENT_IN_BYTES)
            * BLOCK_ALIGNMENT_IN_BYTES;
        &self.data[data_offset..][begin..end]
    }
}

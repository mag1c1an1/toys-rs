// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bloom::Bloom;
use std::{fs::File, path::Path, sync::Arc};

use crate::{
    block::Block,
    key::{KeyBytes, KeySlice},
    lsm_storage::BlockCache,
};

pub(crate) mod bloom;
mod builder;
mod iterator;

pub struct SsTable {
    pub(crate) file: FileObject,
    pub(crate) block_meta: Vec<BlockMeta>,
    pub(crate) block_meta_offset: usize,
    id: usize,
    block_cache: Option<Arc<BlockCache>>,
    first_key: KeyBytes,
    last_key: KeyBytes,
    pub(crate) bloom: Option<Bloom>,
    max_ts: u64,
}
impl SsTable {
    /// Open SSTable from a file.
    pub fn open(
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        file: FileObject,
    ) -> Result<Self> {
        todo!()
        // let len = file.size();
        // let raw_bloom_offset = file.read(len - 4, 4)?;
        // let bloom_offset = (&raw_bloom_offset[..]).get_u32() as u64;
        // let raw_bloom = file.read(bloom_offset, len - 4 - bloom_offset)?;
        // let bloom_filter = Bloom::decode(&raw_bloom)?;
        // let raw_meta_offset = file.read(bloom_offset - 4, 4)?;
        // let block_meta_offset = (&raw_meta_offset[..]).get_u32() as u64;
        // let raw_meta =
        //     file.read(block_meta_offset, bloom_offset - 4 - block_meta_offset)?;
        // let (block_meta, max_ts) = BlockMeta::decode_block_meta(&raw_meta[..])?;
        // Ok(Self {
        //     file,
        //     first_key: block_meta.first().unwrap().first_key.clone(),
        //     last_key: block_meta.last().unwrap().last_key.clone(),
        //     block_meta,
        //     block_meta_offset: block_meta_offset as usize,
        //     id,
        //     block_cache,
        //     bloom: Some(bloom_filter),
        //     max_ts,
        // })
    }
    /// Create a mock SST with only first key + last key metadata
    pub fn create_meta_only(
        id: usize,
        file_size: u64,
        first_key: KeyBytes,
        last_key: KeyBytes,
    ) -> Self {
        todo!()
    }

    /// Read a block from the disk.
    pub fn read_block(&self, block_idx: usize) -> Result<Arc<Block>> {
        todo!()
    }
    /// Read a block from disk, with block cache.
    pub fn read_block_cached(&self, block_idx: usize) -> Result<Arc<Block>> {
        todo!()
        // if let Some(ref block_cache) = self.block_cache {
        //     let blk = block_cache
        //         .try_get_with((self.id, block_idx), || self.read_block(block_idx))
        //         .map_err(|e| anyhow!("{}", e))?;
        //     Ok(blk)
        // } else {
        //     self.read_block(block_idx)
        // }
    }
    pub fn find_block_idx(&self, key: KeySlice) -> usize {
        todo!()
    }
    /// Get number of data blocks.
    pub fn num_of_blocks(&self) -> usize {
        self.block_meta.len()
    }

    /// Get first key in this sstable
    pub fn first_key(&self) -> &KeyBytes {
        &self.first_key
    }

    /// Get last key in this sstable
    pub fn last_key(&self) -> &KeyBytes {
        &self.last_key
    }

    /// size of this sstable
    pub fn table_size(&self) -> u64 {
        self.file.1
    }

    /// id of this sstable
    pub fn sst_id(&self) -> usize {
        self.id
    }

    /// max ts of this sstable
    pub fn max_ts(&self) -> u64 {
        self.max_ts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMeta {
    /// Offset of this data block.
    pub offset: usize,
    /// The first key of the data block.
    pub first_key: KeyBytes,
    /// The last key of the data block.
    pub last_key: KeyBytes,
}
impl BlockMeta {
    /// encode block meta to a buffer
    pub fn encode_block_meta(
        block_meta: &[BlockMeta],
        max_ts: u64,
        buf: &mut Vec<u8>,
    ) {
        todo!()
    }
    /// decode block meta from a buffer
    pub fn decode_block_meta(mut buf: &[u8]) -> Result<(Vec<BlockMeta>, u64)> {
        todo!()
    }
}

/// A file object
pub struct FileObject(Option<File>, u64);

impl FileObject {
    /// Create a new file object and write the file to the disk.
    pub fn create(path: &Path, data: Vec<u8>) -> Result<Self> {
        std::fs::write(path, &data)?;
        File::open(path)?.sync_all()?;
        Ok(FileObject(
            Some(File::options().read(true).write(false).open(path)?),
            data.len() as u64,
        ))
    }

    /// open file object
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::options().read(true).write(false).open(path)?;
        let size = file.metadata()?.len();
        Ok(FileObject(Some(file), size))
    }

    /// read `len`` bytes from `offset` in file
    pub fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        use std::os::unix::fs::FileExt;
        let mut data = vec![0; len as usize];
        self.0
            .as_ref()
            .unwrap()
            .read_exact_at(&mut data[..], offset)?;
        Ok(data)
    }

    /// size of file
    pub fn size(&self) -> u64 {
        self.1
    }
}

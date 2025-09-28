// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use std::{path::Path, sync::Arc};

use crate::{
    block::BlockBuilder,
    key::{KeySlice, KeyVec},
    lsm_storage::BlockCache,
};

use super::{BlockMeta, FileObject, SsTable, bloom::Bloom};
use anyhow::Result;
use bytes::BufMut;

pub struct SsTableBuilder {
    builder: BlockBuilder,
    first_key: KeyVec,
    last_key: KeyVec,
    data: Vec<u8>,
    pub(crate) meta: Vec<BlockMeta>,
    block_size: usize,
    key_hashes: Vec<u32>,
    max_ts: u64,
}

impl SsTableBuilder {
    /// Create a builder based on target block size.
    pub fn new(block_size: usize) -> Self {
        Self {
            data: Vec::new(),
            meta: Vec::new(),
            first_key: KeyVec::new(),
            last_key: KeyVec::new(),
            block_size,
            builder: BlockBuilder::new(block_size),
            key_hashes: Vec::new(),
            max_ts: 0,
        }
    }

    /// Adds a k-v pair to SSTable
    pub fn add(&mut self, key: KeySlice, value: &[u8]) {
        if self.first_key.is_empty() {
            self.first_key.set_from_slice(key);
        }

        self.max_ts = self.max_ts.max(key.ts());

        self.key_hashes.push(farmhash::fingerprint32(key.key_ref()));

        if self.builder.add(key, value) {
            self.last_key.set_from_slice(key);
            return;
        }

        // block full
        // create a new block builder and append block data
        self.finish_block();

        // add the key-value pair to the next block
        assert!(self.builder.add(key, value));
        self.first_key.set_from_slice(key);
        self.last_key.set_from_slice(key);
    }

    /// Get the estimated size of the SSTable.
    pub fn estimated_size(&self) -> usize {
        self.data.len()
    }

    pub fn build(
        mut self,
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        path: impl AsRef<Path>,
    ) -> Result<SsTable> {
        self.finish_block();
        let mut buf = self.data;
        let meta_offset = buf.len();
        BlockMeta::encode_block_meta(&self.meta, self.max_ts, &mut buf);
        buf.put_u32(meta_offset as u32);
        let bloom = Bloom::build_from_key_hashes(
            &self.key_hashes,
            Bloom::bloom_bits_per_key(self.key_hashes.len(), 0.01),
        );
        let bloom_offset = buf.len();
        bloom.encode(&mut buf);
        buf.put_u32(bloom_offset as u32);
        let file = FileObject::create(path.as_ref(), buf)?;
        Ok(SsTable {
            id,
            file,
            first_key: self.meta.first().unwrap().first_key.clone(),
            last_key: self.meta.last().unwrap().last_key.clone(),
            block_meta: self.meta,
            block_meta_offset: meta_offset,
            block_cache,
            bloom: Some(bloom),
            max_ts: self.max_ts,
        })
    }

    fn finish_block(&mut self) {
        let builder = std::mem::replace(
            &mut self.builder,
            BlockBuilder::new(self.block_size),
        );
        let encode_block = builder.build().encode();
        self.meta.push(BlockMeta {
            offset: self.data.len(),
            first_key: std::mem::take(&mut self.first_key).into_key_bytes(),
            last_key: std::mem::take(&mut self.last_key).into_key_bytes(),
        });
        let check_sum = crc32fast::hash(&encode_block);
        self.data.extend(encode_block);
        self.data.put_u32(check_sum);
    }
}

#[cfg(test)]
impl SsTableBuilder {
    pub fn build_for_test(self, path: impl AsRef<Path>) -> Result<SsTable> {
        self.build(0, None, path)
    }
}

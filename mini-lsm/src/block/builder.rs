// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

//! keys
//! (overlap, key_len, key, ts, value_len, value)

use bytes::BufMut;

use crate::key::{KeySlice, KeyVec};

use super::{Block, SIZEOF_U16};

pub struct BlockBuilder {
    offsets: Vec<u16>,
    data: Vec<u8>,
    block_size: usize,
    first_key: KeyVec,
}

impl BlockBuilder {
    pub fn new(block_size: usize) -> Self {
        Self {
            offsets: Vec::new(),
            data: Vec::new(),
            block_size,
            first_key: KeyVec::new(),
        }
    }
    #[must_use]
    /// `false` means no space
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        assert!(!key.is_empty(), "key must not be empty");
        // full
        if self.estimated_size() + key.raw_len() + value.len() + SIZEOF_U16 * 3 /* key_len, value_len and offset */ > self.block_size
            && !self.is_empty()
        {
            return false;
        }
        // Add the offset of the data into the offset array.
        self.offsets.push(self.data.len() as u16);
        let overlap = compute_overlap(self.first_key.as_key_slice(), key);
        // Encode key overlap.
        self.data.put_u16(overlap as u16);
        // Encode key length.
        self.data.put_u16((key.key_len() - overlap) as u16);
        // Encode key content.
        self.data.put(&key.key_ref()[overlap..]);
        // Encode key ts
        self.data.put_u64(key.ts());
        // Encode value length.
        self.data.put_u16(value.len() as u16);
        // Encode value content.
        self.data.put(value);

        if self.first_key.is_empty() {
            self.first_key = key.to_key_vec();
        }

        true
    }
    fn estimated_size(&self) -> usize {
        /* number of key-value pairs in the block */
        SIZEOF_U16  +
          self.offsets.len() * SIZEOF_U16 /* offsets */
           + self.data.len() // key-value pairs
    }
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }
    pub fn build(self) -> Block {
        if self.is_empty() {
            panic!("block should not be empty");
        }
        Block {
            data: self.data,
            offsets: self.offsets,
        }
    }
}

fn compute_overlap(first_key: KeySlice, key: KeySlice) -> usize {
    let mut i = 0;
    loop {
        if i >= first_key.key_len() || i >= key.key_len() {
            break;
        }
        if first_key.key_ref()[i] != key.key_ref()[i] {
            break;
        }
        i += 1;
    }
    i
}

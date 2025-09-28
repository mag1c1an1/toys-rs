// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

pub mod concat_iterator;
pub mod merge_iterator;
pub mod two_merge_iterator;

use anyhow::Result;

pub trait StorageIterator {
    type KeyType<'a>: PartialEq + Eq + PartialOrd + Ord
    where
        Self: 'a;

    fn key(&self) -> Self::KeyType<'_>;
    fn value(&self) -> &[u8];
    fn is_valid(&self) -> bool;
    fn next(&mut self) -> Result<()>;
    fn num_active_iterators(&self) -> usize {
        1
    }
}

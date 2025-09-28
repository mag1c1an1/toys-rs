use std::ops::Bound;

use bytes::Bytes;

use crate::iterators::StorageIterator;
use anyhow::Result;

pub struct LsmIterator {
    // inner: LsmIteratorgccInner,
    end_bound: Bound<Bytes>,
    is_valid: bool,
    read_ts: u64,
    prev_key: Vec<u8>,
}

impl StorageIterator for LsmIterator {
    type KeyType<'a> = &'a [u8];

    fn is_valid(&self) -> bool {
        todo!()
    }

    fn key(&self) -> &[u8] {
        todo!()
    }

    fn value(&self) -> &[u8] {
        todo!()
    }

    fn next(&mut self) -> Result<()> {
        // self.next_inner()?;
        // self.move_to_key()?;
        // Ok(())
        todo!()
    }

    fn num_active_iterators(&self) -> usize {
        todo!()
    }
}

/// A wrapper around existing iterator,
/// will prevent users from calling `next` when the iterator is invalid.
/// If an iterator is already invalid, `next` does not do anything.
/// If `next` returns an error,
/// `is_valid` should return false, and `next` should always return an error.
pub struct FusedIterator<I: StorageIterator> {
    iter: I,
    has_errored: bool,
}

impl<I: StorageIterator> StorageIterator for FusedIterator<I> {
    type KeyType<'a>
        = I::KeyType<'a>
    where
        Self: 'a;

    fn value(&self) -> &[u8] {
        todo!()
    }

    fn key(&self) -> Self::KeyType<'_> {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }

    fn next(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

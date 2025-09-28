use std::{
    cmp,
    collections::{BinaryHeap, binary_heap::PeekMut},
};

use crate::{iterators::StorageIterator, key::KeySlice};

struct HeapWrapper<I: StorageIterator>(pub usize, pub Box<I>);

impl<I: StorageIterator> Ord for HeapWrapper<I> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.1
            .key()
            .cmp(&other.1.key())
            .then(self.0.cmp(&other.0))
            .reverse()
    }
}

impl<I: StorageIterator> PartialOrd for HeapWrapper<I> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: StorageIterator> PartialEq for HeapWrapper<I> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl<I: StorageIterator> Eq for HeapWrapper<I> {}

pub struct MergeIter<I: StorageIterator> {
    iters: BinaryHeap<HeapWrapper<I>>,
    current: Option<HeapWrapper<I>>,
}

impl<I: StorageIterator> MergeIter<I> {
    pub fn create(iters: Vec<Box<I>>) -> Self {
        if iters.is_empty() {
            return Self {
                iters: BinaryHeap::new(),
                current: None,
            };
        }
        let mut heap = BinaryHeap::new();
        if iters.iter().all(|x| x.is_valid()) {
            // all invalid, last one is current
            let mut iters = iters;
            return Self {
                iters: heap,
                current: Some(HeapWrapper(0, iters.pop().unwrap())),
            };
        }

        for (idx, iter) in iters.into_iter().enumerate() {
            if iter.is_valid() {
                heap.push(HeapWrapper(idx, iter))
            }
        }

        let current = heap.pop();

        Self {
            iters: heap,
            current,
        }
    }
}

impl<I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>>
    StorageIterator for MergeIter<I>
{
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        self.current.as_ref().unwrap().1.key()
    }

    fn value(&self) -> &[u8] {
        self.current.as_ref().unwrap().1.value()
    }

    fn is_valid(&self) -> bool {
        self.current
            .as_ref()
            .map(|x| x.1.is_valid())
            .unwrap_or(false)
    }

    fn next(&mut self) -> anyhow::Result<()> {
        let current = self.current.as_mut().unwrap();
        // while for inner iter
        while let Some(mut inner_iter) = self.iters.peek_mut() {
            if inner_iter.1.key() == current.1.key() {
                // no access
                if let e @ Err(_) = inner_iter.1.next() {
                    PeekMut::pop(inner_iter);
                    return e;
                }
                if !inner_iter.1.is_valid() {
                    PeekMut::pop(inner_iter);
                }
            } else {
                break;
            }
        }
        current.1.next()?;
        if !current.1.is_valid() {
            if let Some(iter) = self.iters.pop() {
                *current = iter;
            }
            return Ok(());
        }
        if let Some(mut inner_iter) = self.iters.peek_mut() {
            if *current < *inner_iter {
                // inner_iter > current
                // current still valid
                std::mem::swap(&mut *inner_iter, current);
            }
        }
        Ok(())
    }
}

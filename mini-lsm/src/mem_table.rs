#![allow(dead_code)]

use std::{ops::Bound, path::Path, sync::atomic::AtomicUsize};

use crate::{
    iterators::StorageIterator,
    key::{KeyBytes, KeySlice},
    wal::Wal,
};
use anyhow::Result;
use bytes::Bytes;
use crossbeam_skiplist::{
    SkipMap,
    map::{Entry, Range},
};

/// thread safe
pub struct MemTable {
    // more format
    pub(crate) map: SkipMap<KeyBytes, Bytes>,
    wal: Option<Wal>,
    id: usize,
    approximate_size: AtomicUsize,
}

impl MemTable {
    /// create a memtable by id
    pub fn create(id: usize) -> Self {
        Self {
            id,
            map: SkipMap::new(),
            wal: None,
            approximate_size: AtomicUsize::new(0),
        }
    }

    /// create a mem table by id and wal
    pub fn create_with_wal(id: usize, path: impl AsRef<Path>) -> Self {
        todo!()
    }

    /// recover a mem table by id and wal
    pub fn recover_from_wal(id: usize, path: impl AsRef<Path>) -> Self {
        todo!()
    }

    /// get val by key
    pub fn get(&self, key: KeySlice) -> Option<Bytes> {
        let key_bytes = KeyBytes::from_bytes_with_ts(
            Bytes::from_static(unsafe {
                // 'a -> 'static
                std::mem::transmute::<&[u8], &[u8]>(key.key_ref())
            }),
            key.ts(),
        );
        self.map.get(&key_bytes).map(|e| e.value().clone())
    }

    /// put val into memtable
    pub fn put(&self, key: KeySlice, value: &[u8]) -> Result<()> {
        self.put_batch(&[(key, value)])
    }

    /// put batch style api
    pub fn put_batch(&self, data: &[(KeySlice, &[u8])]) -> Result<()> {
        let mut estimated = 0;
        for (k, v) in data {
            estimated += k.raw_len() + v.len();
            self.map.insert(
                k.to_key_vec().into_key_bytes(),
                Bytes::copy_from_slice(&v),
            );
        }
        self.approximate_size
            .fetch_add(estimated, std::sync::atomic::Ordering::Relaxed);
        // todo wal
        Ok(())
    }

    /// Get an iterator over a range of keys
    pub fn scan(
        &self,
        lower: Bound<KeySlice>,
        upper: Bound<KeySlice>,
    ) -> MemTableIter {
        let (low, up) = (map_key_bound(lower), map_key_bound(upper));
        let inner = self.map.range((low, up));
        let mut ret = MemTableIter {
            inner_iter: inner,
            item: (KeyBytes::new(), Bytes::new()),
        };
        ret.next().unwrap();
        ret
    }

    // pub fn flush(&self,builder:&mut )

    /// return id of this mem table
    pub fn id(&self) -> usize {
        self.id
    }

    /// return approximate size of this mem table
    pub fn approximate_size(&self) -> usize {
        self.approximate_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// is this mem table empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
// for test api
impl MemTable {
    pub fn for_testing_get_slice(&self, key: &[u8]) -> Option<Bytes> {
        self.get(KeySlice::from_slice(key, crate::key::TS_DEFAULT))
    }
    pub fn for_testing_put_slice(
        &self,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        self.put(KeySlice::from_slice(key, crate::key::TS_DEFAULT), value)
    }
    pub fn for_testing_scan_slice(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> MemTableIter<'_> {
        self.scan(
            lower.map(|x| KeySlice::from_slice(x, crate::key::TS_DEFAULT)),
            upper.map(|x| KeySlice::from_slice(x, crate::key::TS_DEFAULT)),
        )
    }
}

/// Create a bound of `KeyBytes` from a bound of `KeySlice`.
fn map_key_bound(bound: Bound<KeySlice>) -> Bound<KeyBytes> {
    match bound {
        Bound::Included(x) => Bound::Included(KeyBytes::from_bytes_with_ts(
            Bytes::copy_from_slice(x.key_ref()),
            x.ts(),
        )),
        Bound::Excluded(x) => Bound::Excluded(KeyBytes::from_bytes_with_ts(
            Bytes::copy_from_slice(x.key_ref()),
            x.ts(),
        )),
        Bound::Unbounded => Bound::Unbounded,
    }
}

// key range k v
type SkipMapRangeIter<'a> =
    Range<'a, KeyBytes, (Bound<KeyBytes>, Bound<KeyBytes>), KeyBytes, Bytes>;

pub struct MemTableIter<'map> {
    inner_iter: SkipMapRangeIter<'map>,
    item: (KeyBytes, Bytes),
}

impl<'map> MemTableIter<'map> {
    fn entry_to_item(
        entry: Option<Entry<'_, KeyBytes, Bytes>>,
    ) -> (KeyBytes, Bytes) {
        entry
            .map(|x| (x.key().clone(), x.value().clone()))
            .unwrap_or_else(|| (KeyBytes::new(), Bytes::new()))
    }
}

impl<'map> StorageIterator for MemTableIter<'map> {
    type KeyType<'a>
        = KeySlice<'a>
    where
        Self: 'a;

    fn key(&self) -> Self::KeyType<'_> {
        self.item.0.as_key_slice()
    }

    fn value(&self) -> &[u8] {
        &self.item.1[..]
    }

    fn is_valid(&self) -> bool {
        !self.item.0.is_empty()
    }

    fn next(&mut self) -> Result<()> {
        let item = MemTableIter::entry_to_item(self.inner_iter.next());
        self.item = item;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::iterators::StorageIterator;

    use super::MemTable;

    #[test]
    fn test_memtable_get() {
        let memtable = MemTable::create(0);
        memtable.for_testing_put_slice(b"key1", b"value1").unwrap();
        memtable.for_testing_put_slice(b"key2", b"value2").unwrap();
        memtable.for_testing_put_slice(b"key3", b"value3").unwrap();
        assert_eq!(
            &memtable.for_testing_get_slice(b"key1").unwrap()[..],
            b"value1"
        );
        assert_eq!(
            &memtable.for_testing_get_slice(b"key2").unwrap()[..],
            b"value2"
        );
        assert_eq!(
            &memtable.for_testing_get_slice(b"key3").unwrap()[..],
            b"value3"
        );
    }
    #[test]
    fn test_memtable_overwrite() {
        let memtable = MemTable::create(0);
        memtable.for_testing_put_slice(b"key1", b"value1").unwrap();
        memtable.for_testing_put_slice(b"key2", b"value2").unwrap();
        memtable.for_testing_put_slice(b"key3", b"value3").unwrap();
        memtable.for_testing_put_slice(b"key1", b"value11").unwrap();
        memtable.for_testing_put_slice(b"key2", b"value22").unwrap();
        memtable.for_testing_put_slice(b"key3", b"value33").unwrap();
        assert_eq!(
            &memtable.for_testing_get_slice(b"key1").unwrap()[..],
            b"value11"
        );
        assert_eq!(
            &memtable.for_testing_get_slice(b"key2").unwrap()[..],
            b"value22"
        );
        assert_eq!(
            &memtable.for_testing_get_slice(b"key3").unwrap()[..],
            b"value33"
        );
    }
    #[test]
    fn test_memtable_iter() {
        use std::ops::Bound;
        let memtable = MemTable::create(0);
        memtable.for_testing_put_slice(b"key1", b"value1").unwrap();
        memtable.for_testing_put_slice(b"key2", b"value2").unwrap();
        memtable.for_testing_put_slice(b"key3", b"value3").unwrap();

        {
            let mut iter = memtable
                .for_testing_scan_slice(Bound::Unbounded, Bound::Unbounded);
            assert_eq!(iter.key().for_testing_key_ref(), b"key1");
            assert_eq!(iter.value(), b"value1");
            assert!(iter.is_valid());
            iter.next().unwrap();
            assert_eq!(iter.key().for_testing_key_ref(), b"key2");
            assert_eq!(iter.value(), b"value2");
            assert!(iter.is_valid());
            iter.next().unwrap();
            assert_eq!(iter.key().for_testing_key_ref(), b"key3");
            assert_eq!(iter.value(), b"value3");
            assert!(iter.is_valid());
            iter.next().unwrap();
            assert!(!iter.is_valid());
        }

        {
            let mut iter = memtable.for_testing_scan_slice(
                Bound::Included(b"key1"),
                Bound::Included(b"key2"),
            );
            assert_eq!(iter.key().for_testing_key_ref(), b"key1");
            assert_eq!(iter.value(), b"value1");
            assert!(iter.is_valid());
            iter.next().unwrap();
            assert_eq!(iter.key().for_testing_key_ref(), b"key2");
            assert_eq!(iter.value(), b"value2");
            assert!(iter.is_valid());
            iter.next().unwrap();
            assert!(!iter.is_valid());
        }

        {
            let mut iter = memtable.for_testing_scan_slice(
                Bound::Excluded(b"key1"),
                Bound::Excluded(b"key3"),
            );
            assert_eq!(iter.key().for_testing_key_ref(), b"key2");
            assert_eq!(iter.value(), b"value2");
            assert!(iter.is_valid());
            iter.next().unwrap();
            assert!(!iter.is_valid());
        }
    }

    #[test]
    fn test_empty_memtable_iter() {
        use std::ops::Bound;
        let memtable = MemTable::create(0);
        {
            let iter = memtable.for_testing_scan_slice(
                Bound::Excluded(b"key1"),
                Bound::Excluded(b"key3"),
            );
            assert!(!iter.is_valid());
        }
        {
            let iter = memtable.for_testing_scan_slice(
                Bound::Included(b"key1"),
                Bound::Included(b"key2"),
            );
            assert!(!iter.is_valid());
        }
        {
            let iter = memtable
                .for_testing_scan_slice(Bound::Unbounded, Bound::Unbounded);
            assert!(!iter.is_valid());
        }
    }

    #[test]
    fn test_merge_1() {}
}

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicUsize},
};

use anyhow::Result;
use bytes::Bytes;
use options::LsmStorageOptions;
use parking_lot::{Mutex, RwLock};

use crate::{
    block::Block, key, mem_table::MemTable, mvcc::LsmMvccInner, table::SsTable,
};

mod options;

pub type BlockCache = moka::sync::Cache<(usize, usize), Arc<Block>>;
pub struct MiniLsm {
    pub(crate) inner: Arc<LsmStorageInner>,
}
impl MiniLsm {
    /// Start the storage engine by either loading an existing directory
    /// or creating a new one if the directory does
    /// not exist.
    pub fn open(
        path: impl AsRef<Path>,
        options: &LsmStorageOptions,
    ) -> anyhow::Result<Arc<Self>> {
        todo!()
    }

    pub fn close(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub fn get(&self, key: &[u8]) -> anyhow::Result<Option<Bytes>> {
        todo!()
    }
    pub fn put(&self, key: &[u8], value: &[u8]) -> anyhow::Result<()> {
        todo!()
    }
    pub fn delete(&self, key: &[u8]) -> anyhow::Result<()> {
        todo!()
    }
    // pub fn scan(&self,lower: Bound<&[u8]>, upper: Bound<&[u8]>) -> anyhow::Result<()> {
    //     todo!()
    // }
}

pub enum WriteBatchRecord<T: AsRef<[u8]>> {
    Put(T, T),
    Del(T),
}

pub(crate) struct LsmStorageInner {
    pub(crate) state: Arc<RwLock<Arc<LsmStorageState>>>,
    pub(crate) state_lock: Mutex<()>,
    path: PathBuf,
    pub(crate) block_cache: Arc<BlockCache>,
    next_sst_id: AtomicUsize,
    pub(crate) options: Arc<LsmStorageOptions>,
    // pub(crate) compaction_controller: CompactionController,
    // pub(crate) manifest: Option<Manifest>,
    pub(crate) mvcc: Option<LsmMvccInner>,
    // pub(crate) compaction_filters: Arc<Mutex<Vec<CompactionFilter>>>,
}

impl LsmStorageInner {
    pub fn open(
        path: impl AsRef<Path>,
        options: &LsmStorageOptions,
    ) -> anyhow::Result<Self> {
        todo!()
    }
    pub(crate) fn next_sst_id(&self) -> usize {
        self.next_sst_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    pub(crate) fn get_with_ts(
        &self,
        key: &[u8],
        read_ts: u64,
    ) -> Result<Option<Bytes>> {
        let snapshot = {
            let guard = self.state.read();
            Arc::clone(&guard)
        }; // drop global lock here
        todo!()
    }

    fn freeze_memtable_with_memtable(
        &self,
        memtable: Arc<MemTable>,
    ) -> Result<()> {
        let mut guard = self.state.write();
        // Swap the current memtable with a new one.
        // care about this clone
        // different snapshot not a same one
        let mut snapshot = guard.as_ref().clone();
        let old_memtable = std::mem::replace(&mut snapshot.memtable, memtable);
        // Add the memtable to the immutable memtables.
        snapshot.imm_memtables.insert(0, old_memtable.clone());
        // Update the snapshot. Origin may be dropped
        *guard = Arc::new(snapshot);

        drop(guard);
        // old_memtable.sync_wal()?;

        Ok(())
    }
}

type MemTableRef = Arc<MemTable>;

#[derive(Clone)]
pub struct LsmStorageState {
    pub memtable: MemTableRef,
    pub imm_memtables: Vec<MemTableRef>,
    /// L0 SSTs, from latest to earliest.
    pub l0_sstable: Vec<usize>,
    /// SsTables sorted by key range; L1 - L_max for leveled compaction, or tiers for tiered
    /// compaction.
    pub levels: Vec<(usize, Vec<usize>)>,
    /// SST objects.
    pub sstables: HashMap<usize, Arc<SsTable>>,
}

impl LsmStorageState {
    fn create(options: &LsmStorageOptions) -> Self {
        todo!()
    }
}

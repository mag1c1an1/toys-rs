use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Context, Result, bail};
use bytes::Buf;
use serde::{Deserialize, Serialize};

pub struct Manifest {
    file: Arc<Mutex<File>>,
}

#[derive(Serialize, Deserialize)]
pub enum ManifestRecord {
    Flush(usize),
    NewMemtable(usize),
    // Compaction(CompactionTask, Vec<usize>),
}

impl Manifest {
    /// create a manifest
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            file: Arc::new(Mutex::new(
                OpenOptions::new()
                    .read(true)
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .context("failed to create manifest")?,
            )),
        })
    }

    /// add a record in manifest
    pub fn add_record(
        &self,
        _state_lock_observer: &MutexGuard<()>,
        record: ManifestRecord,
    ) -> Result<()> {
        self.add_record_when_init(record)
    }

    /// add a record in init process
    pub fn add_record_when_init(&self, record: ManifestRecord) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        let buf = serde_json::to_vec(&record)?;
        let hash = crc32fast::hash(&buf);
        file.write_all(&(buf.len() as u64).to_be_bytes())?;
        file.write_all(&buf)?;
        file.sync_all()?;
        Ok(())
    }

    /// recover a manifest from disk
    pub fn recover(
        path: impl AsRef<Path>,
    ) -> Result<(Self, Vec<ManifestRecord>)> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .context("failed to recover menifest")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut buf_ptr = buf.as_slice();
        let mut records = Vec::new();
        while buf_ptr.has_remaining() {
            let len = buf_ptr.get_u64();
            let slice = &buf_ptr[..len as usize];
            let json = serde_json::from_slice::<ManifestRecord>(slice)?;
            buf_ptr.advance(len as usize);
            let checksum = buf_ptr.get_u32();
            if checksum != crc32fast::hash(slice) {
                bail!("manifest checksum mismatched");
            }
            records.push(json);
        }
        Ok((
            Self {
                file: Arc::new(Mutex::new(file)),
            },
            records,
        ))
    }
}

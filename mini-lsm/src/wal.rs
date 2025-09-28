// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result, bail};
use bytes::{Buf, BufMut, Bytes};
use crossbeam_skiplist::SkipMap;
use std::{
    fs::{File, OpenOptions},
    hash::Hasher,
    io::{BufWriter, Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::key::{KeyBytes, KeySlice};

pub struct Wal {
    file: Arc<Mutex<BufWriter<File>>>,
}

impl Wal {
    /// create WAL (write ahead log)
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            file: Arc::new(Mutex::new(BufWriter::new(
                OpenOptions::new()
                    .read(true)
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .context("failed to create wal")?,
            ))),
        })
    }

    /// recover wal from disk file
    pub fn recover(
        path: impl AsRef<Path>,
        skiplist: &SkipMap<KeyBytes, Bytes>,
    ) -> Result<Self> {
        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .context("failed to recover from WAL")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut rbuf: &[u8] = buf.as_slice();
        while rbuf.has_remaining() {
            let batch_size = rbuf.get_u32() as usize;
            if rbuf.remaining() < batch_size {
                bail!("incomplete WAL");
            }
            let mut batch_buf = &rbuf[..batch_size];
            let mut kv_pairs = Vec::new();
            let mut hasher = crc32fast::Hasher::new();
            let single_checksum = crc32fast::hash(batch_buf);
            while batch_buf.has_remaining() {
                let key_len = batch_buf.get_u16() as usize;
                hasher.write(&(key_len as u16).to_be_bytes());
                let key = Bytes::copy_from_slice(&batch_buf[..key_len]);
                hasher.write(&key);
                batch_buf.advance(key_len);
                let ts = batch_buf.get_u64();
                hasher.write(&ts.to_be_bytes());
                let val_len = batch_buf.get_u16() as usize;
                hasher.write(&(val_len as u16).to_be_bytes());
                let val = Bytes::copy_from_slice(&batch_buf[..val_len]);
                hasher.write(&val);
                kv_pairs.push((key, ts, val));
                batch_buf.advance(val_len);
            }
            rbuf.advance(batch_size);
            let expected_checksum = rbuf.get_u32();
            let component_checksum = hasher.finalize();
            assert_eq!(component_checksum, single_checksum);
            if single_checksum != expected_checksum {
                bail!("checksum mismatch");
            }
            for (k, t, v) in kv_pairs {
                skiplist.insert(KeyBytes::from_bytes_with_ts(k, t), v);
            }
        }
        Ok(Self {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }

    /// put k-v pair
    pub fn put(&self, key: KeySlice, value: &[u8]) -> Result<()> {
        self.put_batch(&[(key, value)])
    }

    /// batch style api for put
    pub fn put_batch(&self, data: &[(KeySlice, &[u8])]) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        let mut buf = Vec::<u8>::new();
        for (k, v) in data {
            buf.put_u16(k.key_len() as u16);
            buf.put_slice(k.key_ref());
            buf.put_u64(k.ts());
            buf.put_u16(v.len() as u16);
            buf.put_slice(v);
        }
        // batch size header
        file.write_all(&(buf.len() as u32).to_be_bytes())?;
        // k-v pairs
        file.write(&buf)?;
        // checksum(u32)
        file.write_all(&crc32fast::hash(&buf).to_be_bytes())?;
        Ok(())
    }

    /// sync memory to disk
    pub fn sync(&self) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        file.flush()?;
        file.get_mut().sync_all()?;
        Ok(())
    }
}

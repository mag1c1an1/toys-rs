use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Micro(u64),
    Finish,
}

pub struct Calculator {
    inner: Vec<u64>,
}

impl Calculator {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add_msg(&mut self, first: u64) -> Result<()> {
        let last = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        self.inner.push(last - first);
        Ok(())
    }

    pub fn print(&self) {
        let sum: u64 = self.inner.iter().sum();
        let len = self.inner.len();
        let avg = sum as f64 / self.inner.len() as f64;
        println!("total :{len} , avg lag: {avg} ms");
    }
}

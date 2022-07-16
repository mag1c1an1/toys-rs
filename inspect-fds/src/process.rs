use std::fmt::{self, write};

/// This enum can be used to represent whether a file is read-only, write-only, or read/write. An
/// enum is basically a value that can be one of some number of "things".
#[allow(unused)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

impl fmt::Display for AccessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       match self {
           AccessMode::Read => write!(f, "read"),
           AccessMode::Write => write!(f,"write"),
           AccessMode::ReadWrite => write!(f,"read/write"),
       } 
    }
}

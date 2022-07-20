use std::vec;

use crate::open_file::OpenFile;

#[derive(Debug, Clone, PartialEq)]
pub struct Process {
    pub pid: usize,
    pub ppid: usize,
    pub command: String,
}

impl Process {
    #[allow(unused)]
    pub fn new(pid: usize, ppid: usize, command: String) -> Self {
        Self { pid, ppid, command }
    }

    /// This function returns a list of file descriptor numbers for this Process, if that
    /// information is available (it will return None if the information is unavailable). The
    /// information will commonly be unavailable if the process has exited. (Zombie processes
    /// still have a pid, but their resources have already been freed, including the file
    /// descriptor table.)
    #[allow(unused)]
    pub fn list_fds(&self) -> Option<Vec<usize>> {
        unimplemented!()
    }

    /// This function returns a list of (fdnumber, OpenFile) tuples, if file descriptor
    /// information is available (it returns None otherwise). The information is commonly
    /// unavailable if the process has already exited.

    #[allow(unused)]
    pub fn list_open_files(&self) -> Option<Vec<(usize, OpenFile)>> {
        let mut open_files = vec![];
        for fd in self.list_fds()? {
            open_files.push((fd,OpenFile::from_fd(self.pid, fd)?));
        }
        Some(open_files)
    }
}

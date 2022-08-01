use regex::Regex;
use std::{fmt, collections::hash_map::DefaultHasher, hash::{Hasher, Hash}, fs};

const O_WRONLY: usize = 0000_0001;
const O_RDWR: usize = 0000_0002;

const COLORS: [&str; 6] = [
    "\x1B[38;5;9m",
    "\x1B[38;5;10m",
    "\x1B[38;5;11m",
    "\x1B[38;5;12m",
    "\x1B[38;5;13m",
    "\x1B[38;5;14m",
];
const CLEAR_COLOR: &str = "\x1B[0m";
/// This enum can be used to represent whether a file is read-only, write-only, or read/write. An
/// enum is basically a value that can be one of some number of "things".
#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

impl fmt::Display for AccessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessMode::Read => write!(f, "read"),
            AccessMode::Write => write!(f, "write"),
            AccessMode::ReadWrite => write!(f, "read/write"),
        }
    }
}

/// Stores information about open file on the system.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenFile {
    pub name: String,
    pub cursor: usize,
    pub access_mode: AccessMode,
}

impl OpenFile {
    #[allow(unused)]
    pub fn new(name: String, cursor: usize, access_mode: AccessMode) -> Self {
        Self {
            name,
            cursor,
            access_mode,
        }
    }

    /// This function takes a path of an open file and returns a more human-friendly name for that
    /// file .
    ///
    /// * For regular files, this will simply return the supplied path.
    /// * For terminals (files starting with /dev/pts), this will return "<terminal>".
    /// * For pipes (filenames formatted like pipe:[pipename]), this will return "<pipe #pipenum>".
    #[allow(unused)]
    fn path_to_name(path: &str) -> String {
        if path.starts_with("/dev/pts/") {
            String::from("<terminal>")
        } else if path.starts_with("pipe:[") && path.ends_with("]") {
            let pipe_num = &path[path.find('[').unwrap() + 1..path.find(']').unwrap()];
            format!("<pipe #{}>", pipe_num)
        } else {
            String::from(path)
        }
    }

    /// This file takes the contents of /proc/{pid}/fdinfo/{fdnum} for some file descriptor and
    /// extrats the cursor position of that file descriptor (technically, the position of the
    /// open file table entry that the fd potions to ) using a regex. It returns None if the cursor
    /// couldn't be found in the fdinfo text.
    #[allow(unused)]
    fn parse_cursor(fdinfo: &str) -> Option<usize> {
        let re = Regex::new(r"pos:\s*(\d+)").unwrap();
        Some(
            re.captures(fdinfo)?
                .get(1)?
                .as_str()
                .parse::<usize>()
                .ok()?,
        )
    }

    /// This file takes the contents of /proc/{pid}/fdinfo/{fdnum} for some file descriptor and
    /// extrats the access mode for that open file using the "flags:" field contained in the
    /// fdinfo text. It returns None if the "flags" field couldn't be found,
    #[allow(unused)]
    fn parse_access_mode(fdinfo: &str) -> Option<AccessMode> {
        let re = Regex::new(r"flags:\s*(\d+)").unwrap();
        let flags = usize::from_str_radix(re.captures(fdinfo)?.get(1)?.as_str(), 8).ok()?;
        if flags & O_WRONLY > 0 {
            Some(AccessMode::Write)
        } else if flags & O_RDWR > 0 {
            Some(AccessMode::ReadWrite)
        } else {
            Some(AccessMode::Read)
        }
    }

    /// Given a specified process and fd number, this function reads /proc/{pid}/fd/{fdnum} and
    /// /proc/{pid}/fdinfo/{fdnum} to populate an OpenFile struct. It returns None if the pid of fd
    /// are invalid, or if necessary information is unavailable.
    ///
    /// (Note: whether this function returns Option or Result is a matter of style and context.
    /// Some people might argue that you should return Result, so that you have finer grained
    /// control over possible things that could go wrong, e.g. you might want to handle things
    /// differently if this fails because the process doesn't have a specified fd, vs if it
    /// fails because it failed to read a /proc file. However, that significantly increases
    /// complexity of error handling. In our case, this does not need to be a super robust
    /// program and we don't need to do fine-grained error handling, so returning Option is a
    /// simple way to indicate that "hey, we weren't able to get the necessary information"
    /// without making a big deal of it.)
    #[allow(unused)]
    pub fn from_fd(pid: usize, fd: usize) -> Option<OpenFile> {
        let fd_path = format!("/proc/{}/fd/{}", pid, fd);
        let name = OpenFile::path_to_name(fs::read_link(fd_path).ok()?.to_str()?);
        let content_path = format!("/proc/{}/fdinfo/{}", pid, fd);
        let content = fs::read_to_string(content_path).ok()?;
        let cursor = OpenFile::parse_cursor(&content)?;
        let access_mode = OpenFile::parse_access_mode(&content)?;
        Some(OpenFile::new(name, cursor, access_mode))
    }

    /// This function returns the OpenFile's name with ANSI escape codes included to colorize
    /// pipe names. It hashed the pipe name so that the same pipe name will always result in the
    /// same color. This is useful for making program output more readable, since a user can
    /// quickly see all the fds that point to a particular pipe.
    #[allow(unused)]
    pub fn colorized_name(&self) -> String {
        if self.name.starts_with("<pipe")
        {
            let mut hash = DefaultHasher::new();
            self.name.hash(&mut hash);
            let hash_val = hash.finish();
            let color = COLORS[(hash_val % COLORS.len() as u64) as usize];
            format!("{}{}{}", color, self.name, CLEAR_COLOR)
        } else {
            self.name.to_owned()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ps_utils;
    use std::process::{Child, Command};

    fn start_c_program(program: &str) -> Child {
        Command::new(program)
            .spawn()
            .expect(&format!("Could not find {}. Have you run make?", program))
    }

    #[test]
    fn test_openfile_from_fd() {
        let mut test_subprocess = start_c_program("./multi_pipe_test");
        let process = ps_utils::get_target("multi_pipe_test").unwrap().unwrap();
        // Get file descriptor 0, which should point to the terminal
        let open_file = OpenFile::from_fd(process.pid, 0)
            .expect("Expected to get open file data for multi_pipe_test, but OpenFile::from_fd returned None");
        assert_eq!(open_file.name, "<terminal>");
        assert_eq!(open_file.cursor, 0);
        assert_eq!(open_file.access_mode, AccessMode::ReadWrite);
        let _ = test_subprocess.kill();
    }

    #[test]
    fn test_openfile_from_fd_invalid_fd() {
        let mut test_subprocess = start_c_program("./multi_pipe_test");
        let process = ps_utils::get_target("multi_pipe_test").unwrap().unwrap();
        // Get file descriptor 30, which should be invalid
        assert!(
            OpenFile::from_fd(process.pid, 30).is_none(),
            "Expected None because file descriptor 30 is invalid"
        );
        let _ = test_subprocess.kill();
    }
    #[test]
    fn test_colorized_name() {
        let of = OpenFile::new("<pipe #test>".to_owned(), 0, AccessMode::ReadWrite);
        assert_eq!("",of.colorized_name())
    }
}
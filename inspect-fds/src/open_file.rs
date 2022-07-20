use regex::Regex;
use std::{fmt, collections::hash_map::DefaultHasher, hash::{Hasher, Hash}};

const O_WRONLY: usize = 0000_0001;
const O_RDWR: usize = 0000_0002;
const COLORS: [&str; 6] = [
    "\x18[38;5;9m",
    "\x18[38;5;10m",
    "\x18[38;5;11m",
    "\x18[38;5;12m",
    "\x18[38;5;13m",
    "\x18[38;5;14m",
];


const CLEAR_COLOR:&str = "\x1B[0m";
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
            AccessMode::Write => write!(f, "write"),
            AccessMode::ReadWrite => write!(f, "read/write"),
        }
    }
}

/// Stores information about open file on the system.
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
        unimplemented!()
    }

    /// This function returns the OpenFile's name with ANSI escape codes included to colorize
    /// pipe names. It hashed the pipe name so that the same pipe name will always result in the
    /// same color. This is useful for making program output more readable, since a user can
    /// quickly see all the fds that point to a particular pipe.
    #[allow(unused)]
    pub fn colorized_name(&self) -> String {
        if self.name.starts_with("pipe")
        {
            let mut hash = DefaultHasher::new();
            self.name.hash(&mut hash);
            let hash_val = hash.finish();
            let color = COLORS[(hash_val % COLORS.len() as u64) as usize];
            format!("{}{}{}",color, self.name,CLEAR_COLOR)
        }else {
            self.name.to_owned()
        }
    }
}

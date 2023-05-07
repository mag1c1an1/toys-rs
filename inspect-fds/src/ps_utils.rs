use std::{fmt, process::Command};
use nix::unistd::getuid;
use crate::process::Process;
#[derive(Debug)]
pub enum Error {
    ExecutableError(std::io::Error),
    OutputFormatError(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ExecutableError(err) => write!(f, "Error executing ps: {}", err),
            Error::OutputFormatError(err) => write!(f, "ps printed malformed output: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::ExecutableError(error)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_error: std::string::FromUtf8Error) -> Self {
        Error::OutputFormatError("Output is not utf-8")
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_error: std::num::ParseIntError) -> Self {
        Error::OutputFormatError("Error parsing integer")
    }
}

#[allow(unused)]
fn parse_ps_line(line: &str) -> Result<Process, Error> {
    let mut remainder = line.trim();
    let first_token_end = remainder
        .find(char::is_whitespace)
        .ok_or(Error::OutputFormatError("Missing second column"))?;
    let pid = remainder[0..first_token_end].parse::<usize>()?;
    remainder = remainder[first_token_end..].trim_start();
    let second_token_end = remainder
        .find(char::is_whitespace)
        .ok_or(Error::OutputFormatError("Missing third column"))?;
    let ppid = remainder[0..second_token_end].parse::<usize>()?;
    remainder = remainder[second_token_end..].trim_start();
    Ok(Process::new(pid, ppid, String::from(remainder)))
}

#[allow(unused)]
fn get_process(pid: usize) -> Result<Option<Process>, Error> {
    let output = String::from_utf8(
        Command::new("ps")
            .args(["--pid", &pid.to_string(), "-o", "pid= ppid= command="])
            .output()?
            .stdout,
    )?;
    if !output.trim().is_empty() {
        Ok(Some(parse_ps_line(output.trim())?))
    } else {
        Ok(None)
    }
}

#[allow(unused)]
pub fn get_child_processes(pid: usize) -> Result<Vec<Process>, Error> {
    let ps_output = Command::new("ps")
        .args(["--ppid", &pid.to_string(), "-o", "pid= ppid= command="])
        .output()?;
    let mut output = vec![];
    for line in String::from_utf8(ps_output.stdout)?.lines() {
        output.push(parse_ps_line(line)?);
    }
    Ok(output)
}

#[allow(unused)]
fn get_pid_by_command_name(name: &str) -> Result<Option<usize>, Error> {
    let output = String::from_utf8(
        Command::new("pgrep")
            .args(["-xU", getuid().to_string().as_str(), name])
            .output()?
            .stdout,
    )?;
    Ok(match output.lines().next() {
        Some(line) => Some(line.parse::<usize>()?),
        None => None,
    })
}


#[allow(unused)]
pub fn get_target(query:&str) -> Result<Option<Process>,Error>
{
    let pid_by_command = get_pid_by_command_name(query)?;
    if let Some(pid) = pid_by_command {
        return get_process(pid);
    }
    
    match query.parse() {
        Ok(pid) => get_process(pid),
        Err(_) => Ok(None)
    }
}
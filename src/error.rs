use nix::errno::Errno;
use status::Status;
use std::error::Error;
use std::str::Utf8Error;
use regex;

#[derive(Debug)]
pub enum DebugError {
    PTrace(Errno),
    Status(Status),
    ParseInt(::std::num::ParseIntError),
    IOError(::std::io::Error),
    Regex(regex::Error),
    Utf8(Utf8Error),
    Error(&'static str),
}

impl From<Errno> for DebugError {
    fn from(err: Errno) -> Self {
        DebugError::PTrace(err)
    }
}

impl From<::std::io::Error> for DebugError {
    fn from(err: ::std::io::Error) -> Self {
        DebugError::IOError(err)
    }
}

impl From<regex::Error> for DebugError {
    fn from(err: regex::Error) -> Self {
        DebugError::Regex(err)
    }
}

impl From<Utf8Error> for DebugError {
    fn from(err: Utf8Error) -> Self {
        DebugError::Utf8(err)
    }
}

impl From<::std::num::ParseIntError> for DebugError {
    fn from(err: ::std::num::ParseIntError) -> Self {
        DebugError::ParseInt(err)
    }
}

impl From<Status> for DebugError {
    fn from(err: Status) -> Self {
        DebugError::Status(err)
    }
}


impl From<&'static str> for DebugError {
    fn from(err: &'static str) -> Self {
        DebugError::Error(err)
    }
}



impl DebugError {
    pub fn description(&self) -> String {
        match self {
            &DebugError::PTrace(ref errno) => {
                errno.desc().to_string()
            },
            &DebugError::Status(ref status) => {
                status.to_string()
            },
            &DebugError::IOError(ref io) => {
                io.to_string()
            }
            &DebugError::Error(ref e) => {
                e.to_string()
            }
            &DebugError::ParseInt(ref e) => {
                e.description().to_string()
            }
            &DebugError::Regex(ref e) => {
                e.description().to_string()
            }
            &DebugError::Utf8(ref e) => {
                e.description().to_string()
            }
        }
    }
}

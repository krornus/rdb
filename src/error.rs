use nix::errno::Errno;
use status::Status;

#[derive(Debug)]
pub enum DebugError {
    PTrace(Errno),
    Status(Status),
    IOError(::std::io::Error),
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
        }
    }
}

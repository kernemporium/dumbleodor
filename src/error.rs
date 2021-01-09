use std::convert::From;
use std::result;

use std::io;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::new(ErrorKind::IoError(err.kind()), "".to_string())
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    MmapFailed,
    IoError(io::ErrorKind),
    Utf8Error,
}
impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind, "".to_string())
    }
}

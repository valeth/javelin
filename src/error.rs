use std::{io, result};
use rml_rtmp::sessions::ServerSessionError as RtmpSessionError;


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RtmpSessionError(RtmpSessionError),
    Custom(String),
    HandshakeFailed,
    RequestError,
    SessionError(String),
    NotEnoughData,
    DecoderConfigurationRecordMissing,
    UnsupportedConfigurationRecordVersion(u8),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<RtmpSessionError> for Error {
    fn from(err: RtmpSessionError) -> Self {
        Error::RtmpSessionError(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &str) -> Self {
        Error::Custom(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Custom(err)
    }
}

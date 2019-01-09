use std::{io, result};
use rml_rtmp::sessions::ServerSessionError as RtmpSessionError;
#[cfg(feature = "hls")]
use mpeg2ts::Error as TransportStreamError;

pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RtmpSessionError(RtmpSessionError),
    Custom(String),
    HandshakeFailed,
    RequestError,
    SessionError(String),
    #[cfg(feature = "hls")]
    ParseError(String),
    #[cfg(feature = "hls")]
    NotEnoughData,
    #[cfg(feature = "hls")]
    DecoderConfigurationRecordMissing,
    #[cfg(feature = "hls")]
    AudioSpecificConfigurationMissing,
    #[cfg(feature = "hls")]
    UnsupportedConfigurationRecordVersion(u8),
    #[cfg(feature = "hls")]
    TransportStreamError(TransportStreamError),
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

#[cfg(feature = "hls")]
impl From<TransportStreamError> for Error {
    fn from(err: TransportStreamError) -> Self {
        Error::TransportStreamError(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &'a str) -> Self {
        Error::Custom(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Custom(err)
    }
}

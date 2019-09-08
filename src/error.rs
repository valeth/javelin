use std::{io, result};
use rml_rtmp::sessions::ServerSessionError as RtmpSessionError;
use tokio::timer::timeout::Error as TokioTimeoutError;
#[cfg(feature = "hls")]
use mpeg2ts::Error as TransportStreamError;
#[cfg(feature = "hls")]
use javelin_codec::Error as CodecError;

pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RtmpSessionError(RtmpSessionError),
    Custom(String),
    HandshakeFailed,
    RequestError,
    Timeout(String),
    SessionError(String),
    #[cfg(feature = "hls")]
    TransportStreamError(TransportStreamError),
    #[cfg(feature = "hls")]
    CodecError(CodecError)
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

#[cfg(feature = "hls")]
impl From<CodecError> for Error {
    fn from(err: CodecError) -> Self {
        Error::CodecError(err)
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

impl<T> From<TokioTimeoutError<T>> for Error {
    fn from(_err: TokioTimeoutError<T>) -> Self {
        Error::Timeout("Timed out".to_string())
    }
}

use {
    std::{io, result},
    rml_rtmp::sessions::ServerSessionError as RtmpSessionError,
};

#[cfg(feature = "hls")]
use {
    mpeg2ts::Error as TransportStreamError,
    javelin_codec::CodecError,
};


pub type Result<T, E=Error> = result::Result<T, E>;


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RtmpSessionError(RtmpSessionError),
    Custom(String),
    HandshakeFailed,
    RequestError,
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

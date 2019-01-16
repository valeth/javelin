use std::{io, result};


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    Custom(String),
    ParseError(String),
    NotEnoughData,
    DecoderConfigurationRecordMissing,
    AudioSpecificConfigurationMissing,
    UnsupportedConfigurationRecordVersion(u8),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
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

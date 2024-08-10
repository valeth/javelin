#![deny(unused_must_use)]
#![warn(rust_2018_idioms)]

pub mod error;
pub mod flv;
pub mod avc;
pub mod aac;
#[cfg(feature = "mpegts")]
pub mod mpegts;

pub use self::error::CodecError;


/// Decode bytes into a specific format.
pub trait ReadFormat<O> {
    type Context;
    type Error;

    fn read_format(&self, input: &[u8], ctx: &Self::Context) -> Result<O, Self::Error>;
}

/// Encode bytes from a specific format.
pub trait WriteFormat<I> {
    type Context;
    type Error;

    fn write_format(&self, input: I, ctx: &Self::Context) -> Result<Vec<u8>, Self::Error>;
}

pub trait FormatReader<F>
    where F: ReadFormat<Self::Output, Error=Self::Error>
{
    type Output;
    type Error;

    fn read_format(&mut self, format: F, input: &[u8]) -> Result<Option<Self::Output>, Self::Error>;
}

pub trait FormatWriter<F>
    where F: WriteFormat<Self::Input, Error=Self::Error>
{
    type Input;
    type Error;

    fn write_format(&mut self, format: F, input: Self::Input) -> Result<Vec<u8>, Self::Error>;
}

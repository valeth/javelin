pub mod adts;
pub mod common;
pub mod config;
pub mod error;


use std::convert::TryInto;

use tracing::warn;

pub use self::adts::AudioDataTransportStream;
use self::config::AudioSpecificConfiguration;
pub use self::error::AacError;
use crate::{FormatReader, FormatWriter, ReadFormat, WriteFormat};


pub struct Aac(Vec<u8>);

impl From<&[u8]> for Aac {
    fn from(val: &[u8]) -> Self {
        Self(Vec::from(val))
    }
}

impl From<Aac> for Vec<u8> {
    fn from(val: Aac) -> Self {
        val.0
    }
}

pub struct Raw;

impl ReadFormat<Aac> for Raw {
    type Context = ();
    type Error = AacError;

    fn read_format(&self, input: &[u8], _ctx: &Self::Context) -> Result<Aac, Self::Error> {
        Ok(input.into())
    }
}


enum State {
    Initializing,
    Ready(AudioSpecificConfiguration),
}

pub struct AacCoder {
    state: State,
}

impl AacCoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_asc<A>(&mut self, asc: A) -> Result<(), AacError>
    where
        A: TryInto<AudioSpecificConfiguration, Error = AacError>,
    {
        self.state = State::Ready(asc.try_into()?);
        Ok(())
    }
}

impl Default for AacCoder {
    fn default() -> Self {
        Self {
            state: State::Initializing,
        }
    }
}

impl FormatReader<Raw> for AacCoder {
    type Error = AacError;
    type Output = Aac;

    fn read_format(
        &mut self,
        format: Raw,
        input: &[u8],
    ) -> Result<Option<Self::Output>, Self::Error> {
        Ok(match &self.state {
            State::Initializing => {
                warn!("AAC reader was not initialized, trying to initialize from current payload");
                self.set_asc(input)?;
                None
            }
            State::Ready(_) => Some(format.read_format(input, &())?),
        })
    }
}

impl FormatWriter<AudioDataTransportStream> for AacCoder {
    type Error = AacError;
    type Input = Aac;

    fn write_format(
        &mut self,
        format: AudioDataTransportStream,
        input: Self::Input,
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(match &self.state {
            State::Initializing => return Err(AacError::NotInitialized),
            State::Ready(asc) => format.write_format(input, asc)?,
        })
    }
}

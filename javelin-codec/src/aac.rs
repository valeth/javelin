pub mod error;
pub mod common;
pub mod adts;
pub mod config;


use {
    std::convert::TryInto,
    self::{
        config::AudioSpecificConfiguration,
    },
    crate::{FormatReader, FormatWriter, ReadFormat, WriteFormat},
};
pub use self::{
    error::AacError,
    adts::AudioDataTransportStream,
};


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
        where A: TryInto<AudioSpecificConfiguration, Error=AacError>
    {
        self.state = State::Ready(asc.try_into()?);
        Ok(())
    }
}

impl Default for AacCoder {
    fn default() -> Self {
        Self { state: State::Initializing }
    }
}

impl FormatReader<Raw> for AacCoder {
    type Output = Aac;
    type Error = AacError;

    fn read_format(&mut self, format: Raw, input: &[u8]) -> Result<Option<Self::Output>, Self::Error> {
        Ok(match &self.state {
            State::Initializing => {
                log::warn!("AAC reader was not initialized, trying to initialize from current payload");
                self.set_asc(input)?;
                None
            },
            State::Ready(_) => {
                Some(format.read_format(input, &())?)
            }
        })
    }
}

impl FormatWriter<AudioDataTransportStream> for AacCoder {
    type Input = Aac;
    type Error = AacError;

    fn write_format(&mut self, format: AudioDataTransportStream, input: Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(match &self.state {
            State::Initializing => return Err(AacError::NotInitialized),
            State::Ready(asc) => format.write_format(input, asc)?
        })
    }
}

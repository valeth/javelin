use log::debug;
use bytes::{Bytes, Buf, IntoBuf};
use super::{
    config::AudioSpecificConfiguration,
    adts::AudioDataTransportStream,
};
use crate::{
    Error,
    Result,
    SharedState,
};


#[derive(Debug, Clone, PartialEq, Eq)]
enum PacketType {
    SequenceHeader,
    AacRaw,
    Unknown(u8),
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketType::SequenceHeader,
            1 => PacketType::AacRaw,
            _ => PacketType::Unknown(value),
        }
    }
}


#[derive(Debug)]
pub struct Packet {
    packet_type: PacketType,
    timestamp: u64,
    payload: AudioDataTransportStream,
}

impl Packet {
    pub fn try_from_bytes<B>(bytes: B, timestamp: u64, shared: &SharedState) -> Result<Self>
        where B: IntoBuf
    {
        let mut buf = bytes.into_buf();

        // drop first byte
        buf.get_u8();

        let packet_type = PacketType::from(buf.get_u8());

        let config = if packet_type == PacketType::SequenceHeader {
            debug!("Received audio sequence header");
            let mut asc = shared.asc.write();
            *asc = Some(AudioSpecificConfiguration::try_from_buf(&mut buf)?);
            asc.clone()
        } else {
            shared.asc.read().clone()
        };

        let config = config.ok_or(Error::AudioSpecificConfigurationMissing)?;
        let payload = AudioDataTransportStream::new(buf.collect(), config);

        Ok(Self {
            packet_type,
            payload,
            timestamp
        })
    }

    pub fn to_bytes(&self) -> Bytes {
        self.payload.clone().into()
    }

    pub fn presentation_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn is_sequence_header(&self) -> bool {
        self.packet_type == PacketType::SequenceHeader
    }
}

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


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioChannel {
    Mono,
    Stereo,
}


/// Bits | Description
/// ---- | -----------
/// 4    | Codec ID
/// 2    | Sample rate
/// 1    | Sample size
/// 1    | Channel type
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub rate: u32,
    pub sample_size: u8,
    pub channels: AudioChannel,
}

impl Metadata {
    pub fn from_buf<B>(buf: &mut B) -> Self
        where B: Buf
    {
        let header = buf.get_u8();
        let codec_id = header >> 4;
        assert_eq!(codec_id, 10, "Not an AAC bitstream, got id {}", codec_id);

        let rate = match header & 0b0000_1100 {
            0b0000_0000 => 5500,
            0b0000_0100 => 11000,
            0b0000_1000 => 22000,
            0b0000_1100 => 44000,
            _ => unreachable!(),
        };

        let sample_size = match header & 0b0000_0010 {
            0b0000_0000 => 8,
            0b0000_0010 => 16,
            _ => unreachable!(),
        };

        let channels = match header & 0b0000_0001 {
            0b0000_0000 => AudioChannel::Mono,
            0b0000_0001 => AudioChannel::Stereo,
            _ => unreachable!(),
        };

        Self { rate, sample_size, channels }
    }

    #[allow(dead_code)] // required for test
    pub fn to_bytes(&self) -> Bytes {
        let mut header = 0b1010_0000u8;

        header |= match self.channels {
            AudioChannel::Mono   => 0b0000_0000,
            AudioChannel::Stereo => 0b0000_0001,
        };

        header |= match self.sample_size {
            8  => 0b0000_0000,
            16 => 0b0000_0010,
            _ => unreachable!(),
        };

        header |= match self.rate {
            5500  => 0b0000_0000,
            11000 => 0b0000_0100,
            22000 => 0b0000_1000,
            44000 => 0b0000_1100,
            _ => unreachable!(),
        };

        Bytes::from(&[header][..])
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

        let metadata = Metadata::from_buf(&mut buf);
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
        let payload = AudioDataTransportStream::new(buf.collect(), metadata, config);

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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_metadata_header() {
        let expected = Metadata { rate: 44000, sample_size: 16, channels: AudioChannel::Stereo };
        let actual = Metadata::from_buf(&mut [0b1010_1111][..].into_buf());
        assert_eq!(expected, actual);

        let expected = Metadata { rate: 5500, sample_size: 8, channels: AudioChannel::Mono };
        let actual = Metadata::from_buf(&mut [0b1010_0000][..].into_buf());
        assert_eq!(expected, actual);

        let expected = Metadata { rate: 22000, sample_size: 16, channels: AudioChannel::Stereo };
        let actual = Metadata::from_buf(&mut [0b1010_1011][..].into_buf());
        assert_eq!(expected, actual);
    }

    #[should_panic]
    #[test]
    fn rejects_non_aac_header() {
        Metadata::from_buf(&mut [0b1000_0000][..].into_buf());
    }

    #[test]
    fn can_convert_metadata_into_bytes() {
        let expected = Bytes::from_static(&[0b1010_1111]);
        let actual = Metadata::from_buf(&mut expected.clone().into_buf()).to_bytes();
        assert_eq!(expected, actual);
    }
}

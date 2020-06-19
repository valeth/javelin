use {
    std::{
        convert::TryFrom,
        fmt::{self, Debug},
        io::{Cursor, Read},
    },
    bytes::{Bytes, Buf},
    crate::flv::error::FlvError,
};


/// Frequency value in Hertz
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Frequency(u32);


#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    Aac,
}

impl TryFrom<u8> for AudioFormat {
    type Error = FlvError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        // only needs to have support for AAC right now
        if val == 10 {
            Ok(Self::Aac)
        } else {
            Err(FlvError::UnsupportedAudioFormat(val))
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AacPacketType {
    SequenceHeader,
    Raw,
    None,
}

impl TryFrom<u8> for AacPacketType {
    type Error = FlvError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
            0 => Self::SequenceHeader,
            1 => Self::Raw,
            x => return Err(FlvError::UnknownPackageType(x))
        })
    }
}


// Field                | Type
// -------------------- | ---
// Audio Format         | u4
// Sampling Rate        | u4
// Sampling Size        | u2
// Stereo Flag          | u1
// AAC Packet Type      | u8
// Body                 | [u8]
#[derive(Clone)]
pub struct AudioData {
    pub format: AudioFormat,
    pub sampling_rate: Frequency,
    pub sample_size: u8,
    pub stereo: bool,
    pub aac_packet_type: AacPacketType,
    pub body: Bytes,
}

impl AudioData {
    pub fn is_sequence_header(&self) -> bool {
        self.aac_packet_type == AacPacketType::SequenceHeader
    }
}

impl TryFrom<&[u8]> for AudioData {
    type Error = FlvError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 3 {
            return Err(FlvError::NotEnoughData("FLV Audio Tag header"))
        }

        let mut buf = Cursor::new(bytes);

        let header = buf.get_u8();
        let format = AudioFormat::try_from(header >> 4)?;
        let sampling_rate = try_convert_sampling_rate((header >> 2) & 0x02)?;
        let sample_size = try_convert_sample_size((header >> 1) & 0x01)?;
        let stereo = (header & 0x01) == 1;

        let aac_packet_type = if format == AudioFormat::Aac {
            AacPacketType::try_from(buf.get_u8())?
        } else {
            AacPacketType::None
        };

        let mut body = Vec::new();
        buf.read_to_end(&mut body)?;

        Ok(Self { format, sampling_rate, sample_size, stereo, aac_packet_type, body: body.into() })
    }
}

impl Debug for AudioData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioData")
            .field("format", &self.format)
            .field("sampling_rate", &self.sampling_rate)
            .field("sample_size", &self.sample_size)
            .field("stereo", &self.stereo)
            .field("aac_packet_type", &self.aac_packet_type)
            .finish()
    }
}


fn try_convert_sampling_rate(val: u8) -> Result<Frequency, FlvError> {
    Ok(match val {
        0 => Frequency(5500),
        1 => Frequency(11000),
        2 => Frequency(22000),
        3 => Frequency(44000),
        x => return Err(FlvError::UnsupportedSamplingRate(x))
    })
}

fn try_convert_sample_size(val: u8) -> Result<u8, FlvError> {
    Ok(match val {
        0 => 8,
        1 => 16,
        x => return Err(FlvError::UnsupportedSampleSize(x))
    })
}

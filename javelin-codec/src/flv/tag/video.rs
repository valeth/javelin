use {
    std::{
        convert::TryFrom,
        io::{Cursor, Read},
        fmt::{self, Debug},
    },
    bytes::{Bytes, Buf},
    crate::flv::error::FlvError,
};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameType {
    KeyFrame,
    InterFrame,
    DisposableInterFrame,
    GeneratedKeyframe,
    VideoInfoFrame,
}

impl TryFrom<u8> for FrameType {
    type Error = FlvError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
            1 => Self::KeyFrame,
            2 => Self::InterFrame,
            3 => Self::DisposableInterFrame,
            4 => Self::GeneratedKeyframe,
            5 => Self::VideoInfoFrame,
            x => return Err(FlvError::UnknownFrameType(x))
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvcPacketType {
    SequenceHeader,
    NalUnit,
    EndOfSequence,
    None,
}

impl TryFrom<u8> for AvcPacketType {
    type Error = FlvError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
            0 => Self::SequenceHeader,
            1 => Self::NalUnit,
            2 => Self::EndOfSequence,
            x => return Err(FlvError::UnknownPackageType(x))
        })
    }
}


// Field                | Type
// -------------------- | ---
// Frame Type           | u4
// Codec ID             | u4
// AVC Packet Type      | u8
// Composition Time     | i24
// Body                 | [u8]
#[derive(Clone)]
pub struct VideoData {
    pub frame_type: FrameType,
    pub packet_type: AvcPacketType,
    pub composition_time: i32,
    pub body: Bytes,
}

impl VideoData {
    pub fn is_sequence_header(&self) -> bool {
        self.packet_type == AvcPacketType::SequenceHeader
    }

    pub fn is_keyframe(&self) -> bool {
        self.frame_type == FrameType::KeyFrame
    }
}

impl Debug for VideoData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Video")
            .field("frame_type", &self.frame_type)
            .field("packet_type", &self.packet_type)
            .field("composition_time", &self.composition_time)
            .finish()
    }
}

impl TryFrom<&[u8]> for VideoData {
    type Error = FlvError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 6 {
            return Err(FlvError::NotEnoughData("FLV Video Tag header"))
        }

        let mut buf = Cursor::new(bytes);

        let header_a = buf.get_u8();

        // Only support AVC payloads
        let codec_id = header_a & 0x0F;
        if codec_id != 7 {
            return Err(FlvError::UnsupportedVideoFormat(codec_id))
        }

        let frame_type = FrameType::try_from(header_a >> 4)?;

        let header_b = buf.get_u32();

        let packet_type =  AvcPacketType::try_from((header_b >> 24) as u8)?;

        let composition_time = (header_b & 0x00_FF_FF_FF) as i32;

        let mut remaining = Vec::new();
        buf.read_to_end(&mut remaining)?;

        Ok(Self { frame_type, packet_type, composition_time, body: remaining.into() })
    }
}

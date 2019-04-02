use log::debug;
use bytes::{Bytes, Buf, IntoBuf};
use super::{
    dcr::DecoderConfigurationRecord,
    bitstream::Bitstream,
};
use crate::{
    SharedState,
    Error,
    Result,
};


#[derive(Debug, Clone, PartialEq, Eq)]
enum PacketType {
    SequenceHeader,
    NalUnit,
    EndOfSequence,
    Keyframe,
    Unknown(u8)
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketType::SequenceHeader,
            1 => PacketType::NalUnit,
            2 => PacketType::EndOfSequence,
            4 => PacketType::Keyframe,
            _ => PacketType::Unknown(value),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameType {
    Keyframe,
    InterFrame,
    DisposableInterFrame,
    GeneratedKeyframe,
    VideoInfoFrame,
    Unknown(u8),
}

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        match value {
            1 => FrameType::Keyframe,
            2 => FrameType::InterFrame,
            3 => FrameType::DisposableInterFrame,
            4 => FrameType::GeneratedKeyframe,
            5 => FrameType::VideoInfoFrame,
            _ => FrameType::Unknown(value),
        }
    }
}


/// AVC encoded byte chunk
///
/// Bits | Name
/// ---- | ----
/// 4    | Frame Type
/// 4    | Codec ID
/// 8    | Packet Type
/// 24   | Composition Time
/// var  | [Decoder Configuration Record](struct.DecoderConfigurationRecord.html)
/// var  | [NALU](../nalu/struct.Unit.html)
///
#[derive(Debug)]
pub struct Packet {
    frame_type: FrameType,
    packet_type: PacketType,
    composition_time: u32,
    nal_units: Bitstream,
    timestamp: u64,
}

impl Packet {
    pub fn try_from_buf<B>(bytes: B, timestamp: u64, shared: &SharedState) -> Result<Self>
        where B: IntoBuf
    {
        let mut buf = bytes.into_buf();

        let tmp = buf.get_u8();
        let frame_type = FrameType::from(tmp >> 4);
        let codec_id = tmp & 0x0F;
        assert!(codec_id == 7);

        let tmp = buf.get_u32_be();
        let packet_type = PacketType::from((tmp >> 24) as u8);
        let composition_time = tmp & 0x00_FF_FF_FF;

        if packet_type == PacketType::SequenceHeader {
            debug!("Received video sequence header");
            let mut dcr = shared.dcr.write();
            *dcr = Some(DecoderConfigurationRecord::try_from_buf(&mut buf)?);
        }

        let dcr = shared.dcr.read().clone().ok_or(Error::DecoderConfigurationRecordMissing)?;
        let nal_units = Bitstream::try_from_buf(buf, dcr.clone())?;

        Ok(Self {
            frame_type,
            packet_type,
            composition_time,
            nal_units,
            timestamp,
        })
    }

    pub fn try_as_bytes(&self) -> Result<Bytes> {
        self.nal_units.try_as_bytes()
    }

    pub fn is_sequence_header(&self) -> bool {
        self.packet_type == PacketType::SequenceHeader
    }

    pub fn is_keyframe(&self) -> bool {
        self.frame_type == FrameType::Keyframe || self.frame_type == FrameType::GeneratedKeyframe
    }

    pub fn presentation_timestamp(&self) -> u64 {
        self.timestamp + (self.composition_time as u64)
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

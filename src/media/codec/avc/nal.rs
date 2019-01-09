use std::fmt;
use bytes::{Bytes, BytesMut, IntoBuf};
use crate::{Error, Result};


#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum UnitType {
    NonIdrPicture = 1,
    DataPartitionA = 2,
    DataPartitionB = 3,
    DataPartitionC = 4,
    IdrPicture = 5,
    SupplementaryEnhancementInformation = 6,
    SequenceParameterSet = 7,
    PictureParameterSet = 8,
    AccessUnitDelimiter = 9,
    SequenceEnd = 10,
    StreamEnd = 11,
    FillerData = 12,
    SequenceParameterSetExtension = 13,
    Prefix = 14,
    SequenceParameterSubset = 15,
    NotAuxiliaryCoded = 19,
    CodedSliceExtension = 20,
}

impl UnitType {
    fn try_from(value: u8) -> Result<Self> {
        let val = match value {
            1 => UnitType::NonIdrPicture,
            2 => UnitType::DataPartitionA,
            3 => UnitType::DataPartitionB,
            4 => UnitType::DataPartitionC,
            5 => UnitType::IdrPicture,
            6 => UnitType::SupplementaryEnhancementInformation,
            7 => UnitType::SequenceParameterSet,
            8 => UnitType::PictureParameterSet,
            9 => UnitType::AccessUnitDelimiter,
            10 => UnitType::SequenceEnd,
            11 => UnitType::StreamEnd,
            12 => UnitType::FillerData,
            13 => UnitType::SequenceParameterSetExtension,
            14 => UnitType::Prefix,
            15 => UnitType::SequenceParameterSubset,
            19 => UnitType::NotAuxiliaryCoded,
            20 => UnitType::CodedSliceExtension,
            16 | 17 | 18 | 22 | 23 => {
                return Err(Error::ParseError(format!("Reserved NAL unit type {}", value)));
            },
            _ => {
                return Err(Error::ParseError(format!("Unknown NAL unit type {}", value)));
            },
        };

        Ok(val)
    }
}


/// Network Abstraction Layer Unit (aka NALU) of a H.264 bitstream.
#[derive(Clone, PartialEq, Eq)]
pub struct Unit {
    pub ref_idc: u8,
    pub kind: UnitType,
    pub data: Bytes, // Raw Byte Sequence Payload (RBSP)
}

impl Unit {
    pub fn try_from_bytes(bytes: Bytes) -> Result<Self> {
        use bytes::Buf;

        let mut buf = bytes.into_buf();
        let header = buf.get_u8();
        assert_eq!(header >> 7, 0);
        let ref_idc = (header >> 5) & 0x03;
        let kind = UnitType::try_from(header & 0x1F)?;
        let data: Bytes = buf.collect();

        Ok(Self { ref_idc, kind, data })
    }
}

impl Into<Bytes> for Unit {
    fn into(self) -> Bytes {
        use bytes::BufMut;

        let mut tmp = BytesMut::with_capacity(self.data.len() + 1);

        let header = (self.ref_idc << 5) | (self.kind as u8);
        tmp.put_u8(header);
        tmp.put(self.data);

        tmp.freeze()
    }
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Unit")
            .field("ref_idc", &self.ref_idc)
            .field("kind", &self.kind)
            .finish()
    }
}

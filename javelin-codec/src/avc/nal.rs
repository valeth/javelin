use {
    std::{
        fmt,
        io::Cursor,
        convert::TryFrom,
    },
    bytes::{Bytes, Buf, BufMut},
    super::AvcError,
};


#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
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

impl TryFrom<u8> for UnitType {
    type Error = AvcError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
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
            _ => {
                return Err(AvcError::UnsupportedNalUnitType(val))
            },
        })
    }
}


/// Network Abstraction Layer Unit (aka NALU) of a H.264 bitstream.
#[derive(Clone, PartialEq, Eq)]
pub struct Unit {
    pub kind: UnitType,
    ref_idc: u8,
    data: Bytes, // Raw Byte Sequence Payload (RBSP)
}

impl Unit {
    pub fn payload(&self) -> &[u8] {
        &self.data
    }
}

impl TryFrom<&[u8]> for Unit {
    type Error = AvcError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut buf = Cursor::new(bytes);

        let header = buf.get_u8();
        // FIXME: return error
        assert_eq!(header >> 7, 0);

        let ref_idc = (header >> 5) & 0x03;
        let kind = UnitType::try_from(header & 0x1F)?;
        let data = buf.to_bytes();

        Ok(Self { ref_idc, kind, data })
    }
}

impl From<&Unit> for Vec<u8> {
    fn from(val: &Unit) -> Self {
        let mut tmp = Vec::with_capacity(val.data.len() + 1);

        let header = (val.ref_idc << 5) | (val.kind as u8);
        tmp.put_u8(header);
        tmp.put(val.data.clone());
        tmp
    }
}

impl From<Unit> for Vec<u8> {
    fn from(val: Unit) -> Self {
        Self::from(&val)
    }
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unit")
            .field("kind", &self.kind)
            .finish()
    }
}

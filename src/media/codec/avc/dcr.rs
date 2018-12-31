use bytes::{Bytes, Buf};
use super::nal;
use crate::Error;


/// AVC decoder configuration record
///
/// Bits | Name
/// ---- | ----
/// 8    | Version
/// 8    | Profile Indication
/// 8    | Profile Compatability
/// 8    | Level Indication
/// 6    | Reserved
/// 2    | NALU Length
/// 3    | Reserved
/// 5    | SPS Count
/// 16   | SPS Length
/// var  | SPS
/// 8    | PPS Count
/// 16   | PPS Length
/// var  | PPS
///
#[derive(Debug, Clone)]
pub struct DecoderConfigurationRecord {
    pub version: u8,
    pub profile_indication: u8,
    pub profile_compatability: u8,
    pub level_indication: u8,
    pub nalu_size: u8,
    pub sps: Vec<nal::Unit>,
    pub pps: Vec<nal::Unit>,
}

impl DecoderConfigurationRecord {
    pub fn try_from_buf<B>(buf: &mut B) -> Result<Self, Error>
        where B: Buf
    {
        if buf.remaining() < 7 {
            return Err(Error::NotEnoughData)
        }

        let version = buf.get_u8();
        if version != 1 {
            return Err(Error::UnsupportedConfigurationRecordVersion(version));
        }

        let profile_indication = buf.get_u8();
        let profile_compatability = buf.get_u8();
        let level_indication = buf.get_u8();
        let nalu_size = (buf.get_u8() & 0x03) + 1;

        let sps_count = buf.get_u8() & 0x1F;
        let mut sps = Vec::new();
        for _ in 0..sps_count {
            let sps_length = buf.get_u16_be() as usize;
            let tmp: Bytes = buf.by_ref().take(sps_length).collect();
            sps.push(nal::Unit::from(tmp));
        }

        let pps_count = buf.get_u8();
        let mut pps = Vec::new();
        for _ in 0..pps_count {
            let pps_length = buf.get_u16_be() as usize;
            let tmp: Bytes = buf.by_ref().take(pps_length).collect();
            pps.push(nal::Unit::from(tmp));
        }

        Ok(Self {
            version,
            profile_indication,
            profile_compatability,
            level_indication,
            nalu_size,
            sps,
            pps,
        })
    }
}

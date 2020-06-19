use {
    std::{
        convert::TryFrom,
        io::Cursor,
    },
    bytes::Buf,
    super::{
        nal,
        AvcError,
    },
};


// Bits | Name
// ---- | ----
// 8    | Version
// 8    | Profile Indication
// 8    | Profile Compatability
// 8    | Level Indication
// 6    | Reserved
// 2    | NALU Length
// 3    | Reserved
// 5    | SPS Count
// 16   | SPS Length
// var  | SPS
// 8    | PPS Count
// 16   | PPS Length
// var  | PPS
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

impl TryFrom<&[u8]> for DecoderConfigurationRecord {
    type Error = AvcError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // FIXME: add checks before accessing buf, otherwise could panic
        let mut buf = Cursor::new(bytes);

        if buf.remaining() < 7 {
            return Err(AvcError::NotEnoughData("AVC configuration record"))
        }

        let version = buf.get_u8();
        if version != 1 {
            return Err(AvcError::UnsupportedConfigurationRecordVersion(version));
        }

        let profile_indication = buf.get_u8();
        let profile_compatability = buf.get_u8();
        let level_indication = buf.get_u8();
        let nalu_size = (buf.get_u8() & 0x03) + 1;

        let sps_count = buf.get_u8() & 0x1F;
        let mut sps = Vec::new();
        for _ in 0..sps_count {
            if buf.remaining() < 2 {
                return Err(AvcError::NotEnoughData("DCR SPS length"));
            }
            let sps_length = buf.get_u16() as usize;

            if buf.remaining() < sps_length {
                return Err(AvcError::NotEnoughData("DCR SPS data"));
            }
            let tmp = buf.bytes()[..sps_length].to_owned();
            buf.advance(sps_length);

            sps.push(nal::Unit::try_from(&*tmp)?);
        }

        let pps_count = buf.get_u8();
        let mut pps = Vec::new();
        for _ in 0..pps_count {
            if buf.remaining() < 2 {
                return Err(AvcError::NotEnoughData("DCR PPS length"));
            }
            let pps_length = buf.get_u16() as usize;

            if buf.remaining() < pps_length {
                return Err(AvcError::NotEnoughData("DCR PPS data"));
            }
            let tmp = buf.bytes()[..pps_length].to_owned();
            buf.advance(pps_length);

            pps.push(nal::Unit::try_from(&*tmp)?);
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

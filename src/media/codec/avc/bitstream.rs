use log::{warn, debug};
use bytes::{Bytes, BytesMut, IntoBuf};
use super::{
    dcr::DecoderConfigurationRecord,
    nal,
};
use crate::{utils, Error};


#[derive(Debug)]
pub struct Bitstream {
    dcr: DecoderConfigurationRecord,
    nal_units: Vec<nal::Unit>,
}

impl<'a> Bitstream {
    const DELIMITER1: &'a [u8] = &[0x00, 0x00, 0x01];
    const DELIMITER2: &'a [u8] = &[0x00, 0x00, 0x00, 0x01];
    const ACCESS_UNIT_DELIMITER: &'a [u8] = &[0x00, 0x00, 0x00, 0x01, 0x09, 0xF0];

    pub fn try_from_buf<B>(bytes: B, dcr: DecoderConfigurationRecord) -> Result<Self, Error>
        where B: IntoBuf,
    {
        use bytes::Buf;

        let mut buf = bytes.into_buf();
        let mut nal_units = Vec::new();

        while buf.has_remaining() {
            let nalu_length = utils::bytes_as_usize_be(dcr.nalu_size as usize, &mut buf);
            let nalu_data: Bytes = buf.by_ref().take(nalu_length).collect();
            nal_units.push(nal::Unit::from(nalu_data))
        };

        if buf.has_remaining() {
            warn!("{} bytes remaining in buffer", buf.remaining());
        }

        Ok(Self { nal_units, dcr })
    }

    pub fn try_as_bytes(&self) -> Result<Bytes, Error> {
        use self::nal::UnitType;

        let mut tmp = BytesMut::new();
        let mut aud_appended = false;
        let mut sps_and_pps_appended = false;
        let nalus = self.nal_units.clone();
        let dcr = &self.dcr;

        for nalu in nalus {
            match &nalu.kind {
                | UnitType::SequenceParameterSet
                | UnitType::PictureParameterSet
                | UnitType::AccessUnitDelimiter => {
                    continue;
                },
                | UnitType::NonIdrPicture
                | UnitType::SupplementaryEnhancementInformation => {
                    if !aud_appended {
                        tmp.extend(Self::ACCESS_UNIT_DELIMITER);
                        aud_appended = true;
                    }
                },
                UnitType::IdrPicture => {
                    if !aud_appended {
                        tmp.extend(Self::ACCESS_UNIT_DELIMITER);
                        aud_appended = true;
                    }

                    if !sps_and_pps_appended {
                        if !dcr.sps.is_empty() {
                            tmp.extend(Self::DELIMITER2);
                            let unit: Bytes = dcr.sps.first().unwrap().clone().into();
                            tmp.extend(unit);
                        }

                        if !dcr.pps.is_empty() {
                            tmp.extend(Self::DELIMITER2);
                            let unit: Bytes = dcr.pps.first().unwrap().clone().into();
                            tmp.extend(unit);
                        }

                        sps_and_pps_appended = true;
                    }
                },
                t => debug!("Received unhandled NALU type {:?}", t),

            }

            if nalu.data.len() < 5 {
                return Err(Error::NotEnoughData);
            }

            tmp.extend(Self::DELIMITER1);
            let nalu_data: Bytes = nalu.into();
            tmp.extend(nalu_data);
        }

        Ok(tmp.freeze())
    }
}

#[cfg(feature = "try_from")]
impl<'a, B> TryFrom<(B, DecoderConfigurationRecord)> for Bitstream<'a>
    where B: IntoBuf
{
    type Error = Error;

    fn try_from(value: (B, DecoderConfigurationRecord)) -> Result<Self, Self::Error> {
        Self::try_from_buf(value.0, value.1)
    }
}

#[cfg(feature = "try_from")]
impl<'a> TryInto<Bytes> for Bitstream<'a> {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        self.try_into_bytes()
    }
}

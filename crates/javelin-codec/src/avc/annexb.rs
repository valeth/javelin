use {
    crate::{
        avc::{Avc, error::AvcError, nal, config::DecoderConfigurationRecord},
        WriteFormat,
    },
};

pub struct AnnexB;

impl AnnexB {
    const DELIMITER1: &'static [u8] = &[0x00, 0x00, 0x01];
    const DELIMITER2: &'static [u8] = &[0x00, 0x00, 0x00, 0x01];
    const ACCESS_UNIT_DELIMITER: &'static [u8] = &[0x00, 0x00, 0x00, 0x01, 0x09, 0xF0];
}

impl WriteFormat<Avc> for AnnexB {
    type Context = DecoderConfigurationRecord;
    type Error = AvcError;

    fn write_format(&self, input: Avc, ctx: &Self::Context) -> Result<Vec<u8>, Self::Error> {
        let mut out_buffer = Vec::new();
        let mut aud_appended = false;
        let mut sps_and_pps_appended = false;
        let nalus: Vec<nal::Unit> = input.into();

        for nalu in nalus {
            use nal::UnitType::*;

            match &nalu.kind {
                SequenceParameterSet | PictureParameterSet | AccessUnitDelimiter => continue,
                NonIdrPicture | SupplementaryEnhancementInformation => {
                    if !aud_appended {
                        out_buffer.extend(Self::ACCESS_UNIT_DELIMITER);
                        aud_appended = true;
                    }
                },
                IdrPicture => {
                    if !aud_appended {
                        out_buffer.extend(Self::ACCESS_UNIT_DELIMITER);
                        aud_appended = true;
                    }

                    if !sps_and_pps_appended {
                        if let Some(sps) = ctx.sps.first() {
                            out_buffer.extend(Self::DELIMITER2);
                            let tmp: Vec<u8> = sps.into();
                            out_buffer.extend(tmp);
                        }

                        if let Some(pps) = ctx.pps.first() {
                            out_buffer.extend(Self::DELIMITER2);
                            let tmp: Vec<u8> = pps.into();
                            out_buffer.extend(tmp);
                        }

                        sps_and_pps_appended = true;
                    }
                },
                t => log::debug!("Received unhandled NALU type {:?}", t),
            }

            out_buffer.extend(Self::DELIMITER1);

            let nalu_data: Vec<u8> = nalu.into();
            out_buffer.extend(nalu_data);
        }

        Ok(out_buffer)
    }
}

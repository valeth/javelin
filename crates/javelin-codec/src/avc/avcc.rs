use std::convert::TryFrom;
use std::io::Cursor;

use bytes::Buf;

use crate::avc::config::DecoderConfigurationRecord;
use crate::avc::error::AvcError;
use crate::avc::{nal, Avc};
use crate::ReadFormat;


pub struct Avcc;

impl ReadFormat<Avc> for Avcc {
    type Context = DecoderConfigurationRecord;
    type Error = AvcError;

    fn read_format(&self, input: &[u8], ctx: &Self::Context) -> Result<Avc, Self::Error> {
        let mut buf = Cursor::new(input);
        let mut nal_units = Vec::new();

        while buf.has_remaining() {
            let unit_size = ctx.nalu_size as usize;

            if buf.remaining() < unit_size {
                return Err(AvcError::NotEnoughData("NALU size"));
            }
            let nalu_length = buf.get_uint(unit_size) as usize;

            let nalu_data = buf
                .chunk()
                .get(..nalu_length)
                .ok_or(AvcError::NotEnoughData("NALU data"))?
                .to_owned();

            buf.advance(nalu_length);

            let nal_unit = nal::Unit::try_from(&*nalu_data)?;
            nal_units.push(nal_unit);
        }

        Ok(nal_units.into())
    }
}

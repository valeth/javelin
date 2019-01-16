use bytes::Buf;
use byteorder::{ReadBytesExt, BigEndian};
use crate::{Error, Result};


pub fn try_bytes_as_usize_be<B: Buf>(length: usize, buf: &mut B) -> Result<usize> {
    assert!(length <= 8, "Expected maximum size of 8, got {}", length);

    let val = buf
        .by_ref()
        .reader()
        .read_uint::<BigEndian>(length)
        .map_err(|_| Error::ParseError("Failed to parse bytes as integer".into()))?;

    Ok(val as usize)
}

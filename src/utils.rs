use bytes::Buf;
use byteorder::{ReadBytesExt, BigEndian};


pub fn bytes_as_usize_be<B: Buf>(length: usize, buf: &mut B) -> usize {
    assert!(length <= 8, "Expected maximum size of 8, got {}", length);
    buf.by_ref().reader().read_uint::<BigEndian>(length).unwrap() as usize
}

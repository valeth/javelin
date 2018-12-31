use bytes::Buf;
use byteorder::{ReadBytesExt, BigEndian};


pub fn bytes_as_u32_be<B: Buf>(length: usize, buf: &mut B) -> u32 {
    assert!(length <= 4, "Expected maximum size of 4, got {}", length);
    bytes_as_usize_be(length, buf) as u32
}


pub fn bytes_as_usize_be<B: Buf>(length: usize, buf: &mut B) -> usize {
    assert!(length <= 8, "Expected maximum size of 8, got {}", length);
    buf.by_ref().reader().read_uint::<BigEndian>(length).unwrap() as usize
}


#[cfg(test)]
mod tests {
    use bytes::{Bytes, IntoBuf};
    use super::*;

    #[test]
    fn test_bytes_conversion() {
        let mut buf = Bytes::from(vec![0u8, 64]).into_buf();
        assert_eq!(64u32, bytes_as_u32_be(2, &mut buf));

        let mut buf = Bytes::from(vec![0u8, 1, 64]).into_buf();
        assert_eq!(320u32, bytes_as_u32_be(3, &mut buf));

        let mut buf = Bytes::from(vec![0u8, 1, 1, 64]).into_buf();
        assert_eq!(65856u32, bytes_as_u32_be(4, &mut buf));
    }
}

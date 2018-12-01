use bytes::Bytes;


#[derive(Debug, PartialEq, Eq)]
pub enum Media {
    AAC(Bytes),
    H264(Bytes),
}

impl Media {
    pub fn is_sequence_header(&self) -> bool {
        match self {
            Media::AAC(ref bytes) => {
                bytes.len() >= 2 && bytes[0] == 0xaf && bytes[1] == 0x00
            },
            Media::H264(ref bytes) => {
                bytes.len() >= 2 && bytes[0] == 0x17 && bytes[1] == 0x00
            },
        }
    }

    pub fn is_keyframe(&self) -> bool {
        match self {
            Media::H264(bytes) => {
                bytes.len() >= 2 && bytes[0] == 0x17 && bytes[1] != 0x00
            }
            _ => false
        }
    }

    pub fn is_sendable(&self) -> bool {
        self.is_sequence_header() || self.is_keyframe()
    }
}

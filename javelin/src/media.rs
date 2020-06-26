use {
    bytes::Bytes,
    rml_rtmp::time::RtmpTimestamp,
};

#[cfg(feature = "hls")]
use futures::sync::mpsc;


#[cfg(feature = "hls")]
pub type Receiver = mpsc::UnboundedReceiver<Media>;
#[cfg(feature = "hls")]
pub type Sender = mpsc::UnboundedSender<Media>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Media {
    AAC(RtmpTimestamp, Bytes),
    H264(RtmpTimestamp, Bytes),
}

impl Media {
    pub fn is_sequence_header(&self) -> bool {
        match self {
            Media::AAC(_, ref bytes) => {
                bytes.len() >= 2 && bytes[0] == 0xaf && bytes[1] == 0x00
            },
            Media::H264(_, ref bytes) => {
                bytes.len() >= 2 && bytes[0] == 0x17 && bytes[1] == 0x00
            },
        }
    }

    pub fn is_keyframe(&self) -> bool {
        match self {
            Media::H264(_, bytes) => {
                bytes.len() >= 2 && bytes[0] == 0x17 && bytes[1] != 0x00
            }
            _ => false
        }
    }

    pub fn is_sendable(&self) -> bool {
        self.is_sequence_header() || self.is_keyframe()
    }
}

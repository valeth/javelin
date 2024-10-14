use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::Timestamp;


// FIXME: remove temporary content id values
pub const METADATA: ContentType = ContentType::new(0);
pub const FLV_VIDEO_H264: ContentType = ContentType::new(1);
pub const FLV_AUDIO_AAC: ContentType = ContentType::new(2);
pub const CONTAINER_MPEGTS: ContentType = ContentType::new(3);


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentType(u32);

impl ContentType {
    const fn new(content_id: u32) -> Self {
        Self(content_id)
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Packet {
    pub content_type: ContentType,
    pub timestamp: Option<Timestamp>,
    pub payload: Bytes,
}

impl Packet {
    pub fn new<T, B>(content_type: ContentType, timestamp: Option<T>, payload: B) -> Self
    where
        T: Into<Timestamp>,
        B: Into<Bytes>,
    {
        Self {
            content_type,
            timestamp: timestamp.map(Into::into),
            payload: payload.into(),
        }
    }
}

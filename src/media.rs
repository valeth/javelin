pub mod codec;


use std::collections::HashSet;
use futures::sync::mpsc;
use bytes::Bytes;
use rml_rtmp::{
    sessions::StreamMetadata,
    time::RtmpTimestamp,
};


pub use self::codec::avc;


pub type Receiver = mpsc::UnboundedReceiver<Media>;
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

    pub fn len(&self) -> usize {
        match self {
            Media::AAC(_, bytes) => bytes.len(),
            Media::H264(_, bytes) => bytes.len(),
        }
    }

    pub fn timestamp(&self) -> u64 {
        match self {
            Media::AAC(timestamp, _) => u64::from(timestamp.value),
            Media::H264(timestamp, _) => u64::from(timestamp.value),
        }
    }
}


pub struct Channel {
    publisher: Option<u64>,
    stream_key: Option<String>,
    pub watchers: HashSet<u64>,
    pub metadata: Option<StreamMetadata>,
    pub video_seq_header: Option<Bytes>,
    pub audio_seq_header: Option<Bytes>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            publisher: None,
            stream_key: None,
            watchers: HashSet::new(),
            metadata: None,
            video_seq_header: None,
            audio_seq_header: None,
        }
    }

    pub fn has_publisher(&self) -> bool {
        self.publisher != None
    }

    pub fn set_publisher(&mut self, publisher_id: u64, stream_key: String) {
        self.publisher = Some(publisher_id);
        self.stream_key = Some(stream_key);
    }

    pub fn add_watcher(&mut self, watcher_id: u64) {
        self.watchers.insert(watcher_id);
    }

    pub fn unpublish(&mut self) {
        self.publisher = None;
        self.stream_key = None;
        self.metadata = None;
        self.video_seq_header = None;
        self.audio_seq_header = None;
    }

    pub fn set_metadata(&mut self, metadata: StreamMetadata) {
        self.metadata = Some(metadata)
    }
}

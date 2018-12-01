use std::collections::HashSet;
use bytes::Bytes;
use rtmp::sessions::StreamMetadata;


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
}

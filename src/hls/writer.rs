use log::{debug, error, warn};
use futures::try_ready;
use tokio::prelude::*;
use super::{
    transport_stream::Buffer as TsBuffer,
    m3u8::Playlist,
};
use crate::media::{self, Media, avc, aac};


pub struct Writer {
    receiver: media::Receiver,
    write_interval: u64,
    next_write: u64,
    last_keyframe: u64,
    keyframe_counter: usize,
    buffer: TsBuffer,
    shared_state: media::codec::SharedState,
    playlist: Playlist,
}

impl Writer {
    pub fn new(receiver: media::Receiver) -> Self {
        let write_interval = 2000; // milliseconds
        let next_write = write_interval; // milliseconds

        Self {
            receiver,
            write_interval,
            next_write,
            last_keyframe: 0,
            keyframe_counter: 0,
            buffer: TsBuffer::new(),
            shared_state: media::codec::SharedState::new(),
            // TODO: Same as TS filename, see below
            playlist: Playlist::new("./tmp/stream/playlist.m3u8"),
        }
    }
}


impl Future for Writer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(media) = try_ready!(self.receiver.poll()) {
            match media {
                Media::H264(timestamp, bytes) => {
                    let timestamp = u64::from(timestamp.value);

                    let packet = match avc::Packet::try_from_buf(bytes, timestamp, &self.shared_state) {
                        Err(why) => {
                            error!("Failed to build packet: {:?}", why);
                            continue;
                        },
                        Ok(p) => p
                    };

                    if packet.is_sequence_header() {
                        debug!("Received video sequence header");
                        continue;
                    }

                    if packet.is_keyframe() {
                        let keyframe_duration = timestamp - self.last_keyframe;

                        if self.keyframe_counter == 1 {
                            self.playlist.set_target_duration(keyframe_duration * 3);
                        }

                        if timestamp >= self.next_write {
                            // TODO: Use publishing application name as output directory and check if exists.
                            let filename = format!("{}-{}-{}.ts", "test", timestamp, self.keyframe_counter);
                            let path = format!("./tmp/stream/{}", filename);
                            self.buffer.write_to_file(&path).unwrap();
                            self.playlist.add_media_segment(filename, keyframe_duration);
                            self.next_write += self.write_interval;
                        }

                        self.keyframe_counter += 1;
                        self.last_keyframe = timestamp;
                    }

                    if let Err(why) = self.buffer.push_video(&packet) {
                        warn!("Failed to put data into buffer: {:?}", why);
                    }
                },
                Media::AAC(timestamp, bytes) => {
                    let timestamp = u64::from(timestamp.value);

                    let packet = match aac::Packet::try_from_bytes(bytes, timestamp, &self.shared_state) {
                        Err(why) => {
                            error!("Failed to build packet: {:?}", why);
                            continue;
                        },
                        Ok(p) => p
                    };

                    if packet.is_sequence_header() {
                        continue;
                    }

                    if self.keyframe_counter == 0 {
                        continue;
                    }

                    if let Err(why) = self.buffer.push_audio(&packet) {
                        warn!("Failed to put data into buffer: {:?}", why);
                    }
                },
            }
        }

        Ok(Async::Ready(()))
    }
}

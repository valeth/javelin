use std::{path::PathBuf, fs};
use log::{debug, error, warn};
use futures::try_ready;
use tokio::prelude::*;
use super::{
    transport_stream::Buffer as TsBuffer,
    m3u8::Playlist,
};
use crate::{
    shared::Shared,
    media::{self, Media, avc, aac},
    error::{Error, Result},
};


pub struct Writer {
    receiver: media::Receiver,
    write_interval: u64,
    next_write: u64,
    last_keyframe: u64,
    keyframe_counter: usize,
    buffer: TsBuffer,
    shared_state: media::codec::SharedState,
    playlist: Playlist,
    _shared: Shared,
    stream_path: PathBuf,
}

impl Writer {
    pub fn create(app_name: String, receiver: media::Receiver, shared: Shared) -> Result<Self> {
        let write_interval = 2000; // milliseconds
        let next_write = write_interval; // milliseconds

        let hls_root = shared.config.read().hls.root_dir.clone();
        let stream_path = hls_root.join(app_name);
        let playlist_path = stream_path.join("playlist.m3u8");

        if stream_path.exists() && !stream_path.is_dir() {
            return Err(Error::Custom(format!("Path '{}' exists, but is not a directory", stream_path.display())));
        } else {
            debug!("Creating HLS directory at '{:?}'", stream_path);
            fs::create_dir_all(&stream_path)?;
        }


        Ok(Self {
            receiver,
            write_interval,
            next_write,
            last_keyframe: 0,
            keyframe_counter: 0,
            buffer: TsBuffer::new(),
            shared_state: media::codec::SharedState::new(),
            playlist: Playlist::new(playlist_path),
            _shared: shared,
            stream_path,
        })
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
                            let filename = format!("{}-{}-{}.ts", "test", timestamp, self.keyframe_counter);
                            let path = self.stream_path.join(&filename);
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

use {
    std::{
        path::PathBuf,
        fs,
        convert::TryFrom
    },
    log::{debug, error, warn},
    futures::try_ready,
    tokio::prelude::*,
    bytes::Bytes,
    chrono::Utc,
    javelin_codec::{
        FormatReader,
        FormatWriter,
        avc::{self, AvcCoder},
        aac::{self, AacCoder},
        flv,
    },
    super::{
        transport_stream::Buffer as TsBuffer,
        m3u8::Playlist,
    },
    crate::{
        shared::Shared,
        media::{self, Media},
        error::{Error, Result},
    },
};


pub struct Writer {
    receiver: media::Receiver,
    write_interval: u64,
    next_write: u64,
    last_keyframe: u64,
    keyframe_counter: usize,
    buffer: TsBuffer,
    playlist: Playlist,
    stream_path: PathBuf,
    avc_coder: AvcCoder,
    aac_coder: AacCoder,
}

impl Writer {
    pub fn create(app_name: String, receiver: media::Receiver, shared: &Shared) -> Result<Self> {
        let write_interval = 2000; // milliseconds
        let next_write = write_interval; // milliseconds

        let hls_root = shared.config.read().hls.root_dir.clone();
        let stream_path = hls_root.join(app_name);
        let playlist_path = stream_path.join("playlist.m3u8");

        if stream_path.exists() && !stream_path.is_dir() {
            return Err(Error::from(format!("Path '{}' exists, but is not a directory", stream_path.display())));
        }

        debug!("Creating HLS directory at '{}'", stream_path.display());
        fs::create_dir_all(&stream_path)?;

        Ok(Self {
            receiver,
            write_interval,
            next_write,
            last_keyframe: 0,
            keyframe_counter: 0,
            buffer: TsBuffer::new(),
            playlist: Playlist::new(playlist_path, shared),
            avc_coder: AvcCoder::new(),
            aac_coder: AacCoder::new(),
            stream_path,
        })
    }

    fn handle_h264<T>(&mut self, timestamp: T, bytes: Bytes) -> Result<()>
        where T: Into<u64>
    {
        let timestamp: u64 = timestamp.into();

        let flv_packet = flv::tag::VideoData::try_from(bytes.as_ref()).unwrap();
        let payload = &flv_packet.body;

        if flv_packet.is_sequence_header() {
            self.avc_coder.set_dcr(payload.as_ref());
            return Ok(());
        }

        let keyframe = flv_packet.is_keyframe();

        if keyframe {
            let keyframe_duration = timestamp - self.last_keyframe;

            if self.keyframe_counter == 1 {
                self.playlist.set_target_duration(keyframe_duration * 3);
            }

            if timestamp >= self.next_write {
                let filename = format!("{}-{}.mpegts", Utc::now().timestamp(), self.keyframe_counter);
                let path = self.stream_path.join(&filename);
                self.buffer.write_to_file(&path)?;
                self.playlist.add_media_segment(filename, keyframe_duration);
                self.next_write += self.write_interval;
            }

            self.keyframe_counter += 1;
            self.last_keyframe = timestamp;
        }

        let video = match self.avc_coder.read_format(avc::Avcc, &payload) {
            Ok(Some(avc)) => match self.avc_coder.write_format(avc::AnnexB, avc) {
                Ok(video) => video,
                Err(why) => {
                    error!("{}", why);
                    return Ok(())
                }
            },
            Err(why) => {
                error!("{}", why);
                return Ok(())
            }
            _ => {
                debug!("Got empty result");
                return Ok(())
            }
        };

        let comp_time = flv_packet.composition_time as u64;

        if let Err(why) = self.buffer.push_video(timestamp, comp_time, keyframe, video) {
            warn!("Failed to put data into buffer: {:?}", why);
        }

        Ok(())
    }

    fn handle_aac<T>(&mut self, timestamp: T, bytes: Bytes) -> Result<()>
        where T: Into<u64>
    {
        let timestamp: u64 = timestamp.into();

        let flv = flv::tag::AudioData::try_from(bytes.as_ref()).unwrap();

        if flv.is_sequence_header() {
            if let Err(why) = self.aac_coder.set_asc(flv.body.as_ref()) {
                log::error!("{}", why);
            };
            return Ok(());
        }

        if self.keyframe_counter == 0 {
            return Ok(());
        }

        let audio = match self.aac_coder.read_format(aac::Raw, &flv.body) {
            Ok(Some(raw_aac)) => match self.aac_coder.write_format(aac::AudioDataTransportStream, raw_aac) {
                Ok(audio) => audio,
                Err(why) => {
                    error!("{}", why);
                    return Ok(())
                }
            },
            Err(why) => {
                error!("{}", why);
                return Ok(())
            }
            _ => {
                debug!("Got empty result");
                return Ok(())
            }
        };

        if let Err(why) = self.buffer.push_audio(timestamp, audio) {
            warn!("Failed to put data into buffer: {:?}", why);
        }

        Ok(())
    }

    fn handle(&mut self, media: Media) -> Result<()> {
        match media {
            Media::H264(timestamp, bytes) => self.handle_h264(timestamp.value, bytes),
            Media::AAC(timestamp, bytes) => self.handle_aac(timestamp.value, bytes),
        }
    }
}


impl Future for Writer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(media) = try_ready!(self.receiver.poll()) {
            self.handle(media).map_err(|why| error!("{:?}", why))?;
        }

        Ok(Async::Ready(()))
    }
}

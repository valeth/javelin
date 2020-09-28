use {
    std::{
        convert::TryFrom,
        path::{Path, PathBuf},
        fs,
    },
    chrono::Utc,
    anyhow::{Result, bail},
    javelin_codec::{
        FormatReader,
        FormatWriter,
        avc::{self, AvcCoder},
        aac::{self, AacCoder},
        flv,
        mpegts::TransportStream,
    },
    javelin_types::{Packet, PacketType},
    javelin_core::session,
    crate::{
        config::Config,
        file_cleaner,
        m3u8::Playlist,
    },
};


pub struct Writer {
    watcher: session::Watcher,
    write_interval: u64,
    next_write: u64,
    last_keyframe: u64,
    keyframe_counter: usize,
    buffer: TransportStream,
    playlist: Playlist,
    stream_path: PathBuf,
    avc_coder: AvcCoder,
    aac_coder: AacCoder,
}

impl Writer {
    pub fn create(app_name: String, watcher: session::Watcher, fcleaner_sender: file_cleaner::Sender, config: &Config) -> Result<Self> {
        let write_interval = 2000; // milliseconds
        let next_write = write_interval; // milliseconds

        let hls_root = config.root_dir.clone();
        let stream_path = hls_root.join(app_name);
        let playlist_path = stream_path.join("playlist.m3u8");

        prepare_stream_directory(&stream_path)?;

        Ok(Self {
            watcher,
            write_interval,
            next_write,
            last_keyframe: 0,
            keyframe_counter: 0,
            buffer: TransportStream::new(),
            playlist: Playlist::new(playlist_path, fcleaner_sender),
            avc_coder: AvcCoder::new(),
            aac_coder: AacCoder::new(),
            stream_path,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        while let Ok(packet) = self.watcher.recv().await {
            if let Err(why) = self.handle_packet(packet) {
                log::error!("{:?}", why);
            }
        }

        Ok(())
    }

    fn handle_video<T>(&mut self, timestamp: T, bytes: &[u8]) -> Result<()>
        where T: Into<u64>
    {
        let timestamp: u64 = timestamp.into();

        let flv_packet = flv::tag::VideoData::try_from(bytes)?;
        let payload = &flv_packet.body;

        if flv_packet.is_sequence_header() {
            self.avc_coder.set_dcr(payload.as_ref())?;
            return Ok(())
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
                self.last_keyframe = timestamp;
            }

            self.keyframe_counter += 1;
        }

        let video = match self.avc_coder.read_format(avc::Avcc, &payload)? {
            Some(avc) => self.avc_coder.write_format(avc::AnnexB, avc)?,
            None => return Ok(())
        };

        let comp_time = flv_packet.composition_time as u64;

        if let Err(why) = self.buffer.push_video(timestamp, comp_time, keyframe, video) {
            log::warn!("Failed to put data into buffer: {:?}", why);
        }

        Ok(())
    }

    fn handle_audio<T>(&mut self, timestamp: T, bytes: &[u8]) -> Result<()>
        where T: Into<u64>
    {
        let timestamp: u64 = timestamp.into();

        let flv = flv::tag::AudioData::try_from(bytes).unwrap();

        if flv.is_sequence_header() {
            self.aac_coder.set_asc(flv.body.as_ref())?;
            return Ok(())
        }

        if self.keyframe_counter == 0 {
            return Ok(());
        }

        let audio = match self.aac_coder.read_format(aac::Raw, &flv.body)? {
            Some(raw_aac) => self.aac_coder.write_format(aac::AudioDataTransportStream, raw_aac)?,
            None => return Ok(())
        };

        if let Err(why) = self.buffer.push_audio(timestamp, audio) {
            log::warn!("Failed to put data into buffer: {:?}", why);
        }

        Ok(())
    }

    fn handle_packet(&mut self, packet: Packet) -> Result<()> {
        match packet.kind {
            PacketType::Video => {
                self.handle_video(packet.timestamp.unwrap(), packet.as_ref())
            }
            PacketType::Audio => {
                self.handle_audio(packet.timestamp.unwrap(), packet.as_ref())
            }
            _ => Ok(())
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        log::info!("Closing HLS writer for {}", self.stream_path.display());
    }
}


fn prepare_stream_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let stream_path = path.as_ref();

    if stream_path.exists() && !stream_path.is_dir() {
        bail!("Path '{}' exists, but is not a directory", stream_path.display());
    }

    log::debug!("Creating HLS directory at '{}'", stream_path.display());
    fs::create_dir_all(&stream_path)?;

    Ok(())
}

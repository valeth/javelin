use std::{
    fs,
    path::PathBuf,
    time::Duration,
};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use log::error;
use m3u8_rs::playlist::{MediaPlaylist, MediaSegment};
use tempfile::NamedTempFile;
use super::file_cleaner;
use crate::{
    error::Result,
    shared::Shared
};


pub struct Playlist {
    file_path: PathBuf,
    current_duration: u64,
    playlist: MediaPlaylist,
    file_cleaner: file_cleaner::Sender,
}

impl Playlist {
    const DEFAULT_TARGET_DURATION: f32 = 6.0;
    const MAX_PLAYLIST_DURATION: u64 = 30000; // milliseconds

    pub fn new<P>(path: P, shared: Shared) -> Self
        where P: Into<PathBuf>
    {
        let mut playlist = MediaPlaylist::default();
        playlist.version = 3;
        playlist.target_duration = Self::DEFAULT_TARGET_DURATION;
        playlist.media_sequence = 0;

        let file_cleaner = shared.fcleaner_sender().expect("Missing file cleaner sender");

        Self {
            file_path: path.into(),
            current_duration: 0,
            playlist,
            file_cleaner,
        }
    }

    pub fn set_target_duration(&mut self, duration: u64) {
        self.playlist.target_duration = (duration as f64 / 1000.0) as f32;
    }

    fn schedule_for_deletion(&mut self) {
        let segments: Vec<MediaSegment> = self.playlist.segments.drain(..).collect();
        let items: Vec<_> = segments.iter()
            .map(|seg| self.hls_root().join(&seg.uri))
            .collect();

        self.playlist.media_sequence += items.len() as i32;
        self.file_cleaner.unbounded_send((Duration::from_millis(self.current_duration), items)).unwrap();
        self.current_duration = 0;
    }

    pub fn add_media_segment<S>(&mut self, uri: S, duration: u64)
        where S: Into<String>
    {
        let mut segment = MediaSegment::empty();
        segment.duration = (duration as f64 / 1000.0) as f32;
        segment.title = Some("".into());
        segment.uri = uri.into();

        if self.current_duration >= Self::MAX_PLAYLIST_DURATION {
            self.schedule_for_deletion();
        }

        self.current_duration += duration;
        self.playlist.segments.push(segment);

        if let Err(why) = self.atomic_update() {
            error!("Failed to update playlist: {:?}", why);
        }
    }

    fn atomic_update(&mut self) -> Result<()> {
        let mut tmp_file = tempfile::Builder::new()
            .prefix(".playlist.m3u")
            .suffix(".tmp")
            .tempfile_in(&self.hls_root())?;

        self.write_temporary_file(&mut tmp_file)?;
        fs::rename(&tmp_file.path(), &self.file_path)?;

        Ok(())
    }

    fn hls_root(&self) -> PathBuf {
        self.file_path.parent().expect("No parent directory for playlist").into()
    }

    fn write_temporary_file(&mut self, tmp_file: &mut NamedTempFile) -> Result<()> {
        self.playlist.write_to(tmp_file)?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&tmp_file.path())?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&tmp_file.path(), perms)?;
        }

        Ok(())
    }
}

impl Drop for Playlist {
    fn drop(&mut self) {
        self.schedule_for_deletion();
        self.playlist.end_list = true;

        if let Err(why) = self.atomic_update() {
            error!("Failed to write end tag to playlist: {:?}", why);
        }
    }
}

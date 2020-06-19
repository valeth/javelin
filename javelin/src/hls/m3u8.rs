use {
    std::{fs, path::PathBuf, time::Duration},
    m3u8_rs::playlist::{MediaPlaylist, MediaSegment},
    tempfile::NamedTempFile,
    anyhow::Result,
    super::file_cleaner,
    crate::shared::Shared,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;


pub struct Playlist {
    file_path: PathBuf,
    current_duration: u64,
    cleanup_started: bool,
    playlist: MediaPlaylist,
    file_cleaner: file_cleaner::Sender,
}

impl Playlist {
    const DEFAULT_TARGET_DURATION: f32 = 6.0;
    const PLAYLIST_CACHE_DURATION: u64 = 30000; // milliseconds

    pub fn new<P>(path: P, shared: &Shared) -> Self
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
            cleanup_started: false,
            playlist,
            file_cleaner,
        }
    }

    pub fn set_target_duration(&mut self, duration: u64) {
        self.playlist.target_duration = (duration as f64 / 1000.0) as f32;
    }

    fn schedule_for_deletion(&mut self, amount: usize, delete_after: u64) {
        let segments_to_delete: Vec<_> = self.playlist.segments.drain(..amount).collect();
        let paths: Vec<_> = segments_to_delete.iter()
            .map(|seg| {
                self.current_duration -= (seg.duration * 1000.0) as u64;
                self.file_path.parent().unwrap().join(&seg.uri)
            })
            .collect();

        self.playlist.media_sequence += paths.len() as i32;
        self.file_cleaner.unbounded_send((Duration::from_millis(delete_after), paths)).unwrap();
    }

    pub fn add_media_segment<S>(&mut self, uri: S, duration: u64)
        where S: Into<String>
    {
        let mut segment = MediaSegment::empty();
        segment.duration = (duration as f64 / 1000.0) as f32;
        segment.title = Some("".into()); // adding empty title here, because implementation is broken
        segment.uri = uri.into();


        if self.cleanup_started {
            self.schedule_for_deletion(1, Self::PLAYLIST_CACHE_DURATION);
        } else if self.current_duration >= Self::PLAYLIST_CACHE_DURATION {
            self.cleanup_started = true;
        }

        self.current_duration += duration;
        self.playlist.segments.push(segment);

        if let Err(why) = self.atomic_update() {
            log::error!("Failed to update playlist: {:?}", why);
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
        self.schedule_for_deletion(self.playlist.segments.len(), self.current_duration);
        self.playlist.end_list = true;

        if let Err(why) = self.atomic_update() {
            log::error!("Failed to write end tag to playlist: {:?}", why);
        }
    }
}

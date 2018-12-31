use std::{
    io,
    os::unix::fs::PermissionsExt,
    fs,
    path::PathBuf,
};
use log::error;
use m3u8_rs::playlist::{MediaPlaylist, MediaSegment};


pub struct Playlist {
    file_path: PathBuf,
    playlist: MediaPlaylist,
}

impl Playlist {
    const DEFAULT_TARGET_DURATION: f32 = 6.0;

    pub fn new<P>(path: P) -> Self
        where P: Into<PathBuf>
    {
        let mut playlist = MediaPlaylist::default();
        playlist.version = 3;
        playlist.target_duration = Self::DEFAULT_TARGET_DURATION;

        Self {
            file_path: path.into(),
            playlist
        }
    }

    pub fn set_target_duration(&mut self, duration: u64) {
        self.playlist.target_duration = (duration as f64 / 1000.0) as f32;
    }

    pub fn add_media_segment<S>(&mut self, uri: S, duration: u64)
        where S: Into<String>
    {
        let mut segment = MediaSegment::empty();
        segment.duration = (duration as f64 / 1000.0) as f32;
        segment.title = Some("".into());
        segment.uri = uri.into();
        self.playlist.segments.push(segment);
        self.atomic_update().unwrap();
    }

    fn atomic_update(&mut self) -> io::Result<()> {
        let mut tmp_file = tempfile::Builder::new()
            .prefix(".playlist.m3u")
            .suffix(".tmp")
            .tempfile_in(&self.file_path.parent().unwrap())?;
        self.playlist.write_to(&mut tmp_file)?;
        let mut perms = fs::metadata(&tmp_file.path())?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&tmp_file.path(), perms)?;
        fs::rename(&tmp_file.path(), &self.file_path)?;
        Ok(())
    }
}

impl Drop for Playlist {
    fn drop(&mut self) {
        self.playlist.end_list = true;
        if let Err(why) = self.atomic_update() {
            error!("Failed to write end tag to playlist: {:?}", why);
        }
    }
}

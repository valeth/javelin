use std::collections::HashMap;

use javelin_types::Metadata;
use rml_rtmp::sessions::StreamMetadata;

// Temporary conversion functions

pub(crate) fn from_metadata(val: StreamMetadata) -> Metadata {
    let mut map = HashMap::with_capacity(11);

    if let Some(v) = val.audio_bitrate_kbps {
        map.insert("audio.bitrate", v.to_string());
    }

    if let Some(v) = val.audio_channels {
        map.insert("audio.channels", v.to_string());
    }

    if let Some(v) = val.audio_codec_id {
        map.insert("audio.codec_id", v.to_string());
    }

    if let Some(v) = val.audio_is_stereo {
        map.insert("audio.stereo", v.to_string());
    }

    if let Some(v) = val.audio_sample_rate {
        map.insert("audio.sampling_rate", v.to_string());
    }

    if let Some(v) = val.video_bitrate_kbps {
        map.insert("video.bitrate", v.to_string());
    }

    if let Some(v) = val.video_codec_id {
        map.insert("video.codec_id", v.to_string());
    }

    if let Some(v) = val.video_frame_rate {
        map.insert("video.frame_rate", v.to_string());
    }

    if let Some(v) = val.video_height {
        map.insert("video.height", v.to_string());
    }

    if let Some(v) = val.video_width {
        map.insert("video.width", v.to_string());
    }

    if let Some(v) = val.encoder {
        map.insert("encoder", v);
    }

    Metadata::from(map)
}

pub(crate) fn into_metadata(val: Metadata) -> StreamMetadata {
    StreamMetadata {
        video_width: val.get("video.width"),
        video_height: val.get("video.height"),
        video_codec_id: val.get("video.codec_id"),
        video_frame_rate: val.get("video.frame_rate"),
        video_bitrate_kbps: val.get("video.bitrate"),
        audio_codec_id: val.get("audio.codec_id"),
        audio_bitrate_kbps: val.get("audio.bitrate"),
        audio_sample_rate: val.get("audio.sampling_rate"),
        audio_channels: val.get("audio.channels"),
        audio_is_stereo: val.get("audio.stereo"),
        encoder: val.get("encoder"),
    }
}

use thiserror::Error;


#[derive(Error, Debug)]
pub enum FlvError {
    #[error("Unsupported sampling rate value {0}")]
    UnsupportedSamplingRate(u8),

    #[error("Unsupported sample size value {0}")]
    UnsupportedSampleSize(u8),

    #[error("Received frame with unknown type {0}")]
    UnknownFrameType(u8),

    #[error("Received package with unknown type {0}")]
    UnknownPackageType(u8),

    #[error("Video format with id {0} is not supported")]
    UnsupportedVideoFormat(u8),

    #[error("Audio format with id {0} is not supported")]
    UnsupportedAudioFormat(u8),

    #[error("Not enough data: {0}")]
    NotEnoughData(&'static str),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

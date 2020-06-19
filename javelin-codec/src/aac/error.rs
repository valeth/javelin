use thiserror::Error;


#[derive(Debug, Error)]
pub enum AacError {
    #[error("AAC coder not initialized")]
    NotInitialized,

    #[error("Not enough data: {0}")]
    NotEnoughData(&'static str),

    #[error("Unsupported audio object type")]
    UnsupportedAudioFormat,

    #[error("Reserved or unsupported frequency index {0}")]
    UnsupportedFrequencyIndex(u8),

    #[error("Reserved or unsupported channel configuration {0}")]
    UnsupportedChannelConfiguration(u8),

    #[error("Got forbidden sampling frequency index {0}")]
    ForbiddenSamplingFrequencyIndex(u8),
}

use thiserror::Error;


#[derive(Debug, Error)]
pub enum AvcError {
    #[error("Failed to initialize the AVC decoder")]
    DecoderInitializationFailed,

    #[error("AVC coder not initialized")]
    NotInitialized,

    #[error("Did not get enough data")]
    NotEnoughData,

    #[error("Unsupported configuration record version {0}")]
    UnsupportedConfigurationRecordVersion(u8),

    #[error("Unsupported or unknown NAL unit type {0}")]
    UnsupportedNalUnitType(u8),
}

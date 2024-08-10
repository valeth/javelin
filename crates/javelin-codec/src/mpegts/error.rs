use {
    std::io,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum TsError {
    #[error("Failed to create TS file")]
    FileCreationFailed(#[from] io::Error),

    #[error("Failed to write TS file")]
    WriteError,

    #[error("Packet ID {0} is not valid")]
    InvalidPacketId(u16),

    #[error("Invalid timestamp {0}")]
    InvalidTimestamp(u64),

    #[error("Packet payload exceeded packet limit")]
    PayloadTooBig,

    #[error("Clock reference value of {0} exceeds maximum")]
    ClockValueOutOfRange(u64),
}

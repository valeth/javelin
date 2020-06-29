use {
    std::io,
    thiserror::Error,
    crate::proto::Error as ProtocolError,
};


#[derive(Error, Debug)]
pub enum Error {
    #[error("Stream key \"{0}\" is not permitted")]
    StreamKeyNotPermitted(String),

    #[error("No stream with name {0} found")]
    NoSuchStream(String),

    #[error("Client disconnected: {0}")]
    Disconnected(#[from] io::Error),

    #[error("Failed to create new session")]
    SessionCreationFailed,

    #[error("Failed to release session")]
    SessionReleaseFailed,

    #[error("Failed to join session")]
    SessionJoinFailed,

    #[error("Failed to send to session")]
    SessionSendFailed,

    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
}

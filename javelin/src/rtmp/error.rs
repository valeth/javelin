use {
    std::io,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Got invalid byte input")]
    InvalidInput,

    #[error("Failed to create an RTMP session")]
    SessionCreationFailed,

    #[error("RTMP handshake failed")]
    HandshakeFailed,

    #[error("Request {0} was not accepted")]
    RequestNotAccepted(u32),

    #[error("Application name can not be empty")]
    ApplicationNameRequired,

    #[error("Stream key \"{0}\" is not permitted")]
    StreamKeyNotPermitted(String),

    #[error("Publish request failed")]
    PublishRequestFailed,

    #[error("Failed to prepare {0}")]
    DataPreparationFailed(&'static str),

    #[error("Client disconnected: {0}")]
    Disconnected(#[from] io::Error),

    #[error("Stream ID missing")]
    NoStreamId,
}

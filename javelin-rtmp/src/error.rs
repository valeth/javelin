use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error("RTMP handshake failed")]
    HandshakeFailed,

    #[error("RTMP session initialization failed")]
    SessionInitializationFailed,

    #[error("Tried to use RTMP session while not initialized")]
    SessionNotInitialized,

    #[error("Received invalid input")]
    InvalidInput,

    #[error("RTMP request was not accepted")]
    RequestRejected,

    #[error("No stream ID")]
    NoStreamId,

    #[error("Application name cannot be empty")]
    EmptyAppName,
}


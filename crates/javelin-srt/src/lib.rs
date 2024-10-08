mod config;
mod peer;
mod service;


use std::io;

pub use self::service::Service;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Client did not provide a stream id parameter")]
    StreamIdMissing,

    #[error("Client did not provide a well formed ACL through parameters")]
    InvalidAccessControlParams,

    #[error("Client did not provide required ACL parameters")]
    MissingAccessControlParams,

    #[error("Requested mode is not supported")]
    ModeNotSupported,

    #[error("Client is not authorized to access the resource")]
    Unauthorized,

    #[error("Failed to parse the given stream id")]
    StreamIdDecodeFailed { stream_id: String },

    #[error(transparent)]
    Io(#[from] io::Error),
}

use {
    std::io,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Stream key \"{0}\" is not permitted")]
    StreamKeyNotPermitted(String),

    #[error("No stream with name {0} found")]
    NoSuchStream(String),

    #[error("Client disconnected: {0}")]
    Disconnected(#[from] io::Error),
}

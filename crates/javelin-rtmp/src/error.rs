use std::io;

use thiserror::Error;
use tokio::time;

use crate::proto::Error as ProtocolError;


#[derive(Error, Debug)]
pub enum Error {
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

    #[error("Failed to return packet to peer {0}")]
    ReturnPacketFailed(u64),

    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),

    #[error("Connection timeout")]
    ConnectionTimeout(#[from] time::error::Elapsed),
}

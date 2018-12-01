use rtmp::sessions::{
    ServerSession,
    ServerSessionConfig,
    ServerSessionResult
};
use error::Result;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    Waiting,
    Publishing(String, String),
    Watching(String, u32),
}


/// Represents a session of a connected client
pub struct Client {
    peer_id: u64,
    state: ClientState,
    pub session: ServerSession,
    pub received_video_keyframe: bool,
}

impl Client {
    pub fn new(peer_id: u64) -> Result<(Self, Vec<ServerSessionResult>)> {
        let session_config = ServerSessionConfig::new();
        let (session, results) = ServerSession::new(session_config)?;

        let this = Self {
            peer_id,
            session,
            state: ClientState::Waiting,
            received_video_keyframe: false,
        };

        Ok((this, results))
    }
}

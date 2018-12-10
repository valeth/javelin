use rml_rtmp::sessions::{
    ServerSession,
    ServerSessionConfig,
    ServerSessionResult
};
use crate::{
    error::{Error, Result},
    peer::media::Channel,
};


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
    #[allow(clippy::new_ret_no_self)]
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

    pub fn accept_request(&mut self, request_id: u32) -> Result<Vec<ServerSessionResult>> {
        self.session.accept_request(request_id).map_err(|_| Error::RequestError)
    }

    pub fn publish(&mut self, channel: &mut Channel, app_name: String, stream_key: String) {
        channel.set_publisher(self.peer_id, stream_key.clone());
        self.state = ClientState::Publishing(app_name, stream_key);
    }

    pub fn publishing_app_name(&self) -> Option<String> {
        match self.state {
            ClientState::Publishing(ref app_name, _) => Some(app_name.clone()),
            _ => None,
        }
    }

    pub fn watch(&mut self, channel: &mut Channel, stream_id: u32, app_name: String) {
        channel.add_watcher(self.peer_id);
        self.state = ClientState::Watching(app_name, stream_id);
    }

    pub fn watched_stream(&self) -> Option<u32> {
        match self.state {
            ClientState::Watching(_, stream_id) => Some(stream_id),
            _ => None,
        }
    }

    pub fn watched_app_name(&self) -> Option<String> {
        match self.state {
            ClientState::Watching(ref app_name, _) => Some(app_name.clone()),
            _ => None,
        }
    }
}

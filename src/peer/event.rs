use std::{
    collections::VecDeque,
};
use shared::Shared;
use rtmp::{
    sessions::{
        ServerSessionResult,
        ServerSessionEvent as Event,
        StreamMetadata,
    },
    chunk_io::Packet,
    time::RtmpTimestamp
};
use error::Result;
use peer::Client;
use super::media::Media;


#[derive(Debug)]
pub enum EventResult {
    Outbound(u64, Packet),
}


pub struct Handler {
    peer_id: u64,
    results: VecDeque<EventResult>,
    shared: Shared,
}

impl Handler {
    pub fn new(peer_id: u64, shared: Shared) -> Result<Self> {
        let results = {
            let mut clients = shared.clients.lock();
            let (client, results) = Client::new(peer_id)?;
            clients.insert(peer_id, client);
            results
        };

        let mut this = Self {
            peer_id,
            results: VecDeque::new(),
            shared
        };

        this.handle_server_session_results(results)?;

        Ok(this)
    }

    pub fn handle(&mut self, bytes: &[u8]) -> Result<Vec<EventResult>> {
        let results = {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();
            client.session.handle_input(bytes)?
        };

        self.handle_server_session_results(results)?;

        Ok(self.results.drain(..).collect())
    }

    fn handle_server_session_results(&mut self, results: Vec<ServerSessionResult>) -> Result<()> {
        use self::ServerSessionResult::*;

        for result in results {
            match result {
                OutboundResponse(packet) => {
                    self.results.push_back(EventResult::Outbound(self.peer_id, packet));
                },
                RaisedEvent(event) => {
                    self.handle_event(event)?;
                },
                UnhandleableMessageReceived(_) => {
                    debug!("Unhandleable message received");
                },
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        use self::Event::*;

        match event {
            ConnectionRequested { request_id, app_name } => {
                self.connection_requested(request_id, app_name)?;
            },
            PublishStreamRequested { request_id, app_name, stream_key, .. } => {
                self.publish_requested(request_id, app_name, stream_key)?;
            }
            PlayStreamRequested { request_id, app_name, stream_id, .. } => {
                self.play_requested(request_id, app_name, stream_id)?;
            },
            StreamMetadataChanged { app_name, stream_key, metadata } => {
                self.metadata_received(app_name, stream_key, metadata)?;
            },
            VideoDataReceived { stream_key, data, timestamp, .. } => {
                self.multimedia_data_received(stream_key, timestamp, Media::H264(data))?;
            },
            AudioDataReceived { stream_key, data, timestamp, .. } => {
                self.multimedia_data_received(stream_key, timestamp, Media::AAC(data))?;
            },
            PublishStreamFinished { app_name, stream_key } => {
                self.publish_stream_finished(app_name, stream_key)?;
            }
            _ => {
                debug!("Event: {:?}", event);
            }
        }

        Ok(())
    }


    fn connection_requested(&mut self, request_id: u32, app_name: String) -> Result<()> {
        info!("Connection request from client {} for app '{}'", self.peer_id, app_name);

        let results = {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();
            client.accept_request(request_id)?
        };

        self.handle_server_session_results(results)?;

        Ok(())
    }

    fn publish_requested(&mut self, request_id: u32, app_name: String, stream_key: String) -> Result<()> {
        info!("Client {} requested publishing to app '{}' using stream key {}", self.peer_id, app_name, stream_key);
        Ok(())
    }

    fn publish_stream_finished(&mut self, app_name: String, stream_key: String) -> Result<()> {
        info!("Publishing of app '{}' finished", app_name);
        Ok(())
    }

    fn play_requested(&mut self, request_id: u32, app_name: String, stream_id: u32) -> Result<()> {
        info!("Client {} requested playback of app '{}'", self.peer_id, app_name);
        Ok(())
    }

    fn metadata_received(&mut self, app_name: String, stream_key: String, metadata: StreamMetadata) -> Result<()> {
        debug!("Received stream metadata for app '{}'", app_name);
        Ok(())
    }

    fn multimedia_data_received(&mut self, stream_key: String, timestamp: RtmpTimestamp, media: Media) -> Result<()> {
        // debug!("Received video data for stream with key {}", stream_key);
        Ok(())
    }
}

use std::{
    collections::VecDeque,
    rc::Rc,
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
use error::{Error, Result};
use peer::Client;
use super::media::{Media, Channel};


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
            StreamMetadataChanged { app_name, metadata, .. } => {
                self.metadata_received(app_name, metadata)?;
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

        {
            let streams = self.shared.streams.read();
            match streams.get(&app_name) {
                Some(stream) => {
                    match stream.has_publisher() {
                        false => (),
                        true => return Err(Error::from(format!("{} is already being published to", app_name))),
                    }
                },
                None => (),
            }
        }

        let result = {
            let mut clients = self.shared.clients.lock();
            let mut client = clients.get_mut(&self.peer_id).unwrap();
            let mut streams = self.shared.streams.write();
            let mut stream = streams.entry(app_name.clone()).or_insert(Channel::new());
            client.publish(&mut stream, app_name.clone(), stream_key.clone());
            client.accept_request(request_id)
        };

        {
            let mut app_names = self.shared.app_names.write();
            app_names.insert(stream_key, app_name);
        }

        match result {
            Err(why) => {
                error!("Error while accepting publishing request: {:?}", why);
                return Err(Error::SessionError("Publish request failed".to_string()));
            },
            Ok(results) => self.handle_server_session_results(results)?
        }

        Ok(())
    }

    fn publish_stream_finished(&mut self, app_name: String, stream_key: String) -> Result<()> {
        info!("Publishing of app '{}' finished", app_name);

        {
            let mut streams = self.shared.streams.write();
            let stream = streams.get_mut(&app_name).unwrap();
            stream.unpublish();
        }

        {
            let mut app_names = self.shared.app_names.write();
            app_names.remove(&stream_key);
        }

        Ok(())
    }

    fn play_requested(&mut self, request_id: u32, app_name: String, stream_id: u32) -> Result<()> {
        info!("Client {} requested playback of app '{}'", self.peer_id, app_name);

        let results = {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();

            {
                let mut streams = self.shared.streams.write();
                let mut stream = streams.entry(app_name.clone()).or_insert(Channel::new());
                client.watch(&mut stream, stream_id, app_name.clone());
            }

            client.accept_request(request_id)?
        };

        {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();
            let streams = self.shared.streams.read();

            if let Some(ref metadata) = streams.get(&app_name).unwrap().metadata {
                let packet = client.session.send_metadata(stream_id, Rc::new(metadata.clone()))
                    .map_err(|_| Error::SessionError("Failed to send metadata".to_string()))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }

            if let Some(ref v_seq_h) = streams.get(&app_name).unwrap().video_seq_header {
                let packet = client.session.send_video_data(stream_id, v_seq_h.clone(), RtmpTimestamp::new(0), false)
                    .map_err(|_| Error::SessionError("Failed to send video data".to_string()))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }

            if let Some(ref a_seq_h) = streams.get(&app_name).unwrap().audio_seq_header {
                let packet = client.session.send_audio_data(stream_id, a_seq_h.clone(), RtmpTimestamp::new(0), false)
                    .map_err(|_| Error::SessionError("Failed to send audio data".to_string()))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }
        }

        self.handle_server_session_results(results)?;

        Ok(())
    }

    fn metadata_received(&mut self, app_name: String, metadata: StreamMetadata) -> Result<()> {
        debug!("Received stream metadata for app '{}'", app_name);

        let mut streams = self.shared.streams.write();
        if let Some(stream) = streams.get_mut(&app_name) {
            stream.set_metadata(metadata.clone());
            let mut clients = self.shared.clients.lock();

            for client_id in &stream.watchers {
                let client = clients.get_mut(client_id).unwrap();

                if let Some(watched_stream) = client.watched_stream() {
                    let packet = client.session
                        .send_metadata(watched_stream, Rc::new(metadata.clone()))
                        .map_err(|_| Error::SessionError("Failed to send metadata".to_string()))?;

                    self.results.push_back(EventResult::Outbound(self.peer_id, packet));
                }
            }
        }

        Ok(())
    }

    fn multimedia_data_received(&mut self, stream_key: String, timestamp: RtmpTimestamp, media: Media) -> Result<()> {
        // debug!("Received video data for stream with key {}", stream_key);

        let app_name = self.shared
            .app_name_from_stream_key(stream_key)
            .ok_or(Error::SessionError("No app for stream key".to_string()))?;

        let mut streams = self.shared.streams.write();
        if let Some(stream) = streams.get_mut(&app_name) {
            match &media {
                Media::AAC(ref data) if media.is_sequence_header() => {
                    stream.audio_seq_header = Some(data.clone());
                },
                Media::H264(ref data) if media.is_sequence_header() => {
                    stream.video_seq_header = Some(data.clone());
                },
                _ => (),
            }

            for client_id in &stream.watchers {
                let mut clients = self.shared.clients.lock();
                let mut client = match clients.get_mut(&client_id) {
                    Some(client) => client,
                    None => continue,
                };

                if !(client.received_video_keyframe || media.is_sendable()) {
                    continue;
                }

                if let Some(active_stream) = client.watched_stream() {
                    let packet = match &media {
                        Media::AAC(bytes) => {
                            client.session.send_audio_data(active_stream, bytes.clone(), timestamp.clone(), true)?
                        }
                        Media::H264(ref bytes) => {
                            if media.is_keyframe() {
                                client.received_video_keyframe = true;
                            }
                            client.session.send_video_data(active_stream, bytes.clone(), timestamp.clone(), true)?
                        },
                    };

                    self.results.push_back(EventResult::Outbound(*client_id, packet));
                }
            }
        }

        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        let mut clients = self.shared.clients.lock();
        clients.remove(&self.peer_id);
    }
}

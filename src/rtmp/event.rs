use std::{
    collections::VecDeque,
    rc::Rc,
};
use::log::{debug, error, info};
#[cfg(feature = "hls")]
use futures::{sync::oneshot, Future};
use rml_rtmp::{
    sessions::{
        ServerSessionResult,
        ServerSessionEvent as Event,
        StreamMetadata,
    },
    chunk_io::Packet,
    time::RtmpTimestamp
};
use crate::{
    error::{Error, Result},
    shared::Shared,
    media::{Media, Channel},
};
#[cfg(feature = "hls")]
use crate::media;
use super::Client;


#[derive(Debug)]
pub enum EventResult {
    Outbound(u64, Packet),
    Disconnect,
}


pub struct Handler {
    peer_id: u64,
    results: VecDeque<EventResult>,
    shared: Shared,
    #[cfg(feature = "hls")]
    media_sender: Option<media::Sender>,
}

impl Handler {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(peer_id: u64, shared: Shared) -> Result<Self> {
        let results = {
            let mut clients = shared.clients.lock();
            let (client, results) = Client::new(peer_id, shared.clone())?;
            clients.insert(peer_id, client);
            results
        };

        let mut this = Self {
            peer_id,
            results: VecDeque::new(),
            shared,
            #[cfg(feature = "hls")]
            media_sender: None,
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
                self.connection_requested(request_id, &app_name)?;
            },
            PublishStreamRequested { request_id, app_name, stream_key, .. } => {
                self.publish_requested(request_id, app_name, stream_key)?;
            }
            PlayStreamRequested { request_id, app_name, stream_id, .. } => {
                self.play_requested(request_id, &app_name, stream_id)?;
            },
            StreamMetadataChanged { app_name, metadata, .. } => {
                self.metadata_received(&app_name, &metadata)?;
            },
            VideoDataReceived { stream_key, data, timestamp, .. } => {
                self.multimedia_data_received(&stream_key, &Media::H264(timestamp, data))?;
            },
            AudioDataReceived { stream_key, data, timestamp, .. } => {
                self.multimedia_data_received(&stream_key, &Media::AAC(timestamp, data))?;
            },
            PublishStreamFinished { app_name, stream_key } => {
                self.publish_stream_finished(&app_name, &stream_key)?;
            },
            _ => {
                debug!("Event: {:?}", event);
            }
        }

        Ok(())
    }

    fn connection_requested(&mut self, request_id: u32, app_name: &str) -> Result<()> {
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
            let config = self.shared.config.read();
            if stream_key.is_empty() || !config.permitted_stream_keys.contains(&stream_key) {
                return Err(Error::SessionError(format!("Stream key '{}' is not permitted", stream_key)));
            }
        }

        debug!("Stream key '{}' permitted", stream_key);

        {
            let streams = self.shared.streams.read();
            if let Some(stream) = streams.get(&app_name) {
                if stream.has_publisher() {
                    return Err(Error::SessionError(format!("App '{}' is already being published to", app_name)));
                }
            }
        }

        #[cfg(feature = "hls")]
        self.register_on_hls_server(app_name.clone());

        let result = {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();
            let mut streams = self.shared.streams.write();
            let mut stream = streams.entry(app_name.clone()).or_insert_with(Channel::new);
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
                return Err(Error::SessionError("Publish request failed".into()));
            },
            Ok(results) => self.handle_server_session_results(results)?
        }

        Ok(())
    }

    fn publish_stream_finished(&mut self, app_name: &str, stream_key: &str) -> Result<()> {
        info!("Publishing of app '{}' finished", app_name);

        {
            let mut streams = self.shared.streams.write();
            let stream = streams.get_mut(app_name).unwrap();
            stream.unpublish();
        }

        {
            let mut app_names = self.shared.app_names.write();
            app_names.remove(stream_key);
        }

        self.results.push_back(EventResult::Disconnect);

        Ok(())
    }

    fn play_requested(&mut self, request_id: u32, app_name: &str, stream_id: u32) -> Result<()> {
        info!("Client {} requested playback of app '{}'", self.peer_id, app_name);

        let results = {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();

            {
                let mut streams = self.shared.streams.write();
                let mut stream = streams.entry(app_name.to_string()).or_insert_with(Channel::new);
                client.watch(&mut stream, stream_id, app_name.to_string());
            }

            client.accept_request(request_id)?
        };

        {
            let mut clients = self.shared.clients.lock();
            let client = clients.get_mut(&self.peer_id).unwrap();
            let streams = self.shared.streams.read();

            if let Some(ref metadata) = streams.get(app_name).unwrap().metadata {
                let packet = client.session.send_metadata(stream_id, Rc::new(metadata.clone()))
                    .map_err(|_| Error::SessionError("Failed to send metadata".into()))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }

            if let Some(ref v_seq_h) = streams.get(app_name).unwrap().video_seq_header {
                let packet = client.session.send_video_data(stream_id, v_seq_h.clone(), RtmpTimestamp::new(0), false)
                    .map_err(|_| Error::SessionError("Failed to send video data".into()))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }

            if let Some(ref a_seq_h) = streams.get(app_name).unwrap().audio_seq_header {
                let packet = client.session.send_audio_data(stream_id, a_seq_h.clone(), RtmpTimestamp::new(0), false)
                    .map_err(|_| Error::SessionError("Failed to send audio data".into()))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }
        }

        self.handle_server_session_results(results)?;

        Ok(())
    }

    fn metadata_received(&mut self, app_name: &str, metadata: &StreamMetadata) -> Result<()> {
        debug!("Received stream metadata for app '{}'", app_name);

        let mut streams = self.shared.streams.write();
        if let Some(stream) = streams.get_mut(app_name) {
            stream.set_metadata(metadata.clone());
            let mut clients = self.shared.clients.lock();

            for client_id in &stream.watchers {
                let client = clients.get_mut(client_id).unwrap();

                if let Some(watched_stream) = client.watched_stream() {
                    let packet = client.session
                        .send_metadata(watched_stream, Rc::new(metadata.clone()))
                        .map_err(|_| Error::SessionError("Failed to send metadata".into()))?;

                    self.results.push_back(EventResult::Outbound(self.peer_id, packet));
                }
            }
        }

        Ok(())
    }

    fn multimedia_data_received(&mut self, stream_key: &str, media: &Media) -> Result<()> {
        // debug!("Received video data for stream with key {}", stream_key);

        #[cfg(feature = "hls")]
        self.send_to_hls_writer(media.clone());

        let app_name = self.shared
            .app_name_from_stream_key(&stream_key)
            .ok_or_else(|| Error::SessionError("No app for stream key".into()))?;

        let mut streams = self.shared.streams.write();
        if let Some(stream) = streams.get_mut(&app_name) {
            match &media {
                Media::AAC(_, ref data) if media.is_sequence_header() => {
                    stream.audio_seq_header = Some(data.clone());
                },
                Media::H264(_, ref data) if media.is_sequence_header() => {
                    stream.video_seq_header = Some(data.clone());
                },
                _ => (),
            }

            for client_id in &stream.watchers {
                let mut clients = self.shared.clients.lock();
                let client = match clients.get_mut(&client_id) {
                    Some(client) => client,
                    None => continue,
                };

                if !(client.received_video_keyframe || media.is_sendable()) {
                    continue;
                }

                if let Some(active_stream) = client.watched_stream() {
                    let packet = match &media {
                        Media::AAC(timestamp, bytes) => {
                            client.session.send_audio_data(active_stream, bytes.clone(), timestamp.clone(), true)?
                        }
                        Media::H264(timestamp, ref bytes) => {
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

    #[cfg(feature = "hls")]
    fn register_on_hls_server(&mut self, app_name: String) {
        if let Some(sender) = self.shared.hls_sender() {
            let (request, response) = oneshot::channel();
            sender.unbounded_send((app_name, request))
                .map_err(|err| error!("{:?}", err))
                .map(|_| {
                    response.map(|hls_writer_handle| {
                        self.media_sender = Some(hls_writer_handle);
                    })
                    .wait().unwrap()
                })
                .unwrap();
        }
    }


    #[cfg(feature = "hls")]
    fn send_to_hls_writer(&self, media: Media) {
        if let Some(media_sender) = &self.media_sender {
            media_sender.unbounded_send(media).unwrap();
        }
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        let mut clients = self.shared.clients.lock();
        clients.remove(&self.peer_id);
    }
}


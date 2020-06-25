use {
    std::{collections::VecDeque, rc::Rc},
    rml_rtmp::{
        sessions::{
            ServerSessionResult,
            ServerSessionEvent as Event,
            StreamMetadata,
            ServerSession,
            ServerSessionConfig,
        },
        chunk_io::Packet,
        time::RtmpTimestamp
    },
    bytes::Bytes,
    crate::{
        config::{RepublishAction, RtmpConfig},
        shared::Shared,
        media::{Media, Channel},
    },
    super::{Error, peer},
};

#[cfg(feature = "hls")]
use {
    futures::{sync::oneshot, Future},
    crate::media,
};


#[derive(Debug)]
pub enum EventResult {
    Outbound(u64, Packet),
    Metadata(u64, StreamMetadata),
    VideoData(u64, RtmpTimestamp, Bytes),
    AudioData(u64, RtmpTimestamp, Bytes),
    Disconnect,
}


pub struct Handler {
    peer_id: u64,
    results: VecDeque<EventResult>,
    shared: Shared,
    config: RtmpConfig,
    session: ServerSession,
    stream_id: Option<u32>,
    received_video_keyframe: bool,
    #[cfg(feature = "hls")]
    media_sender: Option<media::Sender>,
}

impl Handler {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(peer_id: u64, shared: Shared, config: RtmpConfig) -> Result<Self, Error> {
        let session_config = ServerSessionConfig::new();
        let (session, results) = ServerSession::new(session_config)
            .map_err(|_| Error::SessionCreationFailed)?;

        let mut this = Self {
            peer_id,
            results: VecDeque::new(),
            shared,
            config,
            session,
            stream_id: None,
            received_video_keyframe: false,
            #[cfg(feature = "hls")]
            media_sender: None,
        };

        this.handle_server_session_results(results)?;

        Ok(this)
    }

    pub fn handle(&mut self, bytes: &[u8]) -> Result<Vec<EventResult>, Error> {
        let results = self.session.handle_input(bytes)
            .map_err(|_| Error::InvalidInput)?;

        self.handle_server_session_results(results)?;

        Ok(self.results.drain(..).collect())
    }

    pub fn pack_metadata(&mut self, metadata: StreamMetadata) -> Result<Packet, Error> {
        let stream_id = self.stream_id.ok_or(Error::NoStreamId)?;
        self.session
            .send_metadata(stream_id, Rc::new(metadata))
            .map_err(|_| Error::DataPreparationFailed("metadata"))
    }

    pub fn pack_video(&mut self, timestamp: RtmpTimestamp, payload: Bytes) -> Result<Packet, Error> {
        let stream_id = self.stream_id.ok_or(Error::NoStreamId)?;
        self.session
            .send_video_data(stream_id, payload, timestamp, true)
            .map_err(|_| Error::DataPreparationFailed("metadata"))
    }

    pub fn pack_audio(&mut self, timestamp: RtmpTimestamp, payload: Bytes) -> Result<Packet, Error> {
        let stream_id = self.stream_id.ok_or(Error::NoStreamId)?;
        self.session
            .send_audio_data(stream_id, payload, timestamp, true)
            .map_err(|_| Error::DataPreparationFailed("metadata"))
    }

    fn handle_server_session_results(&mut self, results: Vec<ServerSessionResult>) -> Result<(), Error> {
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
                    log::debug!("Unhandleable message received");
                },
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<(), Error> {
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
            VideoDataReceived { app_name, data, timestamp, .. } => {
                self.multimedia_data_received(&app_name, &Media::H264(timestamp, data))?;
            },
            AudioDataReceived { app_name, data, timestamp, .. } => {
                self.multimedia_data_received(&app_name, &Media::AAC(timestamp, data))?;
            },
            PublishStreamFinished { app_name, .. } => {
                self.publish_stream_finished(&app_name)?;
            },
            _ => {
                log::debug!("Event: {:?}", event);
            }
        }

        Ok(())
    }

    fn connection_requested(&mut self, request_id: u32, app_name: &str) -> Result<(), Error> {
        log::info!("Connection request from client {} for app '{}'", self.peer_id, app_name);

        if app_name.is_empty() {
            return Err(Error::ApplicationNameRequired);
        }

        let results = self.session.accept_request(request_id)
            .map_err(|_| Error::RequestNotAccepted(request_id))?;

        self.handle_server_session_results(results)?;

        Ok(())
    }

    fn publish_requested(&mut self, request_id: u32, app_name: String, stream_key: String) -> Result<(), Error> {
        log::info!(
            "Client {} requested publishing to app '{}' using stream key {}",
            self.peer_id,
            app_name,
            stream_key
        );

        {
            if stream_key.is_empty() || !self.config.permitted_stream_keys.contains(&stream_key) {
                return Err(Error::StreamKeyNotPermitted(stream_key));
            }
        }

        log::debug!("Stream key '{}' permitted", stream_key);

        {
            let mut streams = self.shared.streams.write();
            if let Some(stream) = streams.get_mut(&app_name) {
                if let Some(publisher) = &stream.publisher {
                    match self.config.republish_action {
                        RepublishAction::Replace => {
                            log::info!("Another client is already publishing to this app, removing client");
                            let peers = self.shared.peers.write();
                            let peer = peers.get(publisher).unwrap();
                            peer.unbounded_send(peer::Message::Disconnect).unwrap();
                            stream.unpublish();
                        },
                        RepublishAction::Deny => {
                            return Err(Error::ApplicationInUse(app_name));
                        }
                    }
                }
            }
        }

        // TODO: lift out of event handler
        #[cfg(feature = "hls")]
        self.register_on_hls_server(app_name.clone());

        {
            let mut streams = self.shared.streams.write();
            let stream = streams.entry(app_name.clone()).or_insert_with(Channel::new);
            stream.set_publisher(self.peer_id, stream_key.clone());
        }

        let results = self.session.accept_request(request_id)
            .map_err(|why| {
                log::error!("Error while accepting publishing request: {:?}", why);
                Error::PublishRequestFailed
            })?;

        self.handle_server_session_results(results)?;

        Ok(())
    }

    fn publish_stream_finished(&mut self, app_name: &str) -> Result<(), Error> {
        log::info!("Publishing of app '{}' finished", app_name);

        {
            let mut streams = self.shared.streams.write();
            if let Some(stream) = streams.get_mut(app_name) {
                stream.unpublish();
            }
        }

        self.results.push_back(EventResult::Disconnect);

        Ok(())
    }

    fn play_requested(&mut self, request_id: u32, app_name: &str, stream_id: u32) -> Result<(), Error> {
        log::info!("Client {} requested playback of app '{}'", self.peer_id, app_name);

        self.stream_id = Some(stream_id);

        {
            let mut streams = self.shared.streams.write();
            let stream = streams.entry(app_name.to_string()).or_insert_with(Channel::new);
            stream.add_watcher(self.peer_id);
        }

        let results = self
            .session
            .accept_request(request_id)
            .map_err(|_| Error::RequestNotAccepted(request_id))?;

        {
            let streams = self.shared.streams.read();

            if let Some(ref metadata) = streams.get(app_name).unwrap().metadata {
                let packet = self
                    .session
                    .send_metadata(stream_id, Rc::new(metadata.clone()))
                    .map_err(|_| Error::DataPreparationFailed("metadata"))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }

            if let Some(ref v_seq_h) = streams.get(app_name).unwrap().video_seq_header {
                let packet = self
                    .session
                    .send_video_data(stream_id, v_seq_h.clone(), RtmpTimestamp::new(0), false)
                    .map_err(|_| Error::DataPreparationFailed("video data"))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }

            if let Some(ref a_seq_h) = streams.get(app_name).unwrap().audio_seq_header {
                let packet = self
                    .session
                    .send_audio_data(stream_id, a_seq_h.clone(), RtmpTimestamp::new(0), false)
                    .map_err(|_| Error::DataPreparationFailed("audio data"))?;
                self.results.push_back(EventResult::Outbound(self.peer_id, packet));
            }
        }

        self.handle_server_session_results(results)?;

        Ok(())
    }

    fn metadata_received(&mut self, app_name: &str, metadata: &StreamMetadata) -> Result<(), Error> {
        log::debug!("Received stream metadata for app '{}'", app_name);

        let mut streams = self.shared.streams.write();
        if let Some(stream) = streams.get_mut(app_name) {
            stream.set_metadata(metadata.clone());

            for peer_id in &stream.watchers {
                self.results.push_back(EventResult::Metadata(*peer_id, metadata.clone()));
            }
        }

        Ok(())
    }

    fn multimedia_data_received(&mut self, app_name: &str, media: &Media) -> Result<(), Error> {
        // debug!("Received video data for stream with key {}", stream_key);

        // TODO: lift out of event handler
        #[cfg(feature = "hls")]
        self.send_to_hls_writer(media.clone());

        let mut streams = self.shared.streams.write();
        if let Some(stream) = streams.get_mut(app_name) {
            match &media {
                Media::AAC(_, ref data) if media.is_sequence_header() => {
                    stream.audio_seq_header = Some(data.clone());
                },
                Media::H264(_, ref data) if media.is_sequence_header() => {
                    stream.video_seq_header = Some(data.clone());
                },
                _ => (),
            }

            for peer_id in &stream.watchers {
                if !(self.received_video_keyframe || media.is_sendable()) {
                    continue;
                }

                match &media {
                    Media::AAC(timestamp, bytes) => {
                        self.results.push_back(EventResult::AudioData(*peer_id, *timestamp, bytes.clone()));
                    }
                    Media::H264(timestamp, ref bytes) => {
                        if media.is_keyframe() {
                            self.received_video_keyframe = true;
                        }
                        self.results.push_back(EventResult::VideoData(*peer_id, *timestamp, bytes.clone()));
                    },
                };
            }
        }

        Ok(())
    }

    #[cfg(feature = "hls")]
    fn register_on_hls_server(&mut self, app_name: String) {
        if let Some(sender) = self.shared.hls_sender() {
            let (request, response) = oneshot::channel();

            if let Err(err) = sender.unbounded_send((app_name, request)) {
                log::error!("{}", err);
            }

            if let Err(err) = response.map(|hls_writer_handle| {
                self.media_sender = Some(hls_writer_handle);
            }).wait() {
                log::error!("{}", err);
            }
        }
    }


    #[cfg(feature = "hls")]
    fn send_to_hls_writer(&self, media: Media) {
        if let Some(media_sender) = &self.media_sender {
            media_sender.unbounded_send(media).unwrap();
        }
    }
}

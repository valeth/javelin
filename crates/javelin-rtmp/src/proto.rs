use std::convert::{TryFrom, TryInto};

use bytes::Bytes;
use javelin_types::{Packet, PacketType};
use rml_rtmp::handshake::{Handshake, HandshakeProcessResult, PeerType};
use rml_rtmp::sessions::{
    ServerSession, ServerSessionConfig, ServerSessionEvent, ServerSessionResult,
};
use rml_rtmp::time::RtmpTimestamp;
use thiserror::Error;
use tracing::debug;

use crate::convert;


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


pub enum Event {
    ReturnData(Bytes),
    SendPacket(Packet),
    AcquireSession {
        app_name: String,
        stream_key: String,
    },
    JoinSession {
        app_name: String,
        stream_key: String,
    },
    SendInitData {
        app_name: String,
    },
    ReleaseSession,
    LeaveSession,
}


enum State {
    HandshakePending,
    Ready,
    Publishing,
    Playing { stream_id: u32 },
    Finished,
}


pub struct Protocol {
    state: State,
    return_queue: Vec<Event>,
    handshake: Handshake,
    session: Option<ServerSession>,
}

impl Protocol {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_bytes(&mut self, input: &[u8]) -> Result<Vec<Event>, Error> {
        match &mut self.state {
            State::HandshakePending => {
                self.perform_handshake(input)?;
            }
            _ => {
                self.handle_input(input)?;
            }
        }

        Ok(self.return_queue.drain(..).collect())
    }

    fn handle_input(&mut self, input: &[u8]) -> Result<(), Error> {
        let results = self
            .session()?
            .handle_input(input)
            .map_err(|_| Error::InvalidInput)?;
        self.handle_results(results)?;
        Ok(())
    }

    fn perform_handshake(&mut self, input: &[u8]) -> Result<(), Error> {
        let result = self
            .handshake
            .process_bytes(input)
            .map_err(|_| Error::HandshakeFailed)?;

        match result {
            HandshakeProcessResult::InProgress { response_bytes } => {
                self.emit(Event::ReturnData(response_bytes.into()));
            }
            HandshakeProcessResult::Completed {
                response_bytes,
                remaining_bytes,
            } => {
                debug!("RTMP handshake successful");
                if !response_bytes.is_empty() {
                    self.emit(Event::ReturnData(response_bytes.into()));
                }

                self.initialize_session()?;

                if !remaining_bytes.is_empty() {
                    self.handle_input(&remaining_bytes)?;
                }

                self.state = State::Ready;
            }
        }

        Ok(())
    }

    fn initialize_session(&mut self) -> Result<(), Error> {
        let config = ServerSessionConfig::new();
        let (session, results) =
            ServerSession::new(config).map_err(|_| Error::SessionInitializationFailed)?;
        self.session = Some(session);
        self.handle_results(results)
    }

    fn accept_request(&mut self, id: u32) -> Result<(), Error> {
        let results = {
            let session = self.session()?;
            session
                .accept_request(id)
                .map_err(|_| Error::RequestRejected)?
        };
        self.handle_results(results)
    }

    pub fn pack_metadata(&mut self, packet: Packet) -> Result<Vec<u8>, Error> {
        let stream_id = self.stream_id()?;
        let metadata = convert::into_metadata(packet.try_into().unwrap());
        self.session()?
            .send_metadata(stream_id, &metadata)
            .map_err(|_| Error::InvalidInput)
            .map(|v| v.bytes)
    }

    pub fn pack_video(&mut self, packet: Packet) -> Result<Vec<u8>, Error> {
        let stream_id = self.stream_id()?;
        let data = packet.payload;
        let timestamp = packet
            .timestamp
            .map(|v| RtmpTimestamp::new(v.into()))
            .unwrap();

        self.session()?
            .send_video_data(stream_id, data, timestamp, false)
            .map_err(|_| Error::InvalidInput)
            .map(|v| v.bytes)
    }

    pub fn pack_audio(&mut self, packet: Packet) -> Result<Vec<u8>, Error> {
        let stream_id = self.stream_id()?;
        let data = packet.payload;
        let timestamp = packet
            .timestamp
            .map(|v| RtmpTimestamp::new(v.into()))
            .unwrap();

        self.session()?
            .send_audio_data(stream_id, data, timestamp, false)
            .map_err(|_| Error::InvalidInput)
            .map(|v| v.bytes)
    }

    fn handle_results(&mut self, results: Vec<ServerSessionResult>) -> Result<(), Error> {
        for result in results {
            match result {
                ServerSessionResult::OutboundResponse(packet) => {
                    self.emit(Event::ReturnData(packet.bytes.into()));
                }
                ServerSessionResult::RaisedEvent(event) => {
                    self.handle_event(event)?;
                }
                ServerSessionResult::UnhandleableMessageReceived(_) => (),
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: ServerSessionEvent) -> Result<(), Error> {
        use ServerSessionEvent::*;

        match event {
            ConnectionRequested {
                request_id,
                app_name,
                ..
            } => {
                if app_name.is_empty() {
                    return Err(Error::EmptyAppName);
                }

                self.accept_request(request_id)?;
            }
            PublishStreamRequested {
                request_id,
                app_name,
                stream_key,
                ..
            } => {
                self.emit(Event::AcquireSession {
                    app_name,
                    stream_key,
                });
                self.accept_request(request_id)?;
                self.state = State::Publishing;
            }
            PublishStreamFinished { .. } => {
                self.emit(Event::ReleaseSession);
                self.state = State::Finished;
            }
            PlayStreamRequested {
                request_id,
                app_name,
                stream_key,
                stream_id,
                ..
            } => {
                self.emit(Event::JoinSession {
                    app_name: app_name.clone(),
                    stream_key,
                });
                self.accept_request(request_id)?;
                self.emit(Event::SendInitData { app_name });
                self.state = State::Playing { stream_id };
            }
            PlayStreamFinished { .. } => {
                self.emit(Event::LeaveSession);
                self.state = State::Finished;
            }
            AudioDataReceived {
                data, timestamp, ..
            } => {
                let packet = Packet::new_audio(timestamp.value, data);
                self.emit(Event::SendPacket(packet));
            }
            VideoDataReceived {
                data, timestamp, ..
            } => {
                let packet = Packet::new_video(timestamp.value, data);
                self.emit(Event::SendPacket(packet));
            }
            StreamMetadataChanged { metadata, .. } => {
                let metadata = convert::from_metadata(metadata);
                let payload = Bytes::try_from(metadata).unwrap();
                let packet = Packet::new::<u32, Bytes>(PacketType::Meta, None, payload);
                self.emit(Event::SendPacket(packet));
            }
            _ => (),
        }

        Ok(())
    }

    fn emit(&mut self, event: Event) {
        self.return_queue.push(event);
    }

    fn stream_id(&self) -> Result<u32, Error> {
        match self.state {
            State::Playing { stream_id } => Ok(stream_id),
            _ => Err(Error::NoStreamId),
        }
    }

    fn session(&mut self) -> Result<&mut ServerSession, Error> {
        self.session.as_mut().ok_or(Error::SessionNotInitialized)
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self {
            state: State::HandshakePending,
            return_queue: Vec::with_capacity(8),
            handshake: Handshake::new(PeerType::Server),
            session: None,
        }
    }
}

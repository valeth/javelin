use {
    std::convert::TryFrom,
    futures::SinkExt,
    tokio::{
        prelude::*,
        stream::StreamExt,
        sync::oneshot,
    },
    tokio_util::codec::{Framed, BytesCodec},
    javelin_types::{Packet, PacketType, Metadata},
    javelin_core::{
        session::{self, Message, Sender, Receiver, Session, Trigger as HlsTrigger},
        shared::Shared,
    },
    super::{
        Config,
        error::Error,
    },
    crate::{Protocol, Event},
};


enum State {
    Initializing,
    Publishing,
    Playing,
    Disconnecting,
}

/// Represents an incoming connection
pub struct Peer<S>
    where S: AsyncRead + AsyncWrite + Unpin
{
    id: u64,
    bytes_stream: Framed<S, BytesCodec>,
    session_receiver: Receiver,
    session_sender: Sender,
    shared: Shared,
    proto: Protocol,
    config: Config,
    app_name: Option<String>,
    state: State,
    hls_handle: HlsTrigger,
}

impl<S> Peer<S>
    where S: AsyncRead + AsyncWrite + Unpin
{
    pub fn new(id: u64, bytes_stream: S, shared: Shared, hls_handle: HlsTrigger, config: Config) -> Self {
        let (session_sender, session_receiver) = session::channel();

        Self {
            id,
            bytes_stream: Framed::new(bytes_stream, BytesCodec::new()),
            session_receiver,
            session_sender,
            shared,
            proto: Protocol::new(),
            config,
            app_name: None,
            state: State::Initializing,
            hls_handle,
        }
    }

    pub async fn run(mut self) -> Result<(), Error> {
        loop {
            while let Ok(msg) = self.session_receiver.try_recv() {
               if self.handle_session_message(msg).await.is_err() {
                   self.state = State::Disconnecting;
               }
            }

            match self.state {
                State::Initializing | State::Publishing => {
                    match self.bytes_stream.try_next().await {
                        Ok(Some(data)) => {
                            for event in self.proto.handle_bytes(&data).unwrap() {
                                self.handle_event(event).await?;
                            }
                        },
                        Ok(None) => {
                            return Ok(());
                        },
                        Err(why) => {
                            log::error!("{}", why);
                            return Ok(())
                        },
                    }
                },
                State::Disconnecting => {
                    log::debug!("Disconnecting...");
                    return Ok(());
                },
                _ => (),
            }
        }
    }

    async fn handle_session_message(&mut self, message: Message) -> Result<(), Error> {
        match message {
            Message::Packet(packet) => {
                log::debug!("Peer {}: received {:?}", self.id, packet.kind);
                match packet.kind  {
                    PacketType::Meta => {
                        let metadata = Metadata::try_from(packet).unwrap();
                        let bytes= self.proto.pack_metadata(metadata)?;
                        self.bytes_stream.send(bytes.into()).await?;
                    },
                    PacketType::Video => {
                        let bytes = self.proto.pack_video(packet)?;
                        self.bytes_stream.send(bytes.into()).await?;
                    },
                    PacketType::Audio => {
                        let bytes = self.proto.pack_audio(packet)?;
                        self.bytes_stream.send(bytes.into()).await?;
                    },
                }
            },
            _ => ()
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::ReturnData(data) => {
                self.bytes_stream.send(data).await.expect("Failed to return data");
            },
            Event::SendPacket(packet) => {
                let app_name = self.app_name.as_ref().unwrap();
                let mut streams = self.shared.streams.write().await;
                let stream = streams
                    .get_mut(app_name)
                    .ok_or_else(|| Error::NoSuchStream(app_name.clone()))?;
                stream.set_cache(packet.clone()).unwrap();
                stream.send_to_watchers(Message::Packet(packet));
            },
            Event::AcquireSession { app_name, stream_key } => {
                self.authenticate(&app_name, &stream_key)?;
                self.app_name = Some(app_name.clone());
                let mut streams = self.shared.streams.write().await;
                streams.remove(&app_name);
                let mut stream = Session::new();
                // TODO: move into session when implemented
                register_on_hls_server(&mut stream, &mut self.hls_handle, &app_name).await;
                streams.insert(app_name, stream);
                self.state = State::Publishing;
            },
            Event::JoinSession { app_name, .. } => {
                let mut streams = self.shared.streams.write().await;
                let stream = streams
                    .get_mut(&app_name)
                    .ok_or_else(|| Error::NoSuchStream(app_name))?;
                stream.add_watcher(self.session_sender.clone());
                self.state = State::Playing;
            },
            Event::SendInitData { app_name } => {
                let streams = self.shared.streams.read().await;
                let stream = streams
                    .get(&app_name)
                    .ok_or_else(|| Error::NoSuchStream(app_name))?;

                if let Some(metadata) = &stream.metadata {
                    let packet = Packet::try_from(metadata.clone()).unwrap();
                    self.send_back(Message::Packet(packet));
                }

                if let Some(audio_header) = &stream.audio_seq_header {
                    let packet = Packet::new_audio(0u32, audio_header.clone());
                    self.send_back(Message::Packet(packet));
                }

                if let Some(video_header) = &stream.video_seq_header {
                    let packet = Packet::new_video(0u32, video_header.clone());
                    self.send_back(Message::Packet(packet));
                }
            }
            Event::ReleaseSession => {
                let app_name = self.app_name.as_ref().unwrap();
                let mut streams = self.shared.streams.write().await;
                streams.remove(app_name);
                self.state = State::Disconnecting;
            }
            Event::LeaveSession => {
                self.state = State::Disconnecting;
            },
        }

        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    fn authenticate(&self, app: &str, key: &String) -> Result<(), Error> {
        if key.is_empty() || self.config.stream_keys.get(app) != Some(key) {
            return Err(Error::StreamKeyNotPermitted(key.to_string()));
        }

        Ok(())
    }

    fn send_back(&self, message: Message) {
        log::debug!("Peer {}: Send back message", self.id);
        if let Err(why) = self.session_sender.send(message) {
            log::error!("Failed to send message back to peer {}: {}", self.id, why);
        }
    }
}

impl<S> Drop for Peer<S>
    where S: AsyncRead + AsyncWrite + Unpin
{
    fn drop(&mut self) {
        log::info!("Client {} disconnected", self.id);
    }
}


async fn register_on_hls_server(stream: &mut session::Session, hls_handle: &mut HlsTrigger, app_name: &str) {
    let (request, response) = oneshot::channel();

    if let Err(err) = hls_handle.send((app_name.to_string(), request)) {
        log::error!("{}", err);
        return;
    }

    match response.await {
        Ok(hls_sender) => stream.add_watcher(hls_sender),
        Err(why) => log::error!("{}", why)
    }
}

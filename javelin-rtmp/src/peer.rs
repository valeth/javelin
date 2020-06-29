use {
    std::convert::TryFrom,
    futures::SinkExt,
    tokio::{
        prelude::*,
        stream::StreamExt,
        sync::{oneshot, mpsc},
    },
    tokio_util::codec::{Framed, BytesCodec},
    javelin_types::{Packet, PacketType, Metadata},
    javelin_core::session::{self, Message, ManagerMessage},
    super::{
        Config,
        error::Error,
    },
    crate::{Protocol, Event},
};


type ReturnQueue<P> = (mpsc::UnboundedSender<P>, mpsc::UnboundedReceiver<P>);

enum State {
    Initializing,
    Publishing(session::Handle),
    Playing(session::Handle, session::Watcher),
    Disconnecting,
}

/// Represents an incoming connection
pub struct Peer<S>
    where S: AsyncRead + AsyncWrite + Unpin
{
    id: u64,
    bytes_stream: Framed<S, BytesCodec>,
    session_manager: session::ManagerHandle,
    return_queue: ReturnQueue<Packet>,
    proto: Protocol,
    config: Config,
    app_name: Option<String>,
    state: State,
}

impl<S> Peer<S>
    where S: AsyncRead + AsyncWrite + Unpin
{
    pub fn new(id: u64, stream: S, session_manager: session::ManagerHandle, config: Config) -> Self {
        Self {
            id,
            bytes_stream: Framed::new(stream, BytesCodec::new()),
            session_manager,
            return_queue: mpsc::unbounded_channel(),
            proto: Protocol::new(),
            config,
            app_name: None,
            state: State::Initializing,
        }
    }

    pub async fn run(mut self) -> Result<(), Error> {
        loop {
            while let Ok(packet) = self.return_queue.1.try_recv() {
               if self.handle_return_packet(packet).await.is_err() {
                   self.state = State::Disconnecting;
               }
            }

            match &mut self.state {
                State::Initializing | State::Publishing(_) => {
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
                State::Playing(_, watcher) => {
                    if let Ok(packet) = watcher.try_recv() {
                        self.send_back(packet);
                    }
                }
                State::Disconnecting => {
                    log::debug!("Disconnecting...");
                    return Ok(());
                },
            }
        }
    }

    async fn handle_return_packet(&mut self, packet: Packet) -> Result<(), Error> {
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

        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::ReturnData(data) => {
                self.bytes_stream.send(data).await.expect("Failed to return data");
            },
            Event::SendPacket(packet) => {
                if let State::Publishing(session) = &mut self.state {
                    session
                        .send(Message::Packet(packet))
                        .map_err(|_| Error::SessionSendFailed)?;
                }
            },
            Event::AcquireSession { app_name, stream_key } => {
                self.authenticate(&app_name, &stream_key)?;
                self.app_name = Some(app_name.clone());
                let (request, response) = oneshot::channel();
                self.session_manager
                    .send(ManagerMessage::CreateSession((app_name, request)))
                    .map_err(|_| Error::SessionCreationFailed)?;
                let session_sender = response.await.unwrap();
                self.state = State::Publishing(session_sender);
            },
            Event::JoinSession { app_name, .. } => {
                let (request, response) = oneshot::channel();
                self.session_manager
                    .send(ManagerMessage::JoinSession((app_name, request)))
                    .map_err(|_| Error::SessionJoinFailed)?;

                match response.await {
                    Ok((session_sender, session_receiver)) => {
                        self.state = State::Playing(session_sender, session_receiver);
                    }
                    Err(_) => self.state = State::Disconnecting,
                }
            },
            Event::SendInitData { .. } => {
                // TODO: better initialization handling
                if let State::Playing(session, _) = &mut self.state {
                    let (request, response) = oneshot::channel();
                    session
                        .send(Message::GetInitData(request))
                        .map_err(|_| Error::SessionSendFailed)?;

                    if let Ok((Some(meta), Some(video), Some(audio))) = response.await {
                        self.send_back(meta);
                        self.send_back(video);
                        self.send_back(audio);
                    }
                }
            }
            Event::ReleaseSession => {
                let app_name = self.app_name.clone().unwrap();
                if let State::Publishing(session) = &mut self.state {
                    session.send(Message::Disconnect).map_err(|_| Error::SessionSendFailed)?;
                }
                self.session_manager
                    .send(ManagerMessage::ReleaseSession(app_name))
                    .map_err(|_| Error::SessionReleaseFailed)?;
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

    fn send_back(&mut self, packet: Packet) {
        log::debug!("Peer {}: Send back packet", self.id);
        if let Err(why) = self.return_queue.0.send(packet) {
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

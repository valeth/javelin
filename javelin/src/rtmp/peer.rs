use {
    std::convert::TryFrom,
    futures::{try_ready, sync::oneshot},
    tokio::prelude::*,
    javelin_rtmp::{Protocol, Event},
    javelin_types::{Packet, PacketType, Metadata},
    javelin_core::{
        BytesStream,
        session::{self, Message, Sender, Receiver, Session, Trigger as HlsTrigger},
        shared::Shared,
    },
    super::{
        Config,
        error::Error,
    },
};


/// Represents an incoming connection
pub struct Peer<S>
    where S: AsyncRead + AsyncWrite
{
    id: u64,
    bytes_stream: BytesStream<S>,
    session_receiver: Receiver,
    session_sender: Sender,
    shared: Shared,
    proto: Protocol,
    config: Config,
    app_name: Option<String>,
    disconnecting: bool,
    hls_handle: HlsTrigger,
}

impl<S> Peer<S>
    where S: AsyncRead + AsyncWrite
{
    pub fn new(id: u64, bytes_stream: BytesStream<S>, shared: Shared, hls_handle: HlsTrigger, config: Config) -> Self {
        let (session_sender, session_receiver) = session::channel();

        Self {
            id,
            bytes_stream,
            session_receiver,
            session_sender,
            shared,
            proto: Protocol::new(),
            config,
            app_name: None,
            disconnecting: false,
            hls_handle,
        }
    }

    fn handle_message(&mut self, message: Message) -> Result<(), Error> {
        match message {
            Message::Packet(packet) => {
                log::debug!("Peer {}: received {:?}", self.id, packet.kind);
                match packet.kind  {
                    PacketType::Meta => {
                        let metadata = Metadata::try_from(packet).unwrap();
                        let bytes= self.proto.pack_metadata(metadata).unwrap();
                        self.bytes_stream.fill_write_buffer(&bytes);
                    },
                    PacketType::Video => {
                        let bytes = self.proto.pack_video(packet).unwrap();
                        self.bytes_stream.fill_write_buffer(&bytes);
                    },
                    PacketType::Audio => {
                        let bytes = self.proto.pack_audio(packet).unwrap();
                        self.bytes_stream.fill_write_buffer(&bytes);
                    },
                }
            },
            Message::Disconnect => {
                self.disconnecting = true;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::ReturnData(data) => {
                self.bytes_stream.fill_write_buffer(data.as_ref());
            },
            Event::SendPacket(packet) => {
                let app_name = self.app_name.as_ref().unwrap();
                let mut streams = self.shared.streams.write();
                let stream = streams
                    .get_mut(app_name)
                    .ok_or_else(|| Error::NoSuchStream(app_name.clone()))?;
                stream.set_cache(packet.clone()).unwrap();
                stream.send_to_watchers(Message::Packet(packet));
            },
            Event::AcquireSession { app_name, stream_key } => {
                self.authenticate(&app_name, &stream_key)?;
                self.app_name = Some(app_name.clone());
                let mut streams = self.shared.streams.write();
                streams.remove(&app_name);
                let mut stream = Session::new();
                // TODO: move into session when implemented
                register_on_hls_server(&mut stream, &self.hls_handle, &app_name);
                streams.insert(app_name, stream);
            },
            Event::JoinSession { app_name, .. } => {
                let mut streams = self.shared.streams.write();
                let stream = streams
                    .get_mut(&app_name)
                    .ok_or_else(|| Error::NoSuchStream(app_name))?;
                stream.add_watcher(self.session_sender.clone());
            },
            Event::SendInitData { app_name } => {
                let streams = self.shared.streams.read();
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
                let mut streams = self.shared.streams.write();
                streams.remove(app_name);
                self.send_back(Message::Disconnect);
            }
            Event::LeaveSession => {
                self.send_back(Message::Disconnect);
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
        if let Err(why) = self.session_sender.unbounded_send(message) {
            log::error!("Failed to send message back to peer {}: {}", self.id, why);
        }
    }
}

impl<S> Drop for Peer<S>
    where S: AsyncRead + AsyncWrite
{
    fn drop(&mut self) {
        log::info!("Client {} disconnected", self.id);
    }
}

impl<S> Future for Peer<S>
    where S: AsyncRead + AsyncWrite
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(Some(msg)) = self.session_receiver.poll().unwrap() {
            self.handle_message(msg)?;
            self.bytes_stream.poll_flush()?;
        }

        if self.disconnecting {
            return Ok(Async::Ready(()));
        }

        match try_ready!(self.bytes_stream.poll()) {
            Some(data) => {
                for event in self.proto.handle_bytes(&data).unwrap() {
                    self.handle_event(event)?;
                    self.bytes_stream.poll_flush()?;
                }
            },
            None => {
                return Ok(Async::Ready(()));
            },
        }

        Ok(Async::NotReady)
    }
}

#[cfg(feature = "hls")]
fn register_on_hls_server(stream: &mut session::Session, hls_handle: &HlsTrigger, app_name: &str) {
    let (request, response) = oneshot::channel();

    if let Err(err) = hls_handle.unbounded_send((app_name.to_string(), request)) {
        log::error!("{}", err);
        return;
    }

    if let Err(err) = response.map(|hls_writer_handle| {
        stream.add_watcher(hls_writer_handle)
    }).wait() {
        log::error!("{}", err);
    }
}

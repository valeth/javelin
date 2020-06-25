use {
    futures::{sync::mpsc, try_ready},
    tokio::prelude::*,
    bytes::{Bytes, BytesMut, BufMut},
    rml_rtmp::{
        handshake::{
            Handshake as RtmpHandshake,
            HandshakeProcessResult,
            PeerType,
        },
        sessions::StreamMetadata,
        time::RtmpTimestamp,
    },
    crate::{BytesStream, shared::Shared},
    super::{
        event::{
            Handler as EventHandler,
            EventResult,
        },
        error::Error,
    },
};


pub enum Message {
    Raw(Bytes),
    Metadata(StreamMetadata),
    VideoData(RtmpTimestamp, Bytes),
    AudioData(RtmpTimestamp, Bytes),
    Disconnect,
}

pub type Sender = mpsc::UnboundedSender<Message>;
type Receiver = mpsc::UnboundedReceiver<Message>;


/// Represents an incoming connection
pub struct Peer<S>
    where S: AsyncRead + AsyncWrite
{
    id: u64,
    bytes_stream: BytesStream<S>,
    sender: Sender,
    receiver: Receiver,
    shared: Shared,
    buffer: BytesMut,
    event_handler: EventHandler,
    disconnecting: bool,
    handshake_completed: bool,
    handshake: RtmpHandshake,
}

impl<S> Peer<S>
    where S: AsyncRead + AsyncWrite
{
    pub fn new(id: u64, bytes_stream: BytesStream<S>, shared: Shared) -> Self {
        let (sender, receiver) = mpsc::unbounded();
        let event_handler = EventHandler::new(id, shared.clone())
            .unwrap_or_else(|_| {
                panic!("Failed to create event handler for peer {}", id)
            });

        {
            let mut peers = shared.peers.write();
            peers.insert(id, sender.clone());
        }

        Self {
            id,
            bytes_stream,
            sender,
            receiver,
            shared,
            buffer: BytesMut::with_capacity(4096),
            event_handler,
            handshake_completed: false,
            disconnecting: false,
            handshake: RtmpHandshake::new(PeerType::Server),
        }
    }

    fn handle_handshake(&mut self) -> Poll<(), Error> {
        use self::HandshakeProcessResult as HandshakeState;

        let data = self.buffer.take().freeze();

        let response_bytes = match self.handshake.process_bytes(&data) {
            Err(why) => {
                log::error!("Handshake for peer {} failed: {}", self.id, why);
                return Err(Error::HandshakeFailed);
            },
            Ok(HandshakeState::InProgress { response_bytes }) => {
                log::debug!("Handshake pending...");
                response_bytes
            },
            Ok(HandshakeState::Completed { response_bytes, remaining_bytes }) => {
                log::info!("Handshake for client {} successful", self.id);
                log::debug!("Remaining bytes after handshake: {}", remaining_bytes.len());
                self.handshake_completed = true;

                if !remaining_bytes.is_empty() {
                    self.buffer.reserve(remaining_bytes.len());
                    self.buffer.put(remaining_bytes);
                    self.handle_incoming_bytes()?;
                }

                response_bytes
            }
        };

        if !response_bytes.is_empty() {
            self.sender
                .unbounded_send(Message::Raw(Bytes::from(response_bytes)))
                .map_err(|_| Error::HandshakeFailed)?
        }

        Ok(Async::Ready(()))
    }

    fn handle_incoming_bytes(&mut self) -> Result<(), Error> {
        let data = self.buffer.take();

        let event_results = self.event_handler.handle(&data)?;

        for result in event_results {
            match result {
                EventResult::Outbound(target_peer_id, packet) => {
                    let message = Message::Raw(Bytes::from(packet.bytes));
                    self.send_to_peer(target_peer_id, message);
                },
                EventResult::Metadata(target_peer_id, metadata) => {
                    let message = Message::Metadata(metadata.clone());
                    self.send_to_peer(target_peer_id, message);
                },
                EventResult::VideoData(target_peer_id, timestamp, payload) => {
                    let message = Message::VideoData(timestamp.clone(), payload.clone());
                    self.send_to_peer(target_peer_id, message);
                },
                EventResult::AudioData(target_peer_id, timestamp, payload) => {
                    let message = Message::AudioData(timestamp.clone(), payload.clone());
                    self.send_to_peer(target_peer_id, message);
                },
                EventResult::Disconnect => {
                    self.disconnecting = true;
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_message(&mut self, message: Message) -> Result<(), Error> {
        match message {
            Message::Raw(val) => {
                self.bytes_stream.fill_write_buffer(&val);
            },
            Message::Metadata(metadata) => {
                let packet = self.event_handler.pack_metadata(metadata)?;
                self.bytes_stream.fill_write_buffer(&packet.bytes);
            },
            Message::VideoData(timestamp, payload) => {
                let packet = self.event_handler.pack_video(timestamp, payload)?;
                self.bytes_stream.fill_write_buffer(&packet.bytes);
            },
            Message::AudioData(timestamp, payload) => {
                let packet = self.event_handler.pack_audio(timestamp, payload)?;
                self.bytes_stream.fill_write_buffer(&packet.bytes);
            },
            Message::Disconnect => {
                self.disconnecting = true;
            }
        }

        Ok(())
    }

    fn send_to_peer(&self, peer_id: u64, message: Message) {
        let peers = self.shared.peers.read();
        if let Some(peer) = peers.get(&peer_id) {
            if let Err(why) = peer.unbounded_send(message) {
                log::error!("Failed to send message to peer {}: {}", peer_id, why);
            }
        }
    }
}

impl<S> Drop for Peer<S>
    where S: AsyncRead + AsyncWrite
{
    fn drop(&mut self) {
        let mut peers = self.shared.peers.write();
        peers.remove(&self.id);

        log::info!("Closing connection: {}", self.id);
    }
}

impl<S> Future for Peer<S>
    where S: AsyncRead + AsyncWrite
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(Some(msg)) = self.receiver.poll().unwrap() {
            self.handle_message(msg)?;
            if self.disconnecting {
                break;
            }
        }

        let _ = self.bytes_stream.poll_flush()?;

        match try_ready!(self.bytes_stream.poll()) {
            Some(data) => {
                self.buffer.reserve(data.len());
                self.buffer.put(data);

                if self.handshake_completed {
                    self.handle_incoming_bytes()?;
                } else {
                    try_ready!(self.handle_handshake());
                }
            },
            None => {
                return Ok(Async::Ready(()));
            },
        }

        if self.disconnecting {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

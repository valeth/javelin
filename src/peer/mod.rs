mod bytes_stream;
mod client;
pub mod media;
mod event;


use futures::sync::mpsc;
use tokio::prelude::*;
use bytes::{Bytes, BytesMut, BufMut};
use rtmp::{
    handshake::{
        Handshake as RtmpHandshake,
        HandshakeProcessResult,
        PeerType,
    },
};
use error::{Error, Result};
use shared::Shared;
pub use self::bytes_stream::BytesStream;
pub use self::client::Client;
use self::event::{
    Handler as EventHandler,
    EventResult,
};


type Receiver = mpsc::UnboundedReceiver<Bytes>;
pub type Sender = mpsc::UnboundedSender<Bytes>;


/// Represents an incoming connection
pub struct Peer {
    id: u64,
    bytes_stream: BytesStream,
    sender: Sender,
    receiver: Receiver,
    shared: Shared,
    buffer: BytesMut,
    event_handler: EventHandler,
    handshake_completed: bool,
    handshake: RtmpHandshake,
}

impl Peer {
    pub fn new(id: u64, bytes_stream: BytesStream, shared: Shared) -> Self {
        let (sender, receiver) = mpsc::unbounded();
        let event_handler = EventHandler::new(id, shared.clone())
            .expect(&format!("Failed to create event handler for peer {}", id));

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
            handshake: RtmpHandshake::new(PeerType::Server),
        }
    }

    fn handle_handshake(&mut self) -> Poll<(), Error> {
        use self::HandshakeProcessResult as HandshakeState;

        let data = self.buffer.take().freeze();

        let response_bytes = match self.handshake.process_bytes(&data) {
            Err(why) => {
                error!("Handshake for peer {} failed: {}", self.id, why);
                return Err(Error::HandshakeFailed);
            },
            Ok(HandshakeState::InProgress { response_bytes }) => {
                debug!("Handshake pending...");
                response_bytes
            },
            Ok(HandshakeState::Completed { response_bytes, remaining_bytes }) => {
                info!("Handshake for client {} successful", self.id);
                debug!("Remaining bytes after handshake: {}", remaining_bytes.len());
                self.handshake_completed = true;
                self.buffer.reserve(remaining_bytes.len());
                self.buffer.put(remaining_bytes);
                response_bytes
            }
        };

        if response_bytes.len() > 0 {
            self.sender
                .unbounded_send(Bytes::from(response_bytes))
                .map_err(|_| Error::HandshakeFailed)?
        }

        Ok(Async::Ready(()))
    }

    fn handle_incoming_bytes(&mut self) -> Result<()> {
        let data = self.buffer.take();

        let event_results = self.event_handler.handle(&data)?;

        for result in event_results {
            match result {
                EventResult::Outbound(target_peer_id, packet) => {
                    let peers = self.shared.peers.read();
                    let peer = peers.get(&target_peer_id).unwrap();
                    debug!("Packet from {} to {} with {:?} bytes", self.id, target_peer_id, packet.bytes.len());
                    peer.unbounded_send(Bytes::from(packet.bytes)).unwrap();
                }
            }
        }

        Ok(())
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        {
            let clients = self.shared.clients.lock();
            let client = clients.get(&self.id).unwrap();

            if let Some(app_name) = client.publishing_app_name() {
                let mut streams = self.shared.streams.write();
                let stream = streams.get_mut(&app_name).unwrap();
                stream.unpublish();
            }

            if let Some(app_name) = client.watched_app_name() {
                let mut streams = self.shared.streams.write();
                let stream = streams.get_mut(&app_name).unwrap();
                stream.watchers.remove(&self.id);
            }
        }

        let mut peers = self.shared.peers.write();
        peers.remove(&self.id);

        info!("Closing connection: {}", self.id);
    }
}

impl Future for Peer {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // FIXME: potential starvation of socket stream?
        while let Async::Ready(Some(val)) = self.receiver.poll().unwrap() {
            self.bytes_stream.fill_write_buffer(&val);
        }

        let _ = self.bytes_stream.poll_flush()?;

        match try_ready!(self.bytes_stream.poll()) {
            Some(data) => {
                debug!("Received {} bytes", data.len());
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

        Ok(Async::NotReady)
    }
}

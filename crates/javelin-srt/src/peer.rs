use std::io;
use std::time::Instant;

use futures::{SinkExt, StreamExt};
use javelin_core::session::Message;
use javelin_types::{Packet, PacketType};
use srt_tokio::SrtSocket;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, trace, warn};

use crate::Error;


type SessionSender = UnboundedSender<Message>;
type SessionReceiver = Receiver<Packet>;


enum State {
    Publish,
    Receive(SessionReceiver),
}


pub struct Peer {
    session_sender: SessionSender,
    state: State,
}

impl Peer {
    pub fn new_publishing(session_sender: SessionSender) -> Self {
        Self {
            session_sender,
            state: State::Publish,
        }
    }

    pub fn new_receiving(session_sender: SessionSender, session_receiver: SessionReceiver) -> Self {
        Self {
            session_sender,
            state: State::Receive(session_receiver),
        }
    }
}


#[tracing::instrument(skip_all)]
pub(crate) async fn handle_peer(peer: Peer, sock: SrtSocket) {
    let result = match peer.state {
        State::Publish => handle_publishing_peer(sock, peer.session_sender).await,
        State::Receive(session_receiver) => {
            handle_receiving_peer(sock, peer.session_sender, session_receiver).await
        }
    };

    if let Err(err) = result {
        error!(%err);
    }

    trace!("Connection closed");
}


#[tracing::instrument(skip_all)]
async fn handle_receiving_peer(
    mut sock: SrtSocket,
    _session_sender: UnboundedSender<Message>,
    mut session_receiver: Receiver<Packet>,
) -> Result<(), Error> {
    let socket_id = sock.settings().remote_sockid;
    trace!(?socket_id);

    loop {
        match session_receiver.recv().await {
            Ok(packet) => {
                let timestamp = Instant::now();
                match sock.send((timestamp, packet.payload)).await {
                    Ok(_) => (),
                    Err(err) if err.kind() == io::ErrorKind::NotConnected => break,
                    Err(err) => return Err(err.into()),
                }
            }
            Err(RecvError::Closed) => {
                break;
            }
            Err(RecvError::Lagged(skipped_amount)) => {
                warn!(%skipped_amount ,"Client receiver lagged behind");
            }
        }
    }

    sock.close_and_finish().await?;

    Ok(())
}


#[tracing::instrument(skip_all)]
async fn handle_publishing_peer(
    mut sock: SrtSocket,
    session_sender: UnboundedSender<Message>,
) -> Result<(), Error> {
    let socket_id = sock.settings().remote_sockid;
    trace!(?socket_id);

    while let Some(data) = sock.next().await {
        let (_, bytes) = data?;
        let timestamp = chrono::Utc::now().timestamp();
        let packet = Packet::new(PacketType::Bytes, Some(timestamp), bytes);
        if session_sender.send(Message::Packet(packet)).is_err() {
            break;
        }
    }

    sock.close_and_finish().await?;

    Ok(())
}

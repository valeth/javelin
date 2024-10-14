use anyhow::Result;
use javelin_types::{packet, Packet};
use tracing::{error, info};

use super::transport::{IncomingBroadcast, Message, OutgoingBroadcast};


pub struct Session {
    incoming: IncomingBroadcast,
    outgoing: OutgoingBroadcast,
    metadata: Option<Packet>,
    video_seq_header: Option<Packet>,
    audio_seq_header: Option<Packet>,
    closing: bool,
}

impl Session {
    #[allow(clippy::new_without_default)]
    pub fn new(incoming: IncomingBroadcast, outgoing: OutgoingBroadcast) -> Self {
        Self {
            incoming,
            outgoing,
            metadata: None,
            video_seq_header: None,
            audio_seq_header: None,
            closing: false,
        }
    }

    pub async fn run(mut self) {
        while !self.closing {
            if let Some(message) = self.incoming.recv().await {
                self.handle_message(message);
            }
        }
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Packet(packet) => {
                self.set_cache(&packet)
                    .expect("Failed to set session cache");
                self.broadcast_packet(packet);
            }
            Message::GetInitData(responder) => {
                let response = (
                    self.metadata.clone(),
                    self.video_seq_header.clone(),
                    self.audio_seq_header.clone(),
                );
                if responder.send(response).is_err() {
                    error!("Failed to send init data");
                }
            }
            Message::Disconnect => {
                self.closing = true;
            }
        }
    }

    fn broadcast_packet(&self, packet: Packet) {
        if self.outgoing.receiver_count() != 0 && self.outgoing.send(packet).is_err() {
            error!("Failed to broadcast packet");
        }
    }

    fn set_cache(&mut self, packet: &Packet) -> Result<()> {
        match packet.content_type {
            packet::METADATA if self.metadata.is_none() => {
                self.metadata = Some(packet.clone());
            }
            packet::FLV_VIDEO_H264 if self.video_seq_header.is_none() => {
                self.video_seq_header = Some(packet.clone());
            }
            packet::FLV_AUDIO_AAC if self.audio_seq_header.is_none() => {
                self.audio_seq_header = Some(packet.clone());
            }
            _ => (),
        }

        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        info!("Closing session");
    }
}

use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    tokio::sync::{broadcast, mpsc, oneshot, RwLock},
    anyhow::Result,
    javelin_types::{Packet, PacketType},
};


pub type Trigger = mpsc::UnboundedSender<(String, Watcher)>;
pub type TriggerHandle = mpsc::UnboundedReceiver<(String, Watcher)>;

pub fn trigger_channel() -> (Trigger, TriggerHandle) {
    mpsc::unbounded_channel()
}


pub type Responder<P> = oneshot::Sender<P>;

pub enum Message {
    Packet(Packet),
    GetInitData(Responder<(Option<Packet>, Option<Packet>, Option<Packet>)>),
    Disconnect,
}

pub type Handle = mpsc::UnboundedSender<Message>;
pub type IncomingBroadcast = mpsc::UnboundedReceiver<Message>;
pub type OutgoingBroadcast = broadcast::Sender<Packet>;
pub type Watcher = broadcast::Receiver<Packet>;


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
                self.set_cache(&packet).expect("Failed to set session cache");
                self.broadcast_packet(packet);
            },
            Message::GetInitData(responder) => {
                let response = (self.metadata.clone(), self.video_seq_header.clone(), self.audio_seq_header.clone());
                if responder.send(response).is_err() {
                    log::error!("Failed to send init data");
                }
            },
            Message::Disconnect => {
                self.closing = true;
            },
        }
    }

    fn broadcast_packet(&self, packet: Packet) {
        if self.outgoing.receiver_count() != 0 && self.outgoing.send(packet).is_err() {
            log::error!("Failed to broadcast packet");
        }
    }

    fn set_cache(&mut self, packet: &Packet) -> Result<()> {
        match packet.kind {
            PacketType::Meta if self.metadata.is_none() => {
                self.metadata = Some(packet.clone());
            },
            PacketType::Video if self.video_seq_header.is_none() => {
                self.video_seq_header = Some(packet.clone());
            },
            PacketType::Audio if self.audio_seq_header.is_none() => {
                self.audio_seq_header = Some(packet.clone());
            }
            _ => ()
        }

        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        log::info!("Closing session");
    }
}


pub type ManagerSender = mpsc::UnboundedSender<ManagerMessage>;
type ManagerReceiver = mpsc::UnboundedReceiver<ManagerMessage>;
type Event = &'static str;

pub enum ManagerMessage {
    CreateSession((String, Responder<Handle>)),
    ReleaseSession(String),
    JoinSession((String, Responder<(Handle, Watcher)>)),
    RegisterTrigger(Event, Trigger),
}

pub struct Manager {
    sender: ManagerSender,
    receiver: ManagerReceiver,
    sessions: Arc<RwLock<HashMap<String, (Handle, OutgoingBroadcast)>>>,
    triggers: Arc<RwLock<HashMap<Event, Vec<Trigger>>>>,
}

impl Manager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let triggers = Arc::new(RwLock::new(HashMap::new()));

        Self { sender, receiver, sessions, triggers }
    }

    pub fn sender(&self) -> ManagerSender {
        self.sender.clone()
    }

    async fn handle_message(&mut self, message: ManagerMessage) {
        match message {
            ManagerMessage::CreateSession((name, responder)) => {
                let (handle, incoming) = mpsc::unbounded_channel();
                let (outgoing, _watcher) = broadcast::channel(64);
                let mut sessions = self.sessions.write().await;
                sessions.insert(name.clone(), (handle.clone(), outgoing.clone()));

                let triggers = self.triggers.read().await;
                if let Some(event_triggers) = triggers.get("create_session") {
                    for trigger in event_triggers {
                        trigger.send((name.clone(), outgoing.subscribe()));
                    }
                }

                tokio::spawn(async move {
                    Session::new(incoming, outgoing).run().await;
                });

                if responder.send(handle).is_err() {
                    log::error!("Failed to create session");
                };
            },
            ManagerMessage::JoinSession((name, responder)) => {
                let sessions = self.sessions.read().await;
                if let Some((handle, watcher)) = sessions.get(&name) {
                    if responder.send((handle.clone(), watcher.subscribe())).is_err() {
                        log::error!("Failed to join session")
                    };
                }
            },
            ManagerMessage::ReleaseSession(name) => {
                let mut sessions = self.sessions.write().await;
                sessions.remove(&name);
            },
            ManagerMessage::RegisterTrigger(event, trigger) => {
                log::debug!("Registering trigger for {}", event);
                let mut triggers = self.triggers.write().await;
                triggers
                    .entry(event)
                    .or_insert_with(Vec::new)
                    .push(trigger);
            },
        }
    }

    pub async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle_message(message).await;
        }
    }
}

use {
    tokio::sync::{mpsc, oneshot, broadcast},
    javelin_types::Packet,
    super::{AppName, StreamKey, Event},
};


pub type Responder<P> = oneshot::Sender<P>;

// session manager
pub enum ManagerMessage {
    CreateSession((AppName, StreamKey, Responder<Handle>)),
    ReleaseSession(AppName),
    JoinSession((AppName, Responder<(Handle, Watcher)>)),
    RegisterTrigger(Event, Trigger),
}

pub type ManagerHandle = mpsc::UnboundedSender<ManagerMessage>;
pub(super) type ManagerReceiver = mpsc::UnboundedReceiver<ManagerMessage>;


pub type Trigger = mpsc::UnboundedSender<(String, Watcher)>;
pub(super) type TriggerHandle = mpsc::UnboundedReceiver<(String, Watcher)>;

pub fn trigger_channel() -> (Trigger, TriggerHandle) {
    mpsc::unbounded_channel()
}


// session instance
pub enum Message {
    Packet(Packet),
    GetInitData(Responder<(Option<Packet>, Option<Packet>, Option<Packet>)>),
    Disconnect,
}

pub type Handle = mpsc::UnboundedSender<Message>;
pub(super) type IncomingBroadcast = mpsc::UnboundedReceiver<Message>;
pub(super) type OutgoingBroadcast = broadcast::Sender<Packet>;
pub type Watcher = broadcast::Receiver<Packet>;

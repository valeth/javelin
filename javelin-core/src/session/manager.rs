use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    anyhow::{Result, bail},
    tokio::sync::{broadcast, mpsc, RwLock},
    super::{
        instance::{Session, Handle, Watcher, OutgoingBroadcast},
        transport::Responder,
    },
};


pub type Trigger = mpsc::UnboundedSender<(String, Watcher)>;
pub type TriggerHandle = mpsc::UnboundedReceiver<(String, Watcher)>;

pub fn trigger_channel() -> (Trigger, TriggerHandle) {
    mpsc::unbounded_channel()
}


pub type ManagerHandle = mpsc::UnboundedSender<ManagerMessage>;
type ManagerReceiver = mpsc::UnboundedReceiver<ManagerMessage>;
type Event = &'static str;


pub enum ManagerMessage {
    CreateSession((String, Responder<Handle>)),
    ReleaseSession(String),
    JoinSession((String, Responder<(Handle, Watcher)>)),
    RegisterTrigger(Event, Trigger),
}


pub struct Manager {
    handle: ManagerHandle,
    incoming: ManagerReceiver,
    sessions: Arc<RwLock<HashMap<String, (Handle, OutgoingBroadcast)>>>,
    triggers: Arc<RwLock<HashMap<Event, Vec<Trigger>>>>,
}

impl Manager {
    pub fn new() -> Self {
        let (handle, incoming) = mpsc::unbounded_channel();
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let triggers = Arc::new(RwLock::new(HashMap::new()));

        Self { handle, incoming, sessions, triggers }
    }

    pub fn handle(&self) -> ManagerHandle {
        self.handle.clone()
    }

    async fn process_message(&mut self, message: ManagerMessage) -> Result<()> {
        match message {
            ManagerMessage::CreateSession((name, responder)) => {
                let (handle, incoming) = mpsc::unbounded_channel();
                let (outgoing, _watcher) = broadcast::channel(64);
                let mut sessions = self.sessions.write().await;
                sessions.insert(name.clone(), (handle.clone(), outgoing.clone()));

                let triggers = self.triggers.read().await;
                if let Some(event_triggers) = triggers.get("create_session") {
                    for trigger in event_triggers {
                        trigger.send((name.clone(), outgoing.subscribe()))?;
                    }
                }

                tokio::spawn(async move {
                    Session::new(incoming, outgoing).run().await;
                });

                if let Err(_) = responder.send(handle) {
                    bail!("Failed to send response");
                }
            },
            ManagerMessage::JoinSession((name, responder)) => {
                let sessions = self.sessions.read().await;
                if let Some((handle, watcher)) = sessions.get(&name) {
                    if let Err(_) = responder.send((handle.clone(), watcher.subscribe())) {
                        bail!("Failed to send response");
                    }
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

        Ok(())
    }

    pub async fn run(mut self) {
        while let Some(message) = self.incoming.recv().await {
            if let Err(err) = self.process_message(message).await {
                log::error!("{}", err);
            };
        }
    }
}

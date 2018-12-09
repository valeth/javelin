use std::{
    collections::HashMap,
    sync::Arc,
};
use parking_lot::{RwLock, Mutex};
use crate::{
    peer::{
        Client,
        Sender,
        media::Channel,
    },
    config::Config,
};


#[derive(Clone)]
pub struct Shared {
    pub config: Arc<RwLock<Config>>,
    pub peers: Arc<RwLock<HashMap<u64, Sender>>>,
    pub clients: Arc<Mutex<HashMap<u64, Client>>>,
    pub streams: Arc<RwLock<HashMap<String, Channel>>>,
    pub app_names: Arc<RwLock<HashMap<String, String>>>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
            app_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn app_name_from_stream_key(&self, stream_key: String) -> Option<String> {
        let app_names = self.app_names.read();
        let app_name = app_names.get(&stream_key)?;
        Some(app_name.to_string())
    }
}

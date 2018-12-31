use std::{
    collections::HashMap,
    sync::Arc,
};
use parking_lot::{RwLock, Mutex};
use crate::{
    media::Channel,
    rtmp::{
        Client,
        Sender,
    },
    hls,
    config::Config,
};


#[derive(Clone)]
pub struct Shared {
    pub config: Arc<RwLock<Config>>,
    pub peers: Arc<RwLock<HashMap<u64, Sender>>>,
    pub clients: Arc<Mutex<HashMap<u64, Client>>>,
    pub streams: Arc<RwLock<HashMap<String, Channel>>>,
    pub app_names: Arc<RwLock<HashMap<String, String>>>,
    pub hls_sender: hls::server::Sender,
}

impl Shared {
    pub fn new(hls_sender: hls::server::Sender) -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
            app_names: Arc::new(RwLock::new(HashMap::new())),
            hls_sender,
        }
    }

    pub fn app_name_from_stream_key(&self, stream_key: &str) -> Option<String> {
        let app_names = self.app_names.read();
        let app_name = app_names.get(stream_key)?;
        Some(app_name.to_string())
    }
}

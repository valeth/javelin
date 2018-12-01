use std::{
    collections::HashMap,
    sync::Arc,
};
use parking_lot::{RwLock, Mutex};
use peer::{Client, Sender};
use peer::media::Channel;


#[derive(Clone)]
pub struct Shared {
    pub peers: Arc<RwLock<HashMap<u64, Sender>>>,
    pub clients: Arc<Mutex<HashMap<u64, Client>>>,
    pub streams: Arc<RwLock<HashMap<String, Channel>>>,
    pub app_names: Arc<RwLock<HashMap<String, String>>>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
            app_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

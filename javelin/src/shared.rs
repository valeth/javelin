use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    parking_lot::{RwLock, Mutex},
    crate::{
        media::Channel,
        rtmp::{
            Client,
            peer,
        },
        config::Config,
    },
};
#[cfg(feature = "hls")]
use crate::hls;


#[derive(Clone)]
pub struct Shared {
    pub config: Arc<RwLock<Config>>,
    pub peers: Arc<RwLock<HashMap<u64, peer::Sender>>>,
    pub clients: Arc<Mutex<HashMap<u64, Client>>>,
    pub streams: Arc<RwLock<HashMap<String, Channel>>>,
    pub app_names: Arc<RwLock<HashMap<String, String>>>,
    #[cfg(feature = "hls")]
    hls_sender: Arc<RwLock<Option<hls::server::Sender>>>,
    #[cfg(feature = "hls")]
    fcleaner_sender: Arc<RwLock<Option<hls::file_cleaner::Sender>>>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
            app_names: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "hls")]
            hls_sender: Arc::new(RwLock::new(None)),
            #[cfg(feature = "hls")]
            fcleaner_sender: Arc::new(RwLock::new(None)),
        }
    }

    #[cfg(feature = "hls")]
    pub fn set_hls_sender(&mut self, sender: hls::server::Sender) {
        let mut hls_sender = self.hls_sender.write();
        *hls_sender = Some(sender);
    }

    #[cfg(feature = "hls")]
    pub fn hls_sender(&self) -> Option<hls::server::Sender> {
        self.hls_sender.read().clone()
    }

    #[cfg(feature = "hls")]
    pub fn set_fcleaner_sender(&mut self, sender: hls::file_cleaner::Sender) {
        let mut fcleaner_sender = self.fcleaner_sender.write();
        *fcleaner_sender = Some(sender);
    }

    #[cfg(feature = "hls")]
    pub fn fcleaner_sender(&self) -> Option<hls::file_cleaner::Sender> {
        self.fcleaner_sender.read().clone()
    }

    pub fn app_name_from_stream_key(&self, stream_key: &str) -> Option<String> {
        let app_names = self.app_names.read();
        let app_name = app_names.get(stream_key)?;
        Some(app_name.to_string())
    }
}

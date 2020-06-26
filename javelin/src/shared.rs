use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    parking_lot::RwLock,
    crate::{
        session,
    },
};
#[cfg(feature = "hls")]
use crate::hls;


#[derive(Clone)]
pub struct Shared {
    pub streams: Arc<RwLock<HashMap<String, session::Session>>>,
    #[cfg(feature = "hls")]
    hls_sender: Arc<RwLock<Option<hls::server::Sender>>>,
    #[cfg(feature = "hls")]
    fcleaner_sender: Arc<RwLock<Option<hls::file_cleaner::Sender>>>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
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
}

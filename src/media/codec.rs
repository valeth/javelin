pub mod avc;
pub mod aac;


use std::sync::Arc;
use parking_lot::RwLock;
use self::{
    avc::dcr::DecoderConfigurationRecord,
    aac::config::AudioSpecificConfiguration,
};


#[derive(Debug, Clone)]
pub struct SharedState {
    pub dcr: Arc<RwLock<Option<DecoderConfigurationRecord>>>,
    pub asc: Arc<RwLock<Option<AudioSpecificConfiguration>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            dcr: Arc::new(RwLock::new(None)),
            asc: Arc::new(RwLock::new(None)),
        }
    }
}

pub mod avc;


use std::sync::Arc;
use parking_lot::RwLock;
use self::{
    avc::dcr::DecoderConfigurationRecord,
};


#[derive(Debug, Clone)]
pub struct SharedState {
    pub dcr: Arc<RwLock<Option<DecoderConfigurationRecord>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            dcr: Arc::new(RwLock::new(None)),
        }
    }
}

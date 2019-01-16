pub(crate) mod utils;
pub mod avc;
pub mod aac;
pub mod error;


use std::sync::Arc;
use parking_lot::RwLock;
pub(crate) use self::{
    avc::dcr::DecoderConfigurationRecord,
    aac::config::AudioSpecificConfiguration,
};
pub use self::error::{Error, Result};


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

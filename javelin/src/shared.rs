use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    parking_lot::RwLock,
    crate::session,
};


#[derive(Clone)]
pub struct Shared {
    pub streams: Arc<RwLock<HashMap<String, session::Session>>>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

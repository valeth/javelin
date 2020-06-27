use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    parking_lot::RwLock,
    crate::Session,
};


#[derive(Clone)]
pub struct Shared {
    pub streams: Arc<RwLock<HashMap<String, Session>>>,
}

impl Shared {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

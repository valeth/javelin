use std::{
    collections::HashSet,
    sync::Arc,
};
use parking_lot::RwLock;


#[derive(Clone)]
pub struct Shared {
    pub peers: Arc<RwLock<HashSet<u64>>>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

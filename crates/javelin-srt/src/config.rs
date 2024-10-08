use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_addr")]
    pub addr: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: default_addr(),
        }
    }
}

fn default_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 3001))
}

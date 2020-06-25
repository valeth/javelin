use {
    std::net::SocketAddr,
    serde::Deserialize,
};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_addr")]
    pub addr: SocketAddr,

    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: default_addr(),
            enabled: default_enabled(),
        }
    }
}

fn default_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 8080))
}

fn default_enabled() -> bool {
    true
}

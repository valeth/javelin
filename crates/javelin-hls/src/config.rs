use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_root_dir")]
    pub root_dir: PathBuf,

    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub web: WebConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_dir: default_root_dir(),
            enabled: default_enabled(),
            web: WebConfig::default(),
        }
    }
}


#[derive(Debug, Clone, Deserialize)]
pub struct WebConfig {
    #[serde(default = "default_web_addr")]
    pub addr: SocketAddr,

    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            addr: default_web_addr(),
            enabled: default_enabled(),
        }
    }
}


fn default_root_dir() -> PathBuf {
    PathBuf::from("./data/hls")
}

fn default_web_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 8080))
}

fn default_enabled() -> bool {
    true
}

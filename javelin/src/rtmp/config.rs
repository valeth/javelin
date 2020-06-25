use {
    std::{
        collections::HashMap,
        path::PathBuf,
        net::SocketAddr,
    },
    anyhow::Result,
    serde::Deserialize,
};

#[cfg(feature = "tls")]
use std::{fs::File, io::Read};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_addr")]
    pub addr: SocketAddr,

    #[serde(default)]
    pub stream_keys: HashMap<String, String>, // TODO: move to database

    #[cfg(feature = "tls")]
    #[serde(default)]
    pub tls: TlsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: default_addr(),
            stream_keys: HashMap::new(),
            tls: Default::default(),
        }
    }
}

fn default_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 1935))
}


#[cfg(feature = "tls")]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub cert_path: Option<PathBuf>,

    #[serde(default)]
    pub cert_password: String,
}

#[cfg(feature = "tls")]
impl TlsConfig {
    pub fn read_cert(&self) -> Result<Vec<u8>> {
        let path = &self.cert_path.clone().expect("No cert path");
        let mut file = File::open(path)?;
        let mut buf = Vec::with_capacity(2500);
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

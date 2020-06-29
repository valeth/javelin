use {
    std::{
        collections::HashMap,
        net::SocketAddr,
        time::Duration,
    },
    serde::Deserialize,
};

#[cfg(feature = "rtmps")]
use std::{
    fs::File,
    anyhow::Result,
    path::PathBuf,
    io::Read
};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_addr")]
    pub addr: SocketAddr,

    #[serde(default)]
    pub stream_keys: HashMap<String, String>, // TODO: move to database

    #[serde(default = "default_conn_timeout")]
    pub connection_timeout: Duration,

    #[cfg(feature = "rtmps")]
    #[serde(default)]
    pub tls: TlsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: default_addr(),
            stream_keys: HashMap::new(),
            connection_timeout: default_conn_timeout(),
            #[cfg(feature = "rtmps")]
            tls: Default::default(),
        }
    }
}

fn default_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 1935))
}

fn default_conn_timeout() -> Duration {
    Duration::from_secs(5)
}


#[cfg(feature = "rtmps")]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub cert_path: Option<PathBuf>,

    #[serde(default)]
    pub cert_password: String,
}

#[cfg(feature = "rtmps")]
impl TlsConfig {
    pub fn read_cert(&self) -> Result<Vec<u8>> {
        let path = &self.cert_path.clone().expect("No cert path");
        let mut file = File::open(path)?;
        let mut buf = Vec::with_capacity(2500);
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

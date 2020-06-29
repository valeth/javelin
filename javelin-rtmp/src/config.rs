use {
    std::{
        collections::HashMap,
        net::SocketAddr,
        time::Duration,
    },
    serde::Deserialize,
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
    pub tls: tls::Config,
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
mod tls {
    use {
        std::{
            fs::File,
            path::PathBuf,
            io::Read,
            net::SocketAddr,
        },
        serde::Deserialize,
        anyhow::{Result, Context},
    };

    #[derive(Debug, Clone, Deserialize)]
    pub struct Config {
        #[serde(default)]
        pub enabled: bool,

        #[serde(default = "default_tls_addr")]
        pub addr: SocketAddr,

        #[serde(default)]
        pub cert_path: Option<PathBuf>,

        #[serde(default)]
        pub cert_password: String,
    }

    impl Config {
        pub fn read_cert(&self) -> Result<Vec<u8>> {
            let path = &self.cert_path.as_ref().context("No cert path configured")?;
            let mut file = File::open(path)?;
            let mut buf = Vec::with_capacity(2500);
            file.read_to_end(&mut buf)?;
            Ok(buf)
        }
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                enabled: false,
                addr: default_tls_addr(),
                cert_path: None,
                cert_password: String::new(),
            }
        }
    }

    fn default_tls_addr() -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], 1936))
    }
}

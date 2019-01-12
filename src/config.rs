use std::{
    collections::HashSet,
    net::SocketAddr,
};
#[cfg(feature = "tls")]
use std::{
    fs::File,
    io::Read,
    env,
};
#[cfg(any(feature = "tls", feature = "hls"))]
use std::path::PathBuf;
#[cfg(any(feature = "tls", feature = "hls"))]
use clap::ArgMatches;
use crate::args;
#[cfg(feature = "tls")]
use crate::error::Result;


#[derive(Debug, Clone)]
#[cfg(feature = "tls")]
pub struct TlsConfig {
    pub cert_path: Option<PathBuf>,
    pub cert_password: String,
    pub enabled: bool,
}

#[cfg(feature = "tls")]
impl TlsConfig {
    pub fn new(args: &ArgMatches) -> Self {
        let enabled = args.is_present("tls_enabled");

        if enabled {
            let cert_path = args.value_of("tls_cert")
                .map(|v| Some(PathBuf::from(v)))
                .unwrap_or(None);
            let cert_password = Self::cert_password();
            Self { cert_path, cert_password, enabled }
        } else {
            Self { cert_path: None, cert_password: "".to_string(), enabled }
        }
    }

    fn cert_password() -> String {
        env::var("JAVELIN_TLS_PASSWORD").expect("Password for TLS certificate required")
    }

    pub fn read_cert(&self) -> Result<Vec<u8>> {
        let path = self.cert_path.clone().expect("");
        let mut file = File::open(path)?;
        let mut buf = Vec::with_capacity(2500);
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}


#[derive(Debug, Clone)]
#[cfg(feature = "hls")]
pub struct HlsConfig {
    pub root_dir: PathBuf,
}

#[cfg(feature = "hls")]
impl HlsConfig {
    pub fn new(args: &ArgMatches) -> Self {
        let root_dir = args.value_of("hls_root")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./tmp/stream"));

        Self { root_dir }
    }
}


#[derive(Debug, Clone)]
pub struct Config {
    pub addr: SocketAddr,
    pub permitted_stream_keys: HashSet<String>,
    #[cfg(feature = "tls")]
    pub tls: TlsConfig,
    #[cfg(feature = "hls")]
    pub hls: HlsConfig,
}

impl Config {
    pub fn new() -> Self {
        let matches = args::build_args();

        let permitted_stream_keys: HashSet<String> = matches
            .values_of("permitted_stream_keys").unwrap_or_default()
            .map(str::to_string)
            .collect();

        let host = matches.value_of("bind").unwrap_or("0.0.0.0");
        let port = matches.value_of("port").unwrap_or("1935");
        let addr = format!("{}:{}", host, port).parse().expect("Invalid address or port name");

        Self {
            addr,
            permitted_stream_keys,
            #[cfg(feature = "tls")]
            tls: TlsConfig::new(&matches),
            #[cfg(feature = "hls")]
            hls: HlsConfig::new(&matches),
        }
    }
}

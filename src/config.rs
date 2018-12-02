use std::net::SocketAddr;
#[cfg(feature = "tls")]
use std::{
    path::PathBuf,
    fs::File,
    io::Read,
    env,
};
#[cfg(feature = "tls")]
use clap::ArgMatches;
#[cfg(feature = "tls")]
use error::Result;
use args;

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
        let enabled = !args.is_present("no_tls");

        if enabled {
            let cert_path = args.value_of("cert")
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
pub struct Config {
    pub addr: SocketAddr,
    #[cfg(feature = "tls")]
    pub tls: TlsConfig,
}

impl Config {
    pub fn new() -> Self {
        let matches = args::build_args();

        let host = matches.value_of("bind").unwrap_or("0.0.0.0");
        let port = matches.value_of("port").unwrap_or("1935");
        let addr = format!("{}:{}", host, port).parse().expect("Invalid address or port name");

        Self {
            addr,
            #[cfg(feature = "tls")]
            tls: TlsConfig::new(&matches),
        }
    }
}

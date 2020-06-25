use {
    std::{
        path::Path,
        fs::File,
    },
    serde::Deserialize,
    serde_yaml as yaml,
    anyhow::Result,
    crate::rtmp::Config as RtmpConfig,
};

#[cfg(feature = "hls")]
use crate::hls::Config as HlsConfig;

#[cfg(feature = "web")]
use crate::web::Config as WebConfig;


#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rtmp: RtmpConfig,

    #[cfg(feature = "hls")]
    #[serde(default)]
    pub hls: HlsConfig,

    #[cfg(feature = "web")]
    #[serde(default)]
    pub web: WebConfig,
}

pub fn load_config<P: AsRef<Path>>(config_dir: P) -> Result<Config> {
    let path = config_dir.as_ref().join("config.yml");
    if path.exists() {
        log::info!("Loading config from {}", path.display());
        let file = File::open(path)?;
        Ok(yaml::from_reader(file)?)
    } else {
        log::warn!("No config file found, loading defaults");
        Ok(Config::default())
    }
}

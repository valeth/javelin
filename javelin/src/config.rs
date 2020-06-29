use {
    std::{
        path::Path,
        fs::File,
    },
    serde::Deserialize,
    serde_yaml as yaml,
    anyhow::Result,
};

#[cfg(feature = "rtmp")]
use javelin_rtmp::Config as RtmpConfig;

#[cfg(feature = "hls")]
use javelin_hls::Config as HlsConfig;


#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[cfg(feature = "rtmp")]
    #[serde(default)]
    pub rtmp: RtmpConfig,

    #[cfg(feature = "hls")]
    #[serde(default)]
    pub hls: HlsConfig,
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

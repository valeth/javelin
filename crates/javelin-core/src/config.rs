use std::path::Path;

use anyhow::Result;
use serde::de::Deserialize;


pub struct Config(config::Config);

impl Config {
    pub fn try_from_path<P>(config_dir: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = config_dir.as_ref().join("javelin.yml");

        let config = config::Config::builder()
            .add_source(config::File::from(path))
            .build()?;

        Ok(Self(config))
    }

    pub fn get<'de, V>(&self, key: &str) -> Result<V>
    where
        V: Deserialize<'de>,
    {
        let value = self.0.get(key)?;
        Ok(value)
    }
}

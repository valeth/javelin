use std::path::Path;

use anyhow::Result;
pub use config::Config;


pub fn from_path<P>(config_dir: P) -> Result<Config>
where
    P: AsRef<Path>,
{
    let path = config_dir.as_ref().join("javelin.yml");

    let config = Config::builder()
        .add_source(config::File::from(path))
        .build()?;

    Ok(config)
}

use {
    std::path::Path,
    anyhow::Result,
};
pub use config::Config;


pub fn from_path<P>(config_dir: P) -> Result<Config>
    where P: AsRef<Path>
{
    let mut config = Config::new();
    let path = config_dir.as_ref().join("config.yml");

    if path.exists() {
        log::info!("Loading config from {}", path.display());
        config.merge(config::File::from(path))?;
    } else {
        log::warn!("No config file found, using defaults");
    }

    Ok(config)
}

use {
    std::path::PathBuf,
    serde::Deserialize,
};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_root_dir")]
    pub root_dir: PathBuf,

    #[serde(default)]
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_dir: default_root_dir(),
            enabled: default_enabled(),
        }
    }
}

fn default_root_dir() -> PathBuf {
    PathBuf::from("./tmp/stream")
}
fn default_enabled() -> bool {
    true
}

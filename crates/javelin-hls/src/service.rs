use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use axum::Router;
use javelin_core::session::{self, ManagerMessage};
use javelin_core::Config;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};

use crate::config::Config as HlsConfig;
use crate::file_cleaner;
use crate::writer::Writer;


pub struct Service {
    config: HlsConfig,
    session_manager: session::ManagerHandle,
}


impl Service {
    pub fn new(session_manager: session::ManagerHandle, config: &Config) -> Self {
        let config = config.get("hls").unwrap_or_default();
        Self {
            config,
            session_manager,
        }
    }

    pub async fn run(self) {
        let hls_root = self.config.root_dir.clone();
        info!("HLS directory located at '{}'", hls_root.display());

        if let Err(why) = directory_cleanup(&hls_root) {
            error!("{}", why);
            return;
        }

        let fcleaner = file_cleaner::FileCleaner::new();
        let fcleaner_sender = fcleaner.sender();
        tokio::spawn(async move { fcleaner.run().await });

        if self.config.web.enabled {
            let addr = self.config.web.addr;

            let serve_dir = ServeDir::new(hls_root);
            let routes = Router::new().nest_service("/hls", serve_dir);

            tokio::spawn(async move {
                let listener = TcpListener::bind(addr).await.unwrap();
                axum::serve(listener, routes).await.unwrap();
            });
        }

        let (trigger, mut trigger_handle) = session::trigger_channel();

        if self
            .session_manager
            .send(ManagerMessage::RegisterTrigger("create_session", trigger))
            .is_err()
        {
            error!("Failed to register session trigger");
            return;
        }

        while let Some((app_name, watcher)) = trigger_handle.recv().await {
            match Writer::create(app_name, watcher, fcleaner_sender.clone(), &self.config) {
                Ok(writer) => {
                    tokio::spawn(async move { writer.run().await.unwrap() });
                }
                Err(why) => error!("Failed to create writer: {:?}", why),
            }
        }
    }
}


fn directory_cleanup<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        debug!("Attempting cleanup of HLS directory");

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let child_path = entry?.path();

                if child_path.is_dir() {
                    fs::remove_dir_all(child_path)?;
                } else {
                    fs::remove_file(child_path)?;
                }
            }
        } else {
            bail!("HLS root is not a directory")
        }

        info!("HLS directory purged");
    }

    Ok(())
}

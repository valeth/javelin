use {
    std::{fs, path::Path},
    anyhow::{Result, bail},
    warp::Filter,
    javelin_core::{
        session::{self, ManagerMessage},
        Config,
    },
    crate::{
        config::Config as HlsConfig,
        file_cleaner,
        writer::Writer,
    },
};


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

    pub async fn run(self)  {
        let hls_root = self.config.root_dir.clone();
        log::info!("HLS directory located at '{}'", hls_root.display());

        if let Err(why) = directory_cleanup(&hls_root) {
            log::error!("{}", why);
            return
        }

        let fcleaner = file_cleaner::FileCleaner::new();
        let fcleaner_sender = fcleaner.sender();
        tokio::spawn(async move {
            fcleaner.run().await
        });

        if self.config.web.enabled {
            let addr = self.config.web.addr;

            let routes = warp::path("hls")
                .and(warp::fs::dir(hls_root));

            tokio::spawn(async move {
                warp::serve(routes).run(addr).await;
            });
        }

        let (trigger, mut trigger_handle) = session::manager::trigger_channel();

        if let Err(_) = self.session_manager.send(ManagerMessage::RegisterTrigger("create_session", trigger)) {
            log::error!("Failed to register session trigger");
            return;
        }

        while let Some((app_name, watcher)) = trigger_handle.recv().await {
            match Writer::create(app_name, watcher, fcleaner_sender.clone(), &self.config) {
                Ok(writer) => {
                    tokio::spawn(async move {
                        writer.run().await.unwrap()
                    });
                },
                Err(why) => log::error!("Failed to create writer: {:?}", why),
            }
        }
    }
}


fn directory_cleanup<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        log::debug!("Attempting cleanup of HLS directory");

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

        log::info!("HLS directory purged");
    }

    Ok(())
}

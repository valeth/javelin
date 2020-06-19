use {
    std::{fs, path::Path},
    tokio::prelude::*,
    futures::{
        try_ready,
        sync::{mpsc, oneshot},
    },
    anyhow::Result,
    super::writer::Writer,
    crate::{media, shared::Shared},
};


type Message = (String, oneshot::Sender<media::Sender>);
type Receiver = mpsc::UnboundedReceiver<Message>;
pub type Sender = mpsc::UnboundedSender<Message>;


pub struct Server {
    sender: Sender,
    receiver: Receiver,
    shared: Shared,
}


impl Server {
    pub fn new(shared: Shared) -> Self {
        let (sender, receiver) = mpsc::unbounded();

        let hls_root = shared.config.read().hls.root_dir.clone();
        log::info!("HLS directory located at '{}'", hls_root.display());

        log::debug!("Attempting cleanup of HLS directory");
        directory_cleanup(hls_root).expect("Failed to clean up HLS directory");
        log::info!("HLS directory purged");

        Self { sender, receiver, shared }
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }
}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some((app_name, request)) = try_ready!(self.receiver.poll()) {
            let (sender, receiver) = mpsc::unbounded();
            request.send(sender).unwrap();

            match Writer::create(app_name, receiver, &self.shared) {
                Ok(writer) => { tokio::spawn(writer); },
                Err(why) => log::error!("Failed to create writer: {:?}", why),
            }
        }

        Ok(Async::Ready(()))
    }
}


fn directory_cleanup<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
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
            panic!("HLS root is not a directory, aborting");
        }
    }

    Ok(())
}

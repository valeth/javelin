use {
    std::{fs, path::Path},
    tokio::prelude::*,
    futures::{
        try_ready,
        sync::{mpsc, oneshot},
    },
    anyhow::{Result, bail},
    super::{file_cleaner, writer::Writer, Config},
    crate::media,
};


type Message = (String, oneshot::Sender<media::Sender>);
type Receiver = mpsc::UnboundedReceiver<Message>;
pub type Sender = mpsc::UnboundedSender<Message>;


enum State {
    Initializing,
    Listening(file_cleaner::Sender),
}


pub struct Service {
    config: Config,
    state: State,
    sender: Sender,
    receiver: Receiver,
}


impl Service {
    pub fn new(config: Config) -> Self {
        let (sender, receiver) = mpsc::unbounded();

        Self {
            config,
            state: State::Initializing,
            sender,
            receiver,
        }
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }

    fn initialize(&mut self) -> Result<()>{
        let hls_root = &self.config.root_dir;
        log::info!("HLS directory located at '{}'", hls_root.display());
        directory_cleanup(hls_root)?;

        let fcleaner = file_cleaner::FileCleaner::new();
        let fcleaner_sender = fcleaner.sender();
        tokio::spawn(fcleaner);
        self.state = State::Listening(fcleaner_sender);

        Ok(())
    }
}

impl Future for Service {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match &self.state {
            State::Initializing => {
                self.initialize().expect("HLS initialization failed");
                task::current().notify(); // poll again ASAP
                return Ok(Async::NotReady);
            },
            State::Listening(fcleaner_sender) => {
                while let Some((app_name, request)) = try_ready!(self.receiver.poll()) {
                    let (sender, receiver) = mpsc::unbounded();
                    request.send(sender).unwrap();

                    match Writer::create(app_name, receiver, fcleaner_sender.clone(), &self.config) {
                        Ok(writer) => { tokio::spawn(writer); },
                        Err(why) => log::error!("Failed to create writer: {:?}", why),
                    }
                }
            }
        }

        Ok(Async::Ready(()))
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

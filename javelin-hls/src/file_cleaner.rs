use {
    std::{
        path::PathBuf,
        fs,
    },
    tokio::{
        stream::StreamExt,
        sync::mpsc,
        time::{DelayQueue, Instant, Duration},
    },
};


type Batch = Vec<PathBuf>;
type Message = (Duration, Batch);
pub type Sender = mpsc::UnboundedSender<Message>;
type Receiver = mpsc::UnboundedReceiver<Message>;


pub struct FileCleaner {
    items: DelayQueue<Batch>,
    sender: Sender,
    receiver: Receiver,
}

impl FileCleaner {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            items: DelayQueue::new(),
            sender,
            receiver,
        }
    }

    pub async fn run(mut self) {
        loop {
            let (duration, files) = self.receiver.recv().await.unwrap();

            let timestamp = Instant::now() + ((duration / 100) * 150);
            log::debug!("{} files queued for cleanup at {:?}", files.len(), timestamp);
            self.items.insert_at(files, timestamp);

            match self.items.next().await {
                Some(Ok(expired)) => remove_files(expired.get_ref()),
                Some(Err(why)) => log::error!("{}", why),
                None => ()
            }
        }
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }
}


fn remove_files(paths: &[PathBuf]) {
    log::debug!("Cleaning up {} files", paths.len());

    for path in paths {
        remove_file(path);
    }
}

fn remove_file(path: &PathBuf) {
    if let Err(why) = fs::remove_file(path) {
        log::error!("Failed to remove file '{}': {}", path.display(), why);
    }
}

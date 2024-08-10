use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use futures_delay_queue::delay_queue;
use futures_intrusive::buffer::GrowingHeapBuf;
use tokio::sync::mpsc;
use tracing::{debug, error};


type Batch = Vec<PathBuf>;
type Message = (Duration, Batch);
pub type Sender = mpsc::UnboundedSender<Message>;
type Receiver = mpsc::UnboundedReceiver<Message>;
type DelayQueue = futures_delay_queue::DelayQueue<Batch, GrowingHeapBuf<Batch>>;
type DelayQueueReceiver = futures_delay_queue::Receiver<Batch>;


pub struct FileCleaner {
    queue: DelayQueue,
    queue_rx: DelayQueueReceiver,
    sender: Sender,
    receiver: Receiver,
}

impl FileCleaner {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let (queue, queue_rx) = delay_queue();

        Self {
            queue,
            queue_rx,
            sender,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some((duration, files)) = self.receiver.recv().await {
            let timestamp = (duration / 100) * 150;
            debug!(
                "{} files queued for cleanup at {:?}",
                files.len(),
                timestamp
            );
            self.queue.insert(files, timestamp);

            match self.queue_rx.receive().await {
                Some(expired) => remove_files(&expired),
                None => (),
            }
        }
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }
}


fn remove_files(paths: &[PathBuf]) {
    debug!("Cleaning up {} files", paths.len());

    for path in paths {
        remove_file(path);
    }
}

fn remove_file(path: &PathBuf) {
    if let Err(why) = fs::remove_file(path) {
        error!("Failed to remove file '{}': {}", path.display(), why);
    }
}

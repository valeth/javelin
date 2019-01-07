use std::{
    path::PathBuf,
    fs,
    time::{Instant, Duration},
};
use log::{debug, error};
use futures::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio::{
    prelude::*,
    timer::DelayQueue,
};
use crate::shared::Shared;


type Batch = Vec<PathBuf>;

type Message = (Duration, Batch);
pub type Sender = UnboundedSender<Message>;
type Receiver = UnboundedReceiver<Message>;


pub struct FileCleaner {
    items: DelayQueue<Batch>,
    receiver: Receiver,
}

impl FileCleaner {
    pub fn new(mut shared: Shared) -> Self {
        let (sender, receiver) = mpsc::unbounded();

        shared.set_fcleaner_sender(sender);

        Self {
            items: DelayQueue::new(),
            receiver,
        }
    }

    fn get_new(&mut self) {
        for _ in 0..10 {
            match self.receiver.poll() {
                Ok(Async::Ready(Some((duration, batch)))) => {
                    let timestamp = Instant::now() + duration;
                    debug!("Cleanup queued at {:?}", timestamp);
                    self.items.insert_at(batch, timestamp);
                },
                _ => break,
            }
        }
    }
}

impl Future for FileCleaner {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.get_new();

        loop {
            match self.items.poll() {
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Ok(Async::Ready(None)) => return Ok(Async::NotReady),
                Ok(Async::Ready(Some(paths))) => remove_files(paths.into_inner()),
                Err(why) => {
                    error!("{:?}", why);
                    return Err(());
                }
            }
        }
    }
}


fn remove_files(paths: Vec<PathBuf>) {
    debug!("Cleaning up {} files", paths.len());

    for path in paths {
        if let Err(why) = fs::remove_file(path) {
            error!("Failed to remove file: {}", why);
        }
    }
}

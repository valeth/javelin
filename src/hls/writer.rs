use log::debug;
use futures::try_ready;
use tokio::prelude::*;
use crate::media;


pub struct Writer {
    receiver: media::Receiver,
}

impl Writer {
    pub fn new(receiver: media::Receiver) -> Self {
        Self { receiver }
    }
}


impl Future for Writer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(media) = try_ready!(self.receiver.poll()) {
            debug!("Received {} bytes of data at {}", media.len(), media.timestamp());
        }

        Ok(Async::Ready(()))
    }
}

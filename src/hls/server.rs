use log::error;
use tokio::prelude::*;
use futures::{
    try_ready,
    sync::{mpsc, oneshot},
};
use crate::{
    media,
    shared::Shared,
};
use super::writer::Writer;


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

            match Writer::create(app_name, receiver, self.shared.clone()) {
                Ok(writer) => { tokio::spawn(writer); },
                Err(why) => error!("Failed to create writer: {:?}", why),
            }
        }

        Ok(Async::Ready(()))
    }
}

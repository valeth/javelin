use tokio::prelude::*;
use futures::sync::{mpsc, oneshot};
use crate::media;
use super::writer::Writer;


type Message = oneshot::Sender<media::Sender>;
type Receiver = mpsc::UnboundedReceiver<Message>;
pub type Sender = mpsc::UnboundedSender<Message>;


pub struct Server {
    sender: Sender,
    receiver: Receiver,
}


impl Server {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }

    /// Returns a future that completes if the channel stream is done.
    pub fn coordinator(self) -> impl Future<Item = (), Error = ()> {
        self.receiver.for_each(|request| {
            let (sender, receiver) = mpsc::unbounded();
            request.send(sender).unwrap();
            let writer = Writer::new(receiver);
            tokio::spawn(writer);
            Ok(())
        })
    }
}

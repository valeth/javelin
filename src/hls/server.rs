use log::error;
use tokio::prelude::*;
use futures::sync::{mpsc, oneshot};
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

    /// Returns a future that completes if the channel stream is done.
    pub fn coordinator(self) -> impl Future<Item = (), Error = ()> {
        let shared = self.shared.clone();

        self.receiver.for_each(move |(app_name, request)| {
            let (sender, receiver) = mpsc::unbounded();
            request.send(sender).unwrap();
            match Writer::create(app_name, receiver, shared.clone()) {
                Ok(writer) => { tokio::spawn(writer); },
                Err(why) => error!("Failed to create writer: {:?}", why),
            }
            Ok(())
        })
    }
}

mod bytes_stream;
mod peer;
mod event;
pub mod client;
pub mod server;


use bytes::Bytes;
use futures::sync::mpsc;
use self::{
    peer::Peer,
    bytes_stream::BytesStream,
};

pub use self::client::Client;
pub use self::server::Server;


pub type Receiver = mpsc::UnboundedReceiver<Bytes>;
pub type Sender = mpsc::UnboundedSender<Bytes>;

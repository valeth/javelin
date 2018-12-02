use std::{
    net::SocketAddr,
    io::ErrorKind as IoErrorKind,
};
use tokio::{
    prelude::*,
    net::TcpListener
};
use error::Error;
use shared::Shared;
use peer::{Peer, BytesStream};


pub struct Server {
    addr: SocketAddr,
    listener: TcpListener,
}

impl Server {
    pub fn new<S: Into<String>>(addr: S) -> Self {
        let addr: SocketAddr = addr.into().parse().expect("Invalid socket address");
        let listener = TcpListener::bind(&addr).expect("Failed to bind TCP listener");
        Self { addr, listener }
    }

    pub fn start(self) {
        let shared = Shared::new();

        let srv = self.listener.incoming()
            .zip(stream::iter_ok(0u64..))
            .map_err(|err| error!("{}", err))
            .for_each(move |(stream, id)| {
                info!("New client connection: {}", id);

                let bytes_stream = BytesStream::new(stream);
                let peer = Peer::new(id, bytes_stream, shared.clone())
                    .map_err(|err| {
                        match err {
                            Error::IoError(ref err) if err.kind() == IoErrorKind::ConnectionReset => (),
                            _ => error!("{:?}", err)
                        }
                    });

                tokio::spawn(peer)
            });

        info!("Starting up Javelin RTMP server on {}", self.addr);

        tokio::run(srv);
    }
}

use std::{
    net::SocketAddr,
};
use tokio::{
    prelude::*,
    net::TcpListener
};
use peer;


pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new<S: Into<String>>(addr: S) -> Self {
        let addr: SocketAddr = addr.into().parse().expect("Invalid socket address");
        let listener = TcpListener::bind(&addr).expect("Failed to bind TCP listener");
        Self { listener }
    }

    pub fn start(self) {
        let srv = self.listener.incoming()
            .zip(stream::iter_ok(0u64..))
            .map_err(|err| error!("{}", err))
            .for_each(move |(stream, id)| {
                info!("New client connection: {}", id);

                let _bytes_stream = peer::BytesStream::new(stream);

                Ok(())
            });

        info!("Starting up Javelin RTMP server");

        tokio::run(srv);
    }
}

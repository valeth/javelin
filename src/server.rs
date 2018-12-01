use std::{
    net::SocketAddr,
};
use tokio::{
    prelude::*,
    net::TcpListener
};


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
            .for_each(move |(_stream, id)| {
                info!("New client connection: {}", id);
                Ok(())
            });

        info!("Starting up Javelin RTMP server");

        tokio::run(srv);
    }
}

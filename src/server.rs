use std::{
    net::SocketAddr,
    io::ErrorKind as IoErrorKind,
};
use log::{info, error};
use tokio::{
    prelude::*,
    net::{TcpListener, TcpStream},
};
#[cfg(feature = "tls")]
use native_tls;
#[cfg(feature = "tls")]
use tokio_tls::TlsAcceptor;
use crate::{
    error::Error,
    shared::Shared,
    peer::{
        Peer,
        BytesStream,
    },
};


pub struct Server {
    shared: Shared,
    addr: SocketAddr,
    listener: TcpListener,
}

impl Server {
    pub fn new() -> Self {
        let shared = Shared::new();
        let addr = shared.config.read().addr;
        let listener = TcpListener::bind(&addr).expect("Failed to bind TCP listener");

        Self { shared, addr, listener }
    }

    pub fn start(self) {
        let shared = self.shared.clone();

        let srv = self.listener.incoming()
            .zip(stream::iter_ok(0u64..))
            .map_err(|err| error!("{}", err))
            .for_each(move |(tcp_stream, id)| {
                spawner(id, tcp_stream, shared.clone());
                Ok(())
            });

        info!("Starting up Javelin RTMP server on {}", self.addr);

        tokio::run(srv);
    }
}

fn process<S>(id: u64, stream: S, shared: Shared)
    where S: AsyncRead + AsyncWrite + Send + 'static
{
    info!("New client connection: {}", id);

    let bytes_stream = BytesStream::new(stream);
    let peer = Peer::new(id, bytes_stream, shared.clone())
        .map_err(|err| {
            match err {
                Error::IoError(ref err) if err.kind() == IoErrorKind::ConnectionReset => (),
                _ => error!("{:?}", err)
            }
        });

    tokio::spawn(peer);
}

#[cfg(not(feature = "tls"))]
fn spawner(id: u64, stream: TcpStream, shared: Shared) {
    process(id, stream, shared);
}

#[cfg(feature = "tls")]
fn spawner(id: u64, stream: TcpStream, shared: Shared) {
    let config = shared.config.read().clone();

    if config.tls.enabled {
        let tls_acceptor = {
            let p12 = config.tls.read_cert().unwrap();
            let password = config.tls.cert_password;
            let cert = native_tls::Identity::from_pkcs12(&p12, &password).unwrap();
            TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap())
        };

        let tls_accept = tls_acceptor.accept(stream)
            .and_then(move |tls_stream| {
                process(id, tls_stream, shared.clone());
                Ok(())
            })
            .map_err(|err| {
                error!("TLS error: {:?}", err);
            });

        tokio::spawn(tls_accept);
    } else {
        process(id, stream, shared);
    }
}

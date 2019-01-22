use std::{
    net::SocketAddr,
    io::ErrorKind as IoErrorKind,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use log::{info, error};
use futures::try_ready;
use tokio::{
    prelude::*,
    net::{TcpListener, TcpStream, tcp::Incoming},
};
#[cfg(feature = "tls")]
use native_tls;
#[cfg(feature = "tls")]
use tokio_tls::TlsAcceptor;
use crate::{
    error::Error,
    shared::Shared,
};

use super::{Peer, BytesStream};


type ClientId = u64;


pub struct Server {
    shared: Shared,
    _addr: SocketAddr,
    listener: Incoming,
    client_id: AtomicUsize,
}

impl Server {
    pub fn new(shared: Shared) -> Self {
        let addr = shared.config.read().addr;
        let listener = TcpListener::bind(&addr).expect("Failed to bind TCP listener");

        info!("Starting up Javelin RTMP server on {}", addr);

        Self {
            shared,
            _addr: addr,
            listener: listener.incoming(),
            client_id: AtomicUsize::default(),
        }
    }

    fn client_id(&self) -> ClientId {
        self.client_id.load(Ordering::SeqCst) as ClientId
    }

    fn increment_client_id(&mut self) {
        self.client_id.fetch_add(1, Ordering::SeqCst);
    }
}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(tcp_stream) = try_ready!(self.listener.poll().map_err(|err| error!("{}", err))) {
            spawner(self.client_id(), tcp_stream, self.shared.clone());
            self.increment_client_id();
        }

        Ok(Async::Ready(()))
    }
}

fn process<S>(id: u64, stream: S, shared: &Shared)
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
    stream.set_keepalive(Some(Duration::from_secs(30)))
        .expect("Failed to set TCP keepalive");

    process(id, stream, &shared);
}

#[cfg(feature = "tls")]
fn spawner(id: u64, stream: TcpStream, shared: Shared) {
    let config = shared.config.read().clone();

    stream.set_keepalive(Some(Duration::from_secs(30)))
        .expect("Failed to set TCP keepalive");

    if config.tls.enabled {
        let tls_acceptor = {
            let p12 = config.tls.read_cert().unwrap();
            let password = config.tls.cert_password;
            let cert = native_tls::Identity::from_pkcs12(&p12, &password).unwrap();
            TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap())
        };

        let tls_accept = tls_acceptor.accept(stream)
            .and_then(move |tls_stream| {
                process(id, tls_stream, &shared);
                Ok(())
            })
            .map_err(|err| {
                error!("TLS error: {:?}", err);
            });

        tokio::spawn(tls_accept);
    } else {
        process(id, stream, &shared);
    }
}

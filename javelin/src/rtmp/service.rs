use {
    std::{
        io::ErrorKind as IoErrorKind,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    },
    futures::try_ready,
    tokio::{
        prelude::*,
        net::{TcpListener, TcpStream, tcp::Incoming},
    },
    javelin_core::{
        session::Trigger as HlsTrigger,
        BytesStream,
        shared::Shared,
    },
    super::{Peer, Error, Config},
};

#[cfg(feature = "tls")]
use {
    native_tls,
    tokio_tls::TlsAcceptor,
};


type ClientId = u64;


pub struct Service {
    shared: Shared,
    listener: Incoming,
    client_id: AtomicUsize,
    config: Config,
    hls_handle: HlsTrigger, // TODO: move this to session when implemented
}

impl Service {
    pub fn new(shared: Shared, hls_handle: HlsTrigger, config: Config) -> Self {
        let addr = &config.addr;
        let listener = TcpListener::bind(addr).expect("Failed to bind TCP listener");

        log::info!("Starting up Javelin RTMP server on {}", addr);

        Self {
            shared,
            config,
            listener: listener.incoming(),
            client_id: AtomicUsize::default(),
            hls_handle,
        }
    }

    fn client_id(&self) -> ClientId {
        self.client_id.load(Ordering::SeqCst) as ClientId
    }

    fn increment_client_id(&mut self) {
        self.client_id.fetch_add(1, Ordering::SeqCst);
    }
}

impl Future for Service {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(tcp_stream) = try_ready!(self.listener.poll().map_err(|err| log::error!("{}", err))) {
            spawner(self.client_id(), tcp_stream, self.shared.clone(), self.hls_handle.clone(), self.config.clone());
            self.increment_client_id();
        }

        Ok(Async::Ready(()))
    }
}

fn process<S>(id: u64, stream: S, shared: &Shared, hls_handle: HlsTrigger, config: Config)
    where S: AsyncRead + AsyncWrite + Send + 'static
{
    log::info!("New client connection: {}", id);

    let bytes_stream = BytesStream::new(stream);
    let peer = Peer::new(id, bytes_stream, shared.clone(), hls_handle, config)
        .map_err(|err| {
            match err {
                Error::Disconnected(e) if e.kind() == IoErrorKind::ConnectionReset => (),
                e => log::error!("{}", e)
            }
        });

    tokio::spawn(peer);
}

#[cfg(not(feature = "tls"))]
fn spawner(id: u64, stream: TcpStream, shared: Shared, hls_handle: HlsTrigger, config: RtmpConfig) {
    stream.set_keepalive(Some(Duration::from_secs(30)))
        .expect("Failed to set TCP keepalive");

    process(id, stream, &shared, hls_handle, config);
}

#[cfg(feature = "tls")]
fn spawner(id: u64, stream: TcpStream, shared: Shared, hls_handle: HlsTrigger, config: Config) {
    stream.set_keepalive(Some(Duration::from_secs(30)))
        .expect("Failed to set TCP keepalive");

    if config.tls.enabled {
        let tls_acceptor = {
            let p12 = config.tls.read_cert().unwrap();
            let password = &config.tls.cert_password;
            let cert = native_tls::Identity::from_pkcs12(&p12, password).unwrap();
            TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap())
        };

        let tls_accept = tls_acceptor.accept(stream)
            .and_then(move |tls_stream| {
                process(id, tls_stream, &shared, hls_handle, config);
                Ok(())
            })
            .map_err(|err| {
                log::error!("TLS error: {:?}", err);
            });

        tokio::spawn(tls_accept);
    } else {
        process(id, stream, &shared, hls_handle, config);
    }
}

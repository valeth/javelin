use {
    std::{
        io::ErrorKind as IoErrorKind,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    },
    tokio::{
        prelude::*,
        net::{TcpListener, TcpStream},
    },
    javelin_core::session,
    super::{Peer, Error, Config},
};

#[cfg(feature = "rtmps")]
use {
    native_tls,
    tokio_tls::TlsAcceptor,
};


type ClientId = u64;


pub struct Service {
    client_id: AtomicUsize,
    config: Config,
    session_manager: session::ManagerHandle,
}

impl Service {
    pub fn new(session_manager: session::ManagerHandle, config: Config) -> Self {
        Self {
            config,
            client_id: AtomicUsize::default(),
            session_manager,
        }
    }

    pub async fn run(mut self) {
        let addr = &self.config.addr;
        log::info!("Starting up Javelin RTMP server on {}", addr);

        let mut listener = TcpListener::bind(addr).await.expect("Failed to bind TCP listener");

        loop {
            match listener.accept().await {
                Ok((tcp_stream, _addr)) => {
                    spawner(self.client_id(), tcp_stream, self.session_manager.clone(), self.config.clone());
                    self.increment_client_id();
                },
                Err(why) => log::error!("{}", why),
            }
        }
    }

    fn client_id(&self) -> ClientId {
        self.client_id.load(Ordering::SeqCst) as ClientId
    }

    fn increment_client_id(&mut self) {
        self.client_id.fetch_add(1, Ordering::SeqCst);
    }
}


fn process<S>(id: u64, stream: S, session_manager: session::ManagerHandle, config: Config)
    where S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    log::info!("New client connection: {}", id);
    let peer = Peer::new(id, stream, session_manager, config);

    tokio::spawn(async move {
        if let Err(err) = peer.run().await {
            match err {
                Error::Disconnected(e) if e.kind() == IoErrorKind::ConnectionReset => (),
                e => log::error!("{}", e)
            }
        }
    });
}

#[cfg(not(feature = "rtmps"))]
fn spawner(id: u64, stream: TcpStream, session_manager: session::ManagerHandle, config: Config) {
    stream.set_keepalive(Some(Duration::from_secs(30)))
        .expect("Failed to set TCP keepalive");

    process(id, stream, session_manager, config);
}

#[cfg(feature = "rtmps")]
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

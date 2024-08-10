use {
    std::{
        io::ErrorKind as IoErrorKind,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
        fmt::{self, Display},
    },
    anyhow::Result,
    tokio::{
        prelude::*,
        net::TcpListener,
    },
    javelin_core::{session, Config},
    crate::{
        config::Config as RtmpConfig,
        peer::Peer,
        Error,
    },
};

#[cfg(feature = "rtmps")]
use {native_tls, tokio_native_tls::TlsAcceptor};


#[derive(Debug, Default)]
pub(crate) struct ClientId {
    value: AtomicUsize
}

impl ClientId {
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }
}

impl Display for ClientId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.value.load(Ordering::SeqCst))
    }
}

impl From<&ClientId> for u64 {
    fn from(id: &ClientId) -> Self {
        id.value.load(Ordering::SeqCst) as u64
    }
}


pub struct Service {
    config: RtmpConfig,
    session_manager: session::ManagerHandle,
    client_id: ClientId,
}

impl Service {
    pub fn new(session_manager: session::ManagerHandle, config: &Config) -> Self {
        Self {
            session_manager,
            config: config.get("rtmp").unwrap_or_default(),
            client_id: ClientId::default()
        }
    }

    pub async fn run(self) {
        #[cfg(not(feature = "rtmps"))]
        let res = self.handle_rtmp().await;
        #[cfg(feature = "rtmps")]
        let res = tokio::try_join!(
            self.handle_rtmp(),
            self.handle_rtmps()
        );

        if let Err(err) = res {
            log::error!("{}", err);
        }
    }

    async fn handle_rtmp(&self) -> Result<()> {
        let addr = &self.config.addr;
        let mut listener = TcpListener::bind(addr).await?;
        log::info!("Listening for RTMP connections on {}", addr);

        loop {
            let (tcp_stream, _addr) = listener.accept().await?;
            tcp_stream.set_keepalive(Some(Duration::from_secs(30)))?;
            self.process(tcp_stream);
            self.client_id.increment();
        }
    }

    #[cfg(feature = "rtmps")]
    async fn handle_rtmps(&self) -> Result<()> {
        if !self.config.tls.enabled {
            return Ok(())
        }

        let addr = &self.config.tls.addr;
        let mut listener = TcpListener::bind(addr).await?;
        log::info!("Listening for RTMPS connections on {}", addr);

        let tls_acceptor = {
            let p12 = self.config.tls.read_cert()?;
            let password = &self.config.tls.cert_password;
            let cert = native_tls::Identity::from_pkcs12(&p12, password)?;
            TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build()?)
        };

        loop {
            let (tcp_stream, _addr) = listener.accept().await?;
            tcp_stream.set_keepalive(Some(Duration::from_secs(30)))?;
            let tls_stream = tls_acceptor.accept(tcp_stream).await?;
            self.process(tls_stream);
        }
    }

    fn process<S>(&self, stream: S)
        where S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        log::info!("New client connection: {}", &self.client_id);
        let id = (&self.client_id).into();
        let peer = Peer::new(id, stream, self.session_manager.clone(), self.config.clone());

        tokio::spawn(async move {
            if let Err(err) = peer.run().await {
                match err {
                    Error::Disconnected(e) if e.kind() == IoErrorKind::ConnectionReset => (),
                    e => log::error!("{}", e)
                }
            }
        });
    }
}

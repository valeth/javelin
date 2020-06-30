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
}

impl Service {
    pub fn new(session_manager: session::ManagerHandle, config: &Config) -> Self {
        let config = config.get("rtmp").unwrap_or_default();
        Self {
            config,
            session_manager,
        }
    }

    pub async fn run(self) {
        let client_id = ClientId::default();

        #[cfg(not(feature = "rtmps"))]
        let res = handle_rtmp(&client_id, &self.session_manager, &self.config).await;
        #[cfg(feature = "rtmps")]
        let res = tokio::try_join!(
            handle_rtmp(&client_id, &self.session_manager, &self.config),
            handle_rtmps(&client_id, &self.session_manager, &self.config)
        );

        if let Err(err) = res {
            log::error!("{}", err);
        }
    }
}


async fn handle_rtmp(client_id: &ClientId, session_manager: &session::ManagerHandle, config: &RtmpConfig) -> Result<()> {
    let addr = &config.addr;
    let mut listener = TcpListener::bind(addr).await?;
    log::info!("Listening for RTMP connections on {}", addr);

    loop {
        let (tcp_stream, _addr) = listener.accept().await?;
        tcp_stream.set_keepalive(Some(Duration::from_secs(30)))?;
        process(client_id, tcp_stream, &session_manager, &config);
        client_id.increment();
    }
}

pub(crate) fn process<S>(id: &ClientId, stream: S, session_manager: &session::ManagerHandle, config: &RtmpConfig)
    where S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    log::info!("New client connection: {}", id);
    let id = id.into();
    let peer = Peer::new(id, stream, session_manager.clone(), config.clone());

    tokio::spawn(async move {
        if let Err(err) = peer.run().await {
            match err {
                Error::Disconnected(e) if e.kind() == IoErrorKind::ConnectionReset => (),
                e => log::error!("{}", e)
            }
        }
    });
}

#[cfg(feature = "rtmps")]
pub(crate) async fn handle_rtmps(client_id: &ClientId, session_manager: &session::ManagerHandle, config: &RtmpConfig) -> Result<()> {
    if !config.tls.enabled {
        return Ok(())
    }

    let addr = &config.tls.addr;
    let mut listener = TcpListener::bind(addr).await?;
    log::info!("Listening for RTMPS connections on {}", addr);

    let tls_acceptor = {
        let p12 = config.tls.read_cert()?;
        let password = &config.tls.cert_password;
        let cert = native_tls::Identity::from_pkcs12(&p12, password)?;
        TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build()?)
    };

    loop {
        let (tcp_stream, _addr) = listener.accept().await?;
        tcp_stream.set_keepalive(Some(Duration::from_secs(30)))?;
        let tls_stream = tls_acceptor.accept(tcp_stream).await?;
        process(client_id, tls_stream, &session_manager, &config);
    }
}

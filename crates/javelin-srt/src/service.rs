use std::borrow::Cow;

use base64::engine::general_purpose::URL_SAFE as BASE64_URL_SAFE;
use base64::Engine;
use futures::StreamExt;
use javelin_core::session::ManagerMessage;
use javelin_core::{session, Config};
use srt_tokio::access::{
    AccessControlList, ConnectionMode, ServerRejectReason, StandardAccessControlEntry,
};
use srt_tokio::options::StreamId;
use srt_tokio::{ConnectionRequest, SrtListener};
use tokio::sync::oneshot;
use tracing::{error, info, trace};

use crate::config::Config as SrtConfig;
use crate::peer::{handle_peer, Peer};
use crate::Error;


#[derive(Debug)]
pub struct Service {
    session_manager: session::ManagerHandle,
    config: SrtConfig,
}

impl Service {
    pub fn new(session_manager: session::ManagerHandle, config: &Config) -> Self {
        Service {
            session_manager,
            config: config.get("srt").unwrap_or_default(),
        }
    }

    pub async fn run(self) {
        let addr = self.config.addr;

        let (_listener, mut conn) = SrtListener::builder().bind(addr.clone()).await.unwrap();

        info!("Listening for SRT connections on {}", &addr);

        while let Some(conn_req) = conn.incoming().next().await {
            let session_manager = self.session_manager.clone();

            tokio::spawn(async move {
                if let Err(err) = handle_request(session_manager, conn_req).await {
                    error!(%err);
                }
            });
        }
    }
}


fn decode_base64_stream_id(stream_id: &StreamId) -> Result<String, Error> {
    trace!("attempting to decode streamid as base64");

    let stream_id = stream_id.as_str();

    let decoded_sid =
        BASE64_URL_SAFE
            .decode(stream_id)
            .map_err(|_| Error::StreamIdDecodeFailed {
                stream_id: stream_id.to_string(),
            })?;

    trace!(decoded_sid.len = %decoded_sid.len());

    String::from_utf8(decoded_sid).map_err(|_| Error::StreamIdDecodeFailed {
        stream_id: stream_id.to_string(),
    })
}


fn parse_stream_id(stream_id: &StreamId) -> Result<AccessControlList, Error> {
    let stream_id = if stream_id.starts_with("#!") {
        trace!("got plain access control list");
        Cow::from(stream_id.as_str())
    } else {
        decode_base64_stream_id(stream_id)?.into()
    };

    trace!(?stream_id);

    let acl = stream_id
        .parse::<AccessControlList>()
        .map_err(|_| Error::InvalidAccessControlParams)?;

    trace!(?acl);

    Ok(acl)
}


#[tracing::instrument(skip_all)]
async fn authorize(
    session_handle: session::ManagerHandle,
    stream_id: Option<&StreamId>,
) -> Result<Peer, Error> {
    let stream_id = stream_id.ok_or(Error::StreamIdMissing)?;

    let acl = parse_stream_id(stream_id)?;
    let acl = acl.0.into_iter().map(StandardAccessControlEntry::try_from);

    let mut user_ident = None;
    let mut res_name = None;
    let mut conn_mode = None;

    for entry in acl {
        let entry = entry.map_err(|_| Error::InvalidAccessControlParams)?;

        match entry {
            StandardAccessControlEntry::UserName(uname) => {
                user_ident = Some(uname);
            }
            StandardAccessControlEntry::ResourceName(rname) => {
                res_name = Some(rname);
            }
            StandardAccessControlEntry::Mode(mode) => {
                conn_mode = Some(mode);
            }
            _ => (),
        }
    }

    let (user_ident, res_name, conn_mode) = match (user_ident, res_name, conn_mode) {
        (Some(u), Some(r), Some(m)) => (u, r, m),
        (Some(u), Some(r), None) => (u, r, ConnectionMode::Request),
        _ => return Err(Error::MissingAccessControlParams),
    };

    let peer = match conn_mode {
        ConnectionMode::Publish => {
            let (tx, rx) = oneshot::channel();

            let message =
                ManagerMessage::CreateSession((res_name.to_string(), user_ident.to_string(), tx));

            session_handle
                .send(message)
                .map_err(|_| Error::Unauthorized)?;

            let session_tx = rx.await.map_err(|_| Error::Unauthorized)?;

            Peer::new_publishing(session_tx)
        }
        ConnectionMode::Request => {
            let (tx, rx) = oneshot::channel();

            let message = ManagerMessage::JoinSession((res_name.to_string(), tx));

            session_handle
                .send(message)
                .map_err(|_| Error::Unauthorized)?;

            let (session_tx, session_rx) = rx.await.map_err(|_| Error::Unauthorized)?;

            Peer::new_receiving(session_tx, session_rx)
        }
        _ => return Err(Error::ModeNotSupported),
    };

    Ok(peer)
}


async fn handle_request(
    session_handle: session::ManagerHandle,
    conn_req: ConnectionRequest,
) -> Result<(), Error> {
    let stream_id = conn_req.stream_id();

    match authorize(session_handle, stream_id).await {
        Ok(peer) => {
            trace!("Accepting request");
            let sock = conn_req.accept(None).await?;
            tokio::spawn(async move { handle_peer(peer, sock).await });
        }
        Err(err) => {
            reject_request(conn_req, err).await?;
        }
    };

    Ok(())
}

async fn reject_request(conn_req: ConnectionRequest, error: Error) -> Result<(), Error> {
    trace!(%error, "Rejecting request");

    let reason = match error {
        Error::StreamIdDecodeFailed { stream_id: _ }
        | Error::StreamIdMissing
        | Error::InvalidAccessControlParams
        | Error::MissingAccessControlParams => ServerRejectReason::BadRequest,
        Error::ModeNotSupported => ServerRejectReason::BadMode,
        Error::Unauthorized => ServerRejectReason::Unauthorized,
        Error::Io(_) => ServerRejectReason::InternalServerError,
    };

    conn_req.reject(reason.into()).await?;

    Ok(())
}

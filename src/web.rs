use std::{
    thread,
    net::SocketAddr,
    error::Error as StdError,
    fmt::{self, Display},
};
use serde_json::json;
use warp::{
    self,
    Filter,
    Rejection,
    Reply,
    http::StatusCode,
    filters::BoxedFilter,
};
use crate::Shared;


#[derive(Clone, Debug)]
enum ApiError {
    NoSuchResource,
}

impl StdError for ApiError {
    fn description(&self) -> &str {
        match *self {
            ApiError::NoSuchResource => "No such resource"
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}


pub struct WebServer {
    shared: Shared,
}

impl WebServer {
    pub fn new(shared: Shared) -> Self {
        Self { shared }
    }

    pub fn start(&mut self) {
        let shared = self.shared.clone();
        thread::spawn(|| server(shared));
    }
}


fn server(shared: Shared) {
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    let hls_root = {
        let config = shared.config.read();
        config.hls.root_dir.clone()
    };

    let hls_files = warp::path("hls")
        .and(warp::fs::dir(hls_root));

    let streams_api = warp::path("api")
        .and(api(shared.clone()));

    let routes = hls_files
        .or(streams_api)
        .recover(error_handler);

    warp::serve(routes).run(addr);
}

fn active_streams(shared: &Shared) -> Vec<String> {
    let streams = shared.streams.read();
    streams.iter()
        .filter_map(|(k, v)| {
            if v.has_publisher() {
                Some(k.clone())
            } else {
                None
            }
        })
        .collect()
}

fn api(shared: Shared) -> BoxedFilter<(impl Reply,)> {
    warp::path("active-streams")
        .map(move || {
            let json = json!({
                "streams": active_streams(&shared)
            });
            warp::reply::json(&json)
        })
        .or_else(|_err| {
            Err(warp::reject::custom(ApiError::NoSuchResource))
        })
        .boxed()
}

fn error_handler(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.find_cause::<ApiError>() {
        Some(e @ ApiError::NoSuchResource) => {
            let code = StatusCode::NOT_FOUND;
            let json = json!({
                "code": code.as_u16(),
                "error": e.description()
            });
            let reply = warp::reply::json(&json);
            Ok(warp::reply::with_status(reply, code))
        },
        None => Err(err)
    }
}

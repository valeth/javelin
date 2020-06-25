use {
    std::thread,
    warp::{Filter, Reply, Rejection, http::StatusCode},
    serde_json::json,
    super::api::{
        api,
        Error as ApiError,
    },
    crate::{
        shared::Shared,
        config::Config,
    },
};


macro_rules! json_error_response {
    ($code:expr, $message:expr) => {{
        let json = json!({ "error": $message });
        let reply = warp::reply::json(&json);
        Ok(warp::reply::with_status(reply, $code))
    }};
}


pub struct Server {
    shared: Shared,
    config: Config,
}

impl Server {
    pub fn new(shared: Shared, config: Config) -> Self {
        Self { shared, config }
    }

    pub fn start(&mut self) {
        let shared = self.shared.clone();
        let config = self.config.clone();
        thread::spawn(|| server(shared, config));
    }
}


fn server(shared: Shared, config: Config) {
    let addr = config.web.addr;
    let hls_root = config.hls.root_dir;

    let hls_files = warp::path("hls")
        .and(warp::fs::dir(hls_root));

    let streams_api = warp::path("api")
        .and(api(shared));

    let routes = hls_files
        .or(streams_api)
        .recover(error_handler);

    warp::serve(routes).run(addr);
}

fn error_handler(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.find_cause() {
        | Some(e @ ApiError::NoSuchResource)
        | Some(e @ ApiError::StreamNotFound) => {
            json_error_response!(StatusCode::NOT_FOUND, e.to_string())
        },
        None => Err(err)
    }
}

use {
    std::thread,
    warp::{
        Filter,
        Reply,
        Rejection,
        http::StatusCode,
    },
    serde_json::json,
    super::api::{
        api,
        Error as ApiError,
    },
    crate::Shared,
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
}

impl Server {
    pub fn new(shared: Shared) -> Self {
        Self { shared }
    }

    pub fn start(&mut self) {
        let shared = self.shared.clone();
        thread::spawn(|| server(shared));
    }
}


fn server(shared: Shared) {
    let addr = {
        let config = shared.config.read();
        config.web.addr
    };

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

fn error_handler(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.find_cause() {
        | Some(e @ ApiError::NoSuchResource)
        | Some(e @ ApiError::StreamNotFound) => {
            json_error_response!(StatusCode::NOT_FOUND, e.to_string())
        },
        None => Err(err)
    }
}

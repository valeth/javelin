use std::{
    thread,
    net::SocketAddr,
};
use serde_json::json;
use warp::{
    self,
    Filter,
};
use crate::Shared;


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


#[allow(clippy::needless_pass_by_value)] // shut up
fn server(shared: Shared) {
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    let hls_root = {
        let config = shared.config.read();
        config.hls.root_dir.clone()
    };

    let hls_files = warp::path("hls").and(warp::fs::dir(hls_root));

    let streams_api = warp::path("api").and(warp::path::param())
        .map(move |resource: String| {
            let json = match resource.as_str() {
                "active_streams" => {
                    json!({ "streams": active_streams(&shared) })
                }
                _ => {
                    json!({ "error": "Unknown resource" })
                }
            };

            warp::reply::json(&json)
        });

    let routes = hls_files.or(streams_api);

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

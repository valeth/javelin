use std::{
    thread,
    net::SocketAddr,
};
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

    warp::serve(hls_files).run(addr);
}

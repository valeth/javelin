use std::{
    thread,
    net::SocketAddr,
    fs::File,
    io::Read,
};
use log::error;
use warp::{
    self,
    path,
    Filter,
    http::{Response, StatusCode}
};
use crate::{Shared, Result};


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


// TODO: routes for TS files
fn server(shared: Shared) {
    let hls_path = warp::path!("hls" / String)
        .map(move |app_name| {
            let mut builder = Response::builder();

            let body = match hls_playlist(app_name, &shared) {
                Ok(content) => {
                    builder
                        .header("content-type", "application/vnd.apple.mpegurl")
                        .status(StatusCode::OK);
                    content
                },
                Err(why) => {
                    builder.status(StatusCode::NOT_FOUND);
                    error!("{:?}", why);
                    String::new()
                }
            };

            builder
                .header("access-control-allow-origin", "*")
                .body(body)
        });

    let routes = warp::get2().and(hls_path);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    warp::serve(routes).run(addr);
}

fn hls_playlist(app_name: String, shared: &Shared) -> Result<String> {
    let config = shared.config.read();
    let playlist = config.hls.root_dir
        .join(app_name)
        .join("playlist.m3u8");

    let mut file = File::open(playlist)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

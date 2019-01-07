#![warn(clippy::all)]

mod error;
mod utils;
mod shared;
mod config;
mod media;
mod rtmp;
mod args;

#[cfg(feature = "hls")]
mod hls;


use futures::future::lazy;
use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};

#[allow(unused_imports)]
use self::{
    shared::Shared,
    error::Error,
};


macro_rules! init_logger {
    [ $kind:ident ] => { $kind::init(LevelFilter::Debug, Config::default()) }
}


fn main() {
    init_logger!(TermLogger).unwrap_or_else(|_|
        init_logger!(SimpleLogger).unwrap_or_else(|err|
            eprintln!("Failed to initialize logger: {}", err)));

    tokio::run(lazy(|| {
        let shared = Shared::new();

        #[cfg(feature = "hls")]
        spawn_hls_server(shared.clone());

        tokio::spawn(rtmp::Server::new(shared.clone()).start());

        Ok(())
    }));
}

#[cfg(feature = "hls")]
fn spawn_hls_server(mut shared: Shared) {
    let hls_server = hls::Server::new(shared.clone());
    let hls_sender = hls_server.sender();
    let file_cleaner = hls::file_cleaner::FileCleaner::new(shared.clone());
    shared.set_hls_sender(hls_sender);
    tokio::spawn(hls_server);
    tokio::spawn(file_cleaner);
}

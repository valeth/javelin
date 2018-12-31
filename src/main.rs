#![warn(clippy::all)]

mod error;
mod shared;
mod config;
mod media;
mod rtmp;
mod hls;
mod args;


use futures::future::lazy;
use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};
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
        let hls_server = hls::Server::new();
        let hls_sender = hls_server.sender();
        tokio::spawn(hls_server.coordinator());

        let shared = Shared::new(hls_sender);

        tokio::spawn(rtmp::Server::new(shared.clone()).start());

        Ok(())
    }));
}

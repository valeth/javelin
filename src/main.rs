extern crate bytes;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate parking_lot;
#[macro_use] extern crate futures;
extern crate tokio;
extern crate rml_rtmp as rtmp;
#[macro_use] extern crate clap;

#[cfg(feature = "tls")]
extern crate native_tls;
#[cfg(feature = "tls")]
extern crate tokio_tls;


mod error;
mod shared;
mod config;
mod peer;
mod server;
mod args;


use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};
use server::Server;


macro_rules! init_logger {
    [ $kind:ident ] => { $kind::init(LevelFilter::Debug, Config::default()) }
}


fn main() {
    init_logger!(TermLogger).unwrap_or_else(|_|
        init_logger!(SimpleLogger).unwrap_or_else(|err|
            eprintln!("Failed to initialize logger: {}", err)));

    Server::new().start();
}

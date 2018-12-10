#![warn(clippy::all)]

mod error;
mod shared;
mod config;
mod peer;
mod server;
mod args;


use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};
use crate::server::Server;


macro_rules! init_logger {
    [ $kind:ident ] => { $kind::init(LevelFilter::Debug, Config::default()) }
}


fn main() {
    init_logger!(TermLogger).unwrap_or_else(|_|
        init_logger!(SimpleLogger).unwrap_or_else(|err|
            eprintln!("Failed to initialize logger: {}", err)));

    Server::new().start();
}

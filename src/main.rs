extern crate bytes;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate parking_lot;
#[macro_use] extern crate futures;
extern crate tokio;
extern crate rml_rtmp as rtmp;
extern crate clap;


mod error;
mod shared;
mod peer;
mod server;
mod args;


use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};
use server::Server;


macro_rules! init_logger {
    [ $kind:ident ] => { $kind::init(LevelFilter::Debug, Config::default()) }
}


fn main() {
    let matches = args::build_args();

    init_logger!(TermLogger).unwrap_or_else(|_|
        init_logger!(SimpleLogger).unwrap_or_else(|err|
            eprintln!("Failed to initialize logger: {}", err)));

    let host = matches.value_of("bind").unwrap_or("0.0.0.0");
    let port = matches.value_of("port").unwrap_or("1935");

    Server::new(format!("{}:{}", host, port)).start();
}

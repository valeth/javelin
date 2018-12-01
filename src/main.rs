extern crate bytes;
#[macro_use] extern crate log;
extern crate simplelog;
#[macro_use] extern crate futures;
extern crate tokio;
extern crate rml_rtmp as rtmp;


mod error;
mod peer;
mod server;


use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};
use server::Server;


macro_rules! init_logger {
    [ $kind:ident ] => { $kind::init(LevelFilter::Debug, Config::default()) }
}


fn main() {
    init_logger!(TermLogger).unwrap_or_else(|_|
        init_logger!(SimpleLogger).unwrap_or_else(|err|
            eprintln!("Failed to initialize logger: {}", err)));

    Server::new("0.0.0.0:1935").start();
}

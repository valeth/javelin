#![allow(unused_imports)] /// TODO: remove this

extern crate bytes;
#[macro_use] extern crate log;
extern crate simplelog;
#[macro_use] extern crate futures;
extern crate tokio;
extern crate rml_rtmp as rtmp;


use simplelog::{Config, SimpleLogger, TermLogger, LevelFilter};


macro_rules! init_logger {
    [ $kind:ident ] => { $kind::init(LevelFilter::Debug, Config::default()) }
}


fn main() {
    init_logger!(TermLogger).unwrap_or_else(|_|
        init_logger!(SimpleLogger).unwrap_or_else(|err|
            eprintln!("Failed to initialize logger: {}", err)));

    debug!("Hello, world!");
}

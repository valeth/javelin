#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

mod error;
mod shared;
mod config;
mod media;
mod rtmp;
mod args;

#[cfg(feature = "hls")]
mod hls;

#[cfg(feature = "web")]
mod web;


use {
    futures::future::lazy,
    self::{
        shared::Shared,
        error::{Error, Result},
    },
};


fn main() {
    if let Err(why) = init_logger() {
        eprintln!("Failed to initialize logger: {}", why);
    };

    let shared = Shared::new();

    #[cfg(feature = "web")]
    spawn_web_server(shared.clone());

    tokio::run(lazy(move || {
        #[cfg(feature = "hls")]
        spawn_hls_server(shared.clone());

        tokio::spawn(rtmp::Server::new(shared.clone()));

        Ok(())
    }));
}

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    use {
        fern::{Dispatch, colors::ColoredLevelConfig, log_file},
        log::LevelFilter,
        chrono::{Utc, Local as LocalTime},
    };

    let colors = ColoredLevelConfig::default();
    Dispatch::new()
        .level(LevelFilter::Error)
        .level_for("javelin", LevelFilter::Warn)
        .level_for("javelin::rtmp", LevelFilter::Debug)
        .level_for("javelin-codec", LevelFilter::Warn)
        .chain(Dispatch::new()
            .format(|out, msg, record| {
                out.finish(format_args!(
                    "level={:5} timestamp={} target={}  {}",
                    record.level(),
                    Utc::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.target(),
                    msg
                ))
            })
            // TODO: implement auto rotating file logger
            .chain(log_file("javelin.log")?)
        )
        .chain(Dispatch::new()
            .format(move |out, msg, record| {
                out.finish(format_args!(
                    "[{:5}] {} ({}) {}",
                    colors.color(record.level()),
                    LocalTime::now().format("%Y-%m-%d %H:%M:%S"),
                    record.target(),
                    msg
                ))
            })
            .chain(std::io::stdout())
        )
        .apply()?;

    Ok(())
}

#[cfg(feature = "hls")]
fn spawn_hls_server(mut shared: Shared) {
    let enabled = {
        let config = shared.config.read();
        config.hls.enabled
    };

    if enabled {
        let hls_server = hls::Server::new(shared.clone());
        let hls_sender = hls_server.sender();
        let file_cleaner = hls::file_cleaner::FileCleaner::new(shared.clone());
        shared.set_hls_sender(hls_sender);
        tokio::spawn(hls_server);
        tokio::spawn(file_cleaner);
    }
}

#[cfg(feature = "web")]
fn spawn_web_server(shared: Shared) {
    let enabled = {
        let config = shared.config.read();
        config.hls.enabled && config.web.enabled
    };

    if enabled {
        web::Server::new(shared).start();
    }
}

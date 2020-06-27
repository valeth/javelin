#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

mod bytes_stream;
mod shared;
mod config;
mod rtmp;
mod args;
mod session;

#[cfg(feature = "hls")]
mod hls;

#[cfg(feature = "web")]
mod web;


use {
    futures::future::lazy,
    anyhow::Result,
    bytes_stream::BytesStream,
    self::{
        config::{load_config, Config},
        shared::Shared,
    },
};


fn main() -> Result<()> {
    if let Err(why) = init_logger() {
        eprintln!("Failed to initialize logger: {}", why);
    };

    let args = args::build();
    let config_dir = args.value_of("config_dir").unwrap_or_default();

    let config = load_config(config_dir)?;
    let shared = Shared::new();

    #[cfg(feature = "web")]
    spawn_web_server(shared.clone(), config.clone());

    tokio::run(lazy(move || {
        #[cfg(feature = "hls")]
        let hls_service = hls::Service::new(config.hls.clone());
        let hls_handle = hls_service.trigger_handle();
        tokio::spawn(hls_service);

        // TODO: remove handle from RTMP and move to session
        tokio::spawn(rtmp::Service::new(shared.clone(), hls_handle, config.rtmp.clone()));

        Ok(())
    }));

    Ok(())
}

fn init_logger() -> Result<()> {
    use {
        fern::{Dispatch, colors::ColoredLevelConfig, log_file},
        log::LevelFilter,
        chrono::{Utc, Local as LocalTime},
    };

    let colors = ColoredLevelConfig::default();
    Dispatch::new()
        .level(LevelFilter::Error)
        .level_for("javelin", LevelFilter::Debug)
        .level_for("javelin::rtmp", LevelFilter::Debug)
        .level_for("javelin-rtmp", LevelFilter::Debug)
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

#[cfg(feature = "web")]
fn spawn_web_server(shared: Shared, config: Config) {
    let enabled = config.hls.enabled && config.web.enabled;

    if enabled {
        web::Server::new(shared, config).start();
    }
}

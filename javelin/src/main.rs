#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

mod config;
mod args;

#[cfg(feature = "hls")]
mod hls;


use {
    anyhow::Result,
    javelin_core::shared::Shared,
    self::config::load_config,
};


#[tokio::main]
async fn main() -> Result<()> {
    if let Err(why) = init_logger() {
        eprintln!("Failed to initialize logger: {}", why);
    };

    let args = args::build();
    let config_dir = args.value_of("config_dir").unwrap_or_default();

    let config = load_config(config_dir)?;
    let shared = Shared::new();

    #[cfg(feature = "hls")]
    let hls_service = hls::Service::new(config.hls.clone());
    #[cfg(feature = "hls")]
    let hls_handle = hls_service.trigger_handle();
    #[cfg(feature = "hls")]
    tokio::spawn(hls_service.run());

    // because we have to move them
    let agh = config.rtmp.clone();
    let ugh = shared.clone();
    // TODO: remove handle from RTMP and move to session
    javelin_rtmp::Service::new(ugh, hls_handle, agh).run().await;

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
        .level(LevelFilter::Info)
        .level_for("javelin", LevelFilter::Debug)
        .level_for("javelin_rtmp", LevelFilter::Debug)
        .level_for("javelin_types", LevelFilter::Debug)
        .level_for("javelin_core", LevelFilter::Debug)
        .level_for("javelin_codec", LevelFilter::Warn)
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

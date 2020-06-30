#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

mod args;
mod database;


use {
    anyhow::Result,
    javelin_core::{session, config},
    database::Database,
};


#[tokio::main]
async fn main() -> Result<()> {
    if let Err(why) = init_logger() {
        eprintln!("Failed to initialize logger: {}", why);
    };

    let args = args::build();
    let config_dir = args.value_of("config_dir").unwrap_or_default();
    let config = config::from_path(config_dir)?;
    let mut handles = Vec::new();

    let database_handle = Database::new(&config).await;

    let session = session::Manager::new(database_handle.clone());
    let session_handle = session.handle();
    handles.push(tokio::spawn({
        session.run()
    }));

    #[cfg(feature = "hls")]
    handles.push(tokio::spawn({
        javelin_hls::Service::new(session_handle.clone(), &config).run()
    }));

    #[cfg(feature = "rtmp")]
    handles.push(tokio::spawn({
        javelin_rtmp::Service::new(session_handle, &config).run()
    }));

    // Wait for all spawned processes to complete
    for handle in handles {
        handle.await?;
    }

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
        .level_for("javelin_hls", LevelFilter::Debug)
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

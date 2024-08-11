#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

mod args;
mod database;
mod management;


use anyhow::Result;
use clap::Parser;
use database::Database;
use javelin_core::config::{self, Config};
use javelin_core::session;

use self::args::{Args, Command};


#[tokio::main]
async fn main() -> Result<()> {
    if let Err(why) = init_tracing() {
        eprintln!("Failed to initialize logger: {}", why);
    };

    let args = Args::parse();
    let config = config::from_path(&args.config_dir)?;

    match args.cmd {
        Command::PermitStream { user, key } => {
            management::permit_stream(&user, &key, &config).await?;
        }
        Command::Run => {
            run_app(&config).await?;
        }
    }

    Ok(())
}

async fn run_app(config: &Config) -> Result<()> {
    let mut handles = Vec::new();

    let database_handle = Database::new(&config).await;

    let session = session::Manager::new(database_handle.clone());
    let session_handle = session.handle();
    handles.push(tokio::spawn(session.run()));

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

fn init_tracing() -> Result<()> {
    use tracing::Level;
    use tracing_subscriber::filter::Targets;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let max_level = if cfg!(debug_assertions) {
        Level::TRACE
    } else {
        Level::INFO
    };

    let filter_layer = Targets::new()
        .with_target("javelin", max_level)
        .with_target("javelin_rtmp", max_level)
        .with_target("javelin_hls", max_level)
        .with_target("javelin_core", max_level)
        .with_target("javelin_codec", max_level)
        .with_default(Level::ERROR);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::Layer::default())
        .with(filter_layer)
        .try_init()?;

    Ok(())
}

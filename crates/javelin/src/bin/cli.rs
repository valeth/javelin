#![warn(clippy::all)]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use javelin::database::{Database, UserRepository};
use javelin_core::config::{self, Config};


#[derive(Parser)]
#[command(version, about)]
pub struct CliArgs {
    #[arg(short, long, default_value = "./config")]
    pub config_dir: PathBuf,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    PermitStream {
        #[arg(long, required = true)]
        user: String,
        #[arg(long, required = true)]
        key: String,
    },
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let config = config::from_path(&args.config_dir)?;

    match args.cmd {
        Command::PermitStream { user, key } => {
            permit_stream(&user, &key, &config).await?;
        }
    }

    Ok(())
}


async fn permit_stream(user: &str, key: &str, config: &Config) -> Result<()> {
    let mut database_handle = Database::new(config).await;

    database_handle.add_user_with_key(user, key).await?;

    Ok(())
}

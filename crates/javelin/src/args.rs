use std::path::PathBuf;

use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, default_value = "./")]
    pub config_dir: PathBuf,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Run,
    PermitStream {
        #[arg(long, required = true)]
        user: String,
        #[arg(long, required = true)]
        key: String,
    },
}

fn capitalize(string: &str) -> String {
    string
        .chars()
        .enumerate()
        .map(|(i, c)| match i {
            0 => c.to_uppercase().to_string(),
            _ => c.to_string(),
        })
        .collect()
}

mod client;
mod commands;
mod server;
mod shared;

use crate::commands::opts::{Opts, SubCommand};
use anyhow::Result;

use clap::Parser;
use commands::{monitor, toggle_liked, toggle_play_pause};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();
    let root_opts = Opts::parse();

    match root_opts.subcmd {
        SubCommand::Monitor(opts) => monitor::run(opts).await,
        SubCommand::ToggleLiked(opts) => toggle_liked::run(opts).await,
        SubCommand::TogglePlayPause(opts) => toggle_play_pause::run(opts).await,
    }
}

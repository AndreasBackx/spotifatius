mod client;
mod commands;
mod server;
mod shared;

use crate::commands::opts::{Opts, SubCommand};
use anyhow::Result;

use clap::Parser;
use commands::{monitor, toggle_saved};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Monitor(opts) => monitor::run(opts).await,
        SubCommand::ToggleSaved(opts) => toggle_saved::run(opts).await,
    }
}

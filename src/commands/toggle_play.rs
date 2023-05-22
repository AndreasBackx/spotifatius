use anyhow::Result;
use clap::Parser;

use crate::client::service::Service;

/// Play/Pause the currently playing song.
/// Requires a monitoring instance to be running.
#[derive(Parser)]
pub struct TogglePlay {}

pub async fn run(_opts: TogglePlay) -> Result<()> {
    println!("MARKER 1");
    Service::toggle_play().await
}

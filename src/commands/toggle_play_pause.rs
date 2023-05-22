use anyhow::Result;
use clap::Parser;

use crate::client::service::Service;

/// Play/Pause the currently playing song.
/// Requires a monitoring instance to be running.
#[derive(Parser)]
pub struct TogglePlayPause {}

pub async fn run(_opts: TogglePlayPause) -> Result<()> {
    Service::toggle_play_pause().await
}

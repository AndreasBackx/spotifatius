use anyhow::Result;
use clap::Parser;

use crate::client::service::Service;

/// Add/remove the currently playing song to your liked songs.
/// Requires a monitoring instance to be running.
///
/// This will also make sure the liked message is shown on the monitoring
/// instances.
#[derive(Parser)]
pub struct ToggleLiked {}

pub async fn run(_opts: ToggleLiked) -> Result<()> {
    Service::toggle_liked().await
}

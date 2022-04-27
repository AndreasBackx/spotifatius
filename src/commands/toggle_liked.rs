use anyhow::Result;
use clap::Parser;

use crate::client::service::Service;

/// Save the currently playing song to your library if not get liked,
/// otherwise remove it. Requires a monitoring instance to be running.
///
/// This will also make sure the liked message is shown on the monitoring
/// instances.
#[derive(Parser)]
pub struct ToggleLiked {}

pub async fn run(_opts: ToggleLiked) -> Result<()> {
    Service::toggle_liked().await
}

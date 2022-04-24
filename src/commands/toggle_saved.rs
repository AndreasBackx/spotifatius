use anyhow::Result;
use clap::Parser;

use crate::client::service::Service;

/// Save the currently playing song to your library if not get saved,
/// otherwise remove it. Requires a monitoring instance to be running.
///
/// This will also make sure the saved message is shown on the monitoring
/// instances.
#[derive(Parser)]
pub struct ToggleSaved {}

pub async fn run(_opts: ToggleSaved) -> Result<()> {
    Service::toggle_saved().await
}

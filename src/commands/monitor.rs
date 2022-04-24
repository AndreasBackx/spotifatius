use std::time::Duration;

use tokio::{
    select,
    sync::broadcast,
    time::{self, Instant},
};
use tracing::debug;

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    commands::output::{Output, OutputType},
    server::grpc::api::{MonitorResponse, TrackStatus},
    server::service::Service,
};

/// Monitor the status of the currently playing song on Spotify.
///
/// A server will be spawned if not other monitoring instance is running,
/// otherwise it will connect to the existing server. This server is also used
/// by other commands.
#[derive(Parser)]
pub struct Monitor {
    /// Output type.
    #[clap(arg_enum, short, long, default_value = "waybar")]
    output_type: OutputType,
}

pub async fn run(opts: Monitor) -> Result<()> {
    let (monitor_tx, mut monitor_rx) =
        broadcast::channel::<MonitorResponse>(100);
    let mut service = Service::new(monitor_tx).await?;

    let mut monitor_handle =
        tokio::spawn(async move { service.monitor().await });

    let mut interval = time::interval(Duration::from_secs(3600));
    let mut last_output = Output::default();

    loop {
        select! {
            monitor_result = &mut monitor_handle => {
                monitor_result??;
                break;
            }
            _ = interval.tick() => {
                last_output.print(opts.output_type)?;
            }
            Ok(response) = monitor_rx.recv() => {
                debug!("{:#?}", response);
                let status = TrackStatus::from_i32(response.status).context(format!("invalid status value '{}' passed", response.status))?;
                let output = if let Some(track) = response.track {
                    interval.reset();

                    let mut class = vec![];
                    let mut separator = "-";
                    if response.is_saved.unwrap_or_default() {
                        class.push("saved".to_string());
                        separator = "+";
                    }
                    class.push(status.into());
                    let text = match (track.artist, track.title) {
                        (Some(artist), Some(title)) => {
                            format!("{} {} {}", artist, separator, title)
                        }
                        (Some(artist), None) => artist,
                        (None, Some(title)) => title,
                        (None, None) => "".to_string(),
                    };
                    last_output = Output {
                        text,
                        tooltip: track.album,
                        class: Some(class),
                    };
                    last_output.clone()
                } else if status == TrackStatus::Saved {
                    interval = time::interval_at(Instant::now() + Duration::from_secs(2), interval.period());
                    Output {
                        text: "Saved to library!".to_string(),
                        tooltip: None,
                        class: None,
                    }
                } else if status == TrackStatus::Removed {
                    interval = time::interval_at(Instant::now() + Duration::from_secs(2), interval.period());
                    Output {
                        text: "Removed from library!".to_string(),
                        tooltip: None,
                        class: None,
                    }
                } else {
                    Output {
                        text: "".to_string(),
                        tooltip: None,
                        class: None,
                    }
                };
                output.print(opts.output_type)?;
            }
        }
    }
    Ok(())
}

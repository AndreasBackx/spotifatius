use std::{path::PathBuf, time::Duration};

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
    shared::config::{get_config, DEFAULT_CONFIG_PATH},
};

use super::output::OutputFormatter;

/// Monitor the status of the currently playing song on Spotify.
///
/// A server will be spawned if not other monitoring instance is running,
/// otherwise it will connect to the existing server. This server is also used
/// by other commands.
#[derive(Parser)]
pub struct Monitor {
    /// Config file path.
    #[clap(
        short,
        long,
        parse(from_os_str),
        default_value = DEFAULT_CONFIG_PATH
    )]
    pub config: PathBuf,
    /// Output type.
    #[clap(arg_enum, short, long, default_value = "waybar")]
    output_type: OutputType,
}

pub async fn run(opts: Monitor) -> Result<()> {
    let config = get_config(opts.config)?;
    let formatter = OutputFormatter {
        output_type: opts.output_type,
        config,
    };

    let (update_requests_tx, _) = broadcast::channel::<()>(1);
    let (monitor_tx, mut monitor_rx) =
        broadcast::channel::<MonitorResponse>(100);
    let mut service = Service::new(monitor_tx).await?;

    let service_update_requests_tx = update_requests_tx.clone();
    let mut monitor_handle = tokio::spawn(async move {
        service.monitor(service_update_requests_tx).await
    });

    let mut interval = time::interval_at(
        Instant::now() + Duration::from_secs(3600),
        Duration::from_secs(3600),
    );
    loop {
        select! {
            monitor_result = &mut monitor_handle => {
                monitor_result??;
                break;
            }
            _ = interval.tick() => {
                debug!("Interval has passed, updating!");
                update_requests_tx.send(())?;
            }
            Ok(response) = monitor_rx.recv() => {
                debug!("{:#?}", response);
                let status = TrackStatus::from_i32(response.status).context(format!("invalid status value '{}' passed", response.status))?;
                let output = if let Some(track) = response.track {
                    interval.reset();

                    let mut class = vec![];
                    let mut separator = "-";
                    if response.is_liked.unwrap_or_default() {
                        class.push("liked".to_string());
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
                    Output {
                        text,
                        tooltip: track.album,
                        class: Some(class),
                    }
                } else if status == TrackStatus::Added {
                    interval = time::interval_at(Instant::now() + Duration::from_secs(2), interval.period());
                    Output {
                        text: "Added to library!".to_string(),
                        tooltip: None,
                        class: Some(vec![status.into()]),
                    }
                } else if status == TrackStatus::Removed {
                    interval = time::interval_at(Instant::now() + Duration::from_secs(2), interval.period());
                    Output {
                        text: "Removed from library!".to_string(),
                        tooltip: None,
                        class: Some(vec![status.into()]),
                    }
                } else {
                    Output {
                        text: "".to_string(),
                        tooltip: None,
                        class: None,
                    }
                };
                formatter.print(output)?;
            }
        }
    }
    Ok(())
}

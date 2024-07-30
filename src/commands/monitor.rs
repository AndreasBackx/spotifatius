use std::path::PathBuf;

use tokio::{select, sync::broadcast};
use tracing::debug;

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    commands::output::OutputType,
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
    let output_format = config.clone().format;
    let formatter = OutputFormatter {
        output_type: opts.output_type,
        config,
    };

    let (monitor_tx, mut monitor_rx) =
        broadcast::channel::<MonitorResponse>(100);
    let mut service = Service::new(monitor_tx).await?;

    let mut monitor_handle =
        tokio::spawn(async move { service.monitor().await });

    loop {
        select! {
            monitor_result = &mut monitor_handle => {
                monitor_result??;
                break;
            }
            Ok(response) = monitor_rx.recv() => {
                debug!("{:#?}", response);
                let status = TrackStatus::from_i32(response.status).context(format!("invalid status value '{}' passed", response.status))?;
                let output = formatter.format_output(response, status, &output_format);
                formatter.print(output)?;
            }
        }
    }
    Ok(())
}

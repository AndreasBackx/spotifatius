use clap::{Parser, Subcommand};

use super::{
    monitor::Monitor, toggle_liked::ToggleLiked,
    toggle_play_pause::TogglePlayPause,
};

/// A simple Spotify CLI primarily made for monitoring what songs you're
/// listening to and displaying that in your bar of choice like waybar or polybar.
#[derive(Parser)]
#[clap(version, author = "Andreas Backx")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Monitor(Monitor),
    ToggleLiked(ToggleLiked),
    TogglePlayPause(TogglePlayPause),
}

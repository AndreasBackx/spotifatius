use anyhow::{Error, Result};

use std::str::FromStr;

use anyhow::bail;

tonic::include_proto!("spotifatius");

#[derive(Debug, PartialEq, Clone)]
pub enum ChangeEvent {
    SpotifyOpened,
    SpotifyClosed,
    TrackChange(TrackChange),
    TrackLiked(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackChange {
    pub status: TrackStatus,
    pub track: Track,
}

impl FromStr for TrackStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "playing" => Ok(TrackStatus::Playing),
            "stopped" => Ok(TrackStatus::Stopped),
            "paused" => Ok(TrackStatus::Paused),
            "added" => Ok(TrackStatus::Added),
            "removed" => Ok(TrackStatus::Removed),
            _ => bail!(s.to_string()),
        }
    }
}

impl From<TrackStatus> for String {
    fn from(track_status: TrackStatus) -> String {
        match track_status {
            TrackStatus::Playing => "playing",
            TrackStatus::Stopped => "stopped",
            TrackStatus::Paused => "paused",
            TrackStatus::Added => "added",
            TrackStatus::Removed => "removed",
        }
        .to_string()
    }
}

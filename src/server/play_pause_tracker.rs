use super::grpc::api::ChangeEvent;
use anyhow::{Context, Result};
use rspotify::AuthCodeSpotify;
use tokio::sync::mpsc::Sender;

pub struct PlayPauseTracker {
    change_tx: Sender<ChangeEvent>,
    spotify: AuthCodeSpotify,
    pub current_track_id: Option<String>,
}

impl PlayPauseTracker {
    pub async fn new(
        change_tx: Sender<ChangeEvent>,
        spotify: AuthCodeSpotify,
    ) -> Result<Self> {
        Ok(Self {
            change_tx,
            spotify,
            current_track_id: None,
        })
    }

    pub async fn toggle(
        &mut self,
        track_id_or_current: Option<String>,
        force_refresh: bool,
    ) -> Result<bool> {
        let track_id = track_id_or_current
            .or_else(|| self.current_track_id.clone())
            .context("no current track playing")?;
        // TODO
        Ok(false)
    }
}

use std::{collections::HashMap, fs::create_dir_all, path::PathBuf};

use anyhow::{Context, Result};
use rspotify::{
    clients::OAuthClient, model::TrackId, prelude::Id, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth, DEFAULT_CACHE_PATH,
};
use tokio::sync::mpsc::Sender;
use tracing::{debug, info};

use crate::shared::config::{resolve_home_path, DEFAULT_CONFIG_FOLDER};

use super::grpc::api::ChangeEvent;

pub struct SavedTracker {
    spotify: AuthCodeSpotify,
    tracks: Tracks,
    pub current_track_id: Option<String>,
    change_tx: Sender<ChangeEvent>,
}

#[derive(Default)]
struct Tracks {
    saved: HashMap<String, bool>,
}

impl Tracks {
    fn is_saved(&self, track: &String) -> Option<bool> {
        self.saved.get(track).map(|b| b.to_owned())
    }

    fn add(&mut self, track: String, saved: bool) -> Option<bool> {
        self.saved.insert(track, saved)
    }
}

impl SavedTracker {
    pub async fn new(change_tx: Sender<ChangeEvent>) -> Result<Self> {
        let oauth = OAuth {
            redirect_uri: "http://localhost".to_string(),
            scopes: scopes!("user-library-read", "user-library-modify"),
            ..Default::default()
        };
        let creds = Credentials::from_env().context(
            "Could not find RSPOTIFY_CLIENT_(ID|SECRET) environment variables",
        )?;
        let cache_folder =
            resolve_home_path(PathBuf::from(DEFAULT_CONFIG_FOLDER))?;
        create_dir_all(cache_folder.clone()).with_context(|| {
            format!("Could not create cache folder {cache_folder:?}")
        })?;
        let cache_path = cache_folder.join(DEFAULT_CACHE_PATH);
        // error!("{cache_path}");
        let config = Config {
            token_cached: true,
            token_refreshing: true,
            cache_path,
            ..Default::default()
        };
        let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

        let url = spotify.get_authorize_url(true)?;
        spotify.prompt_for_token(&url).await?; // This is where it crashes.

        Ok(SavedTracker {
            spotify,
            tracks: Tracks::default(),
            current_track_id: None,
            change_tx,
        })
    }

    pub async fn save(&mut self, track_id: String) -> Result<()> {
        self.spotify
            .current_user_saved_tracks_add(
                [TrackId::from_id(&track_id)?].iter(),
            )
            .await?;
        self.tracks.add(track_id, true);
        Ok(())
    }

    pub async fn remove(&mut self, track_id: String) -> Result<()> {
        self.spotify
            .current_user_saved_tracks_delete(
                [TrackId::from_id(&track_id)?].iter(),
            )
            .await?;
        self.tracks.add(track_id, false);
        Ok(())
    }

    pub fn is_saved_cached(&self, track_id: &String) -> Option<bool> {
        self.tracks.is_saved(track_id)
    }

    pub async fn check_saved(
        &mut self,
        track_id: String,
        force_refresh: bool,
    ) -> Result<bool> {
        if !force_refresh {
            if let Some(saved) = self.is_saved_cached(&track_id) {
                debug!("{} is cached: {}.", track_id, saved);
                return Ok(saved);
            }
        }
        let saved = self
            .spotify
            .current_user_saved_tracks_contains(
                [TrackId::from_id(&track_id)?].iter(),
            )
            .await
            .map(|saved| saved[0])?;
        debug!("{} is saved: {}", track_id, saved);

        self.tracks.add(track_id, saved);
        Ok(saved)
    }

    pub async fn toggle_saved(
        &mut self,
        track_id_or_current: Option<String>,
        force_refresh: bool,
    ) -> Result<bool> {
        let track_id = track_id_or_current
            .or_else(|| self.current_track_id.clone())
            .context("no current track playing")?;
        if !self.check_saved(track_id.clone(), force_refresh).await? {
            info!("Saving");
            self.save(track_id).await?;
            self.change_tx.send(ChangeEvent::TrackSaved(true)).await?;
            Ok(true)
        } else {
            info!("Removing");
            self.remove(track_id).await?;
            self.change_tx.send(ChangeEvent::TrackSaved(false)).await?;
            Ok(false)
        }
    }
}

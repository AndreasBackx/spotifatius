use anyhow::{Context, Result};
use rspotify::{
    clients::OAuthClient, model::TrackId, prelude::Id, AuthCodeSpotify,
};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info};

use super::grpc::api::ChangeEvent;

pub struct LikedTracker {
    spotify: AuthCodeSpotify,
    tracks: Tracks,
    pub current_track_id: Option<String>,
    change_tx: Sender<ChangeEvent>,
}

#[derive(Default)]
struct Tracks {
    liked: HashMap<String, bool>,
}

impl Tracks {
    fn is_liked(&self, track: &String) -> Option<bool> {
        self.liked.get(track).map(|b| b.to_owned())
    }

    fn add(&mut self, track: String, liked: bool) -> Option<bool> {
        self.liked.insert(track, liked)
    }
}

impl LikedTracker {
    pub async fn new(
        change_tx: Sender<ChangeEvent>,
        spotify: AuthCodeSpotify,
    ) -> Result<Self> {
        Ok(LikedTracker {
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

    pub fn is_liked_cached(&self, track_id: &String) -> Option<bool> {
        self.tracks.is_liked(track_id)
    }

    pub async fn check_liked(
        &mut self,
        track_id: String,
        force_refresh: bool,
    ) -> Result<bool> {
        if !force_refresh {
            if let Some(liked) = self.is_liked_cached(&track_id) {
                debug!("{} is cached: {}.", track_id, liked);
                return Ok(liked);
            }
        }
        let liked = self
            .spotify
            .current_user_saved_tracks_contains(
                [TrackId::from_id(&track_id)?].iter(),
            )
            .await
            .map(|liked| liked[0])?;
        debug!("{} is liked: {}", track_id, liked);

        self.tracks.add(track_id, liked);
        Ok(liked)
    }

    pub async fn toggle_liked(
        &mut self,
        track_id_or_current: Option<String>,
        force_refresh: bool,
    ) -> Result<bool> {
        let track_id = track_id_or_current
            .or_else(|| self.current_track_id.clone())
            .context("no current track playing")?;
        if !self.check_liked(track_id.clone(), force_refresh).await? {
            info!("Saving");
            self.save(track_id).await?;
            self.change_tx.send(ChangeEvent::TrackLiked(true)).await?;
            Ok(true)
        } else {
            info!("Removing");
            self.remove(track_id).await?;
            self.change_tx.send(ChangeEvent::TrackLiked(false)).await?;
            Ok(false)
        }
    }
}

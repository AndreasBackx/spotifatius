use std::pin::Pin;
use std::sync::Arc;

use crate::server::liked_tracker::LikedTracker;
use crate::server::play_pause_tracker::PlayPauseTracker;

use super::api::spotifatius_server::Spotifatius;
use super::api::{
    MonitorRequest, MonitorResponse, ToggleLikedRequest, ToggleLikedResponse,
    TogglePlayRequest, TogglePlayResponse,
};
use super::monitor_client::MonitorClient;
use anyhow::Result;
use futures::Stream;
use tokio::sync::{self, broadcast};

use tokio::sync::broadcast::Sender;
use tonic::{Request, Response, Status};

use super::wake_watcher::WakeWatcher;

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<MonitorResponse, Status>> + Send + Sync>>;

pub struct MySpotifatius {
    liked_tracker: Arc<sync::Mutex<LikedTracker>>,
    play_pause_tracker: Arc<sync::Mutex<PlayPauseTracker>>,
    monitor_tx: Sender<MonitorResponse>,
    wake_watcher: Arc<WakeWatcher>,
    update_requests_tx: broadcast::Sender<()>,
}

impl MySpotifatius {
    pub fn new(
        liked_tracker: Arc<sync::Mutex<LikedTracker>>,
        play_pause_tracker: Arc<sync::Mutex<PlayPauseTracker>>,
        monitor_tx: Sender<MonitorResponse>,
        wake_watcher: Arc<WakeWatcher>,
        update_requests_tx: broadcast::Sender<()>,
    ) -> Self {
        MySpotifatius {
            liked_tracker,
            play_pause_tracker,
            monitor_tx,
            wake_watcher,
            update_requests_tx,
        }
    }
}

#[tonic::async_trait]
impl Spotifatius for MySpotifatius {
    type MonitorStream = ResponseStream;

    async fn monitor(
        &self,
        _request: Request<MonitorRequest>,
    ) -> Result<Response<Self::MonitorStream>, Status> {
        let rx = self.monitor_tx.subscribe();
        self.update_requests_tx
            .send(())
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(Box::pin(MonitorClient {
            rx,
            wake_watcher: self.wake_watcher.clone(),
        }) as Self::MonitorStream))
    }

    async fn toggle_liked(
        &self,
        _request: Request<ToggleLikedRequest>,
    ) -> Result<Response<ToggleLikedResponse>, Status> {
        let is_liked = self
            .liked_tracker
            .lock()
            .await
            .toggle_liked(None, false)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(ToggleLikedResponse { is_liked }))
    }

    async fn toggle_play(
        &self,
        _request: Request<TogglePlayRequest>,
    ) -> Result<Response<TogglePlayResponse>, Status> {
        println!("MARKER 6");
        self.play_pause_tracker
            .lock()
            .await
            .toggle(None, false)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(TogglePlayResponse {}))
    }
}

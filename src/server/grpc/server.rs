use std::pin::Pin;
use std::sync::Arc;

use crate::server::saved_tracker::SavedTracker;

use super::api::spotifatius_server::Spotifatius;
use super::api::{
    MonitorRequest, MonitorResponse, ToggleSavedRequest, ToggleSavedResponse,
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
    saved_tracker: Arc<sync::Mutex<SavedTracker>>,
    monitor_tx: Sender<MonitorResponse>,
    wake_watcher: Arc<WakeWatcher>,
    update_requests_tx: broadcast::Sender<()>,
}

impl MySpotifatius {
    pub fn new(
        saved_tracker: Arc<sync::Mutex<SavedTracker>>,
        monitor_tx: Sender<MonitorResponse>,
        wake_watcher: Arc<WakeWatcher>,
        update_requests_tx: broadcast::Sender<()>,
    ) -> Self {
        MySpotifatius {
            saved_tracker,
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

    async fn toggle_saved(
        &self,
        _request: Request<ToggleSavedRequest>,
    ) -> Result<Response<ToggleSavedResponse>, Status> {
        let is_saved = self
            .saved_tracker
            .lock()
            .await
            .toggle_saved(None, false)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(ToggleSavedResponse { is_saved }))
    }
}

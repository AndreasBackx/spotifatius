use std::error::Error;

use crate::shared::config::{resolve_home_path, DEFAULT_CONFIG_FOLDER};
use std::{fs::create_dir_all, path::PathBuf};
use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};

use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use tokio::time::Instant;
use tokio::{self, time};
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tracing::debug;

use crate::server::grpc::server::MySpotifatius;
use crate::server::{
    dbus::DBusClient,
    grpc::api::{
        spotifatius_client::SpotifatiusClient,
        spotifatius_server::SpotifatiusServer, MonitorRequest, MonitorResponse,
        Track, TrackStatus,
    },
    liked_tracker::LikedTracker,
    play_pause_tracker::PlayPauseTracker,
};

use super::grpc::api::ChangeEvent;
use super::grpc::wake_watcher::WakeWatcher;

use crate::shared::consts::ADDRESS;
use rspotify::{
    clients::OAuthClient, scopes, AuthCodeSpotify, Config, Credentials, OAuth,
    DEFAULT_CACHE_PATH,
};

pub struct Service {
    monitor_tx: broadcast::Sender<MonitorResponse>,
    liked_tracker: Arc<Mutex<LikedTracker>>,
    play_pause_tracker: Arc<Mutex<PlayPauseTracker>>,
    change_tx: mpsc::Sender<ChangeEvent>,
    change_rx: mpsc::Receiver<ChangeEvent>,
    wake_watcher: Arc<WakeWatcher>,
}

impl Service {
    pub async fn new(
        monitor_tx: broadcast::Sender<MonitorResponse>,
    ) -> Result<Self> {
        let (change_tx, change_rx) = mpsc::channel::<ChangeEvent>(100);

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

        let liked_tracker = Arc::new(Mutex::new(
            LikedTracker::new(change_tx.clone(), spotify.clone()).await?,
        ));
        let wake_watcher = Arc::new(WakeWatcher::new());
        let play_pause_tracker = Arc::new(Mutex::new(
            PlayPauseTracker::new(change_tx.clone(), spotify.clone()).await?,
        ));

        Ok(Service {
            monitor_tx,
            liked_tracker,
            play_pause_tracker,
            change_tx,
            change_rx,
            wake_watcher,
        })
    }

    fn send_and_wake(&self, value: MonitorResponse) -> Result<usize> {
        let result = self.monitor_tx.send(value)?;
        self.wake_watcher.wake()?;
        Ok(result)
    }

    async fn monitor_client(&mut self) -> Result<()> {
        let mut client =
            SpotifatiusClient::connect(format!("http://{ADDRESS}")).await?;
        let request = tonic::Request::new(MonitorRequest {});
        let mut stream = client.monitor(request).await?.into_inner();

        while let Some(item) = stream.next().await {
            self.monitor_tx
                .send(item?)
                .context("Could not forward gRPC stream")?;
        }

        Ok(())
    }

    pub async fn monitor(&mut self) -> Result<()> {
        let (update_requests_tx, _) = broadcast::channel::<()>(1);
        let change_tx = self.change_tx.clone();
        let rpc = MySpotifatius::new(
            self.liked_tracker.clone(),
            self.play_pause_tracker.clone(),
            self.monitor_tx.clone(),
            self.wake_watcher.clone(),
            update_requests_tx.clone(),
        );

        let mut dbus = DBusClient::new(change_tx, update_requests_tx.clone());
        let mut dbus_handle = tokio::spawn(async move { dbus.listen().await });

        let addr = ADDRESS.parse()?;
        let mut rpc_handle = tokio::spawn(async move {
            Server::builder()
                .add_service(SpotifatiusServer::new(rpc))
                .serve(addr)
                .await
        });

        let mut interval = time::interval_at(
            Instant::now() + Duration::from_secs(3600),
            Duration::from_secs(3600),
        );

        loop {
            tokio::select! {
                Some(change_event) = self.change_rx.recv() => {
                    debug!("Received {:#?}", change_event);
                    let mut tracker = self.liked_tracker.lock().await;
                    match change_event {
                        ChangeEvent::TrackChange(track_change) => {
                            tracker.current_track_id = track_change.track.id.clone();
                            if let Some(track_id) = track_change.track.id {
                                // If there's an interval running to request an update,
                                // cancel it because it's no longer needed.
                                // Only if the current title/artist are not empty,
                                // because when opening Spotify the title/artist
                                // will be empty initially.
                                if track_change.track.artist.is_some() && track_change.track.title.is_some() {
                                    interval.reset();
                                }

                                let is_cached_liked = tracker.is_liked_cached(&track_id);

                                self.send_and_wake(MonitorResponse {
                                    track:Some( Track { id: Some(track_id.clone()), artist: track_change.track.artist.clone(), title: track_change.track.title.clone(), album: track_change.track.album.clone() }),
                                    status: track_change.status.into(),
                                    is_liked: is_cached_liked,
                                })?;

                                if is_cached_liked.is_none() {
                                    debug!("Save status wasn't cached yet, caching it now!");
                                    let is_liked = tracker.check_liked(track_id.clone(), false).await?;

                                    if is_liked {
                                        debug!("New monitor response because is_liked went from unknown to true");
                                        self.send_and_wake( MonitorResponse {
                                            track:Some( Track { id: Some(track_id), artist: track_change.track.artist, title: track_change.track.title, album: track_change.track.album }),
                                            status: track_change.status.into(),
                                            is_liked: Some(is_liked),
                                        })?;
                                    }
                                }
                            } else {
                                self.send_and_wake( MonitorResponse {
                                    track: None,
                                    status: TrackStatus::Stopped.into(),
                                    is_liked: None,
                                })?;
                            };
                        }
                        ChangeEvent::SpotifyOpened => {
                            // There is a bug where when Spotify opens, it will
                            // not have an artist/title. Immedaitely asking for
                            // an update will not work, but waiting ~0.2s will.
                            // We'll use 0.5s just in case.
                            interval = time::interval_at(Instant::now() + Duration::from_millis(500), interval.period());
                        }
                        ChangeEvent::SpotifyClosed => {
                            tracker.current_track_id = None;
                            self.send_and_wake( MonitorResponse {
                                track: None,
                                status: TrackStatus::Stopped.into(),
                                is_liked: None,
                            })?;
                        }
                        ChangeEvent::TrackLiked(is_liked) => {
                            interval = time::interval_at(Instant::now() + Duration::from_secs(2), interval.period());
                            self.send_and_wake( MonitorResponse {
                                track: None,
                                status: if is_liked {TrackStatus::Added} else {TrackStatus::Removed}.into(),
                                is_liked: Some(is_liked),
                            })?;
                        }
                    }
                }
                _ = interval.tick() => {
                    debug!("Interval has passed, updating!");
                    update_requests_tx.send(()).context("Could not request update")?;
                }
                join_result = &mut dbus_handle => {
                    rpc_handle.abort();
                    self.change_rx.close();
                    return join_result?.context("DBUS client closed early!");
                }
                join_result = &mut rpc_handle => {
                    dbus_handle.abort();
                    self.change_rx.close();
                    if let Err(e) = join_result? {
                        let error_message = e.source().map(|err| err.to_string()).unwrap_or_else(||"".to_string());
                        let is_address_in_use = error_message.ends_with("(os error 98)");

                        if is_address_in_use {
                            return self.monitor_client().await;
                        }
                        return Err(e).context("RPC server closed early");
                    }
                    return Ok(());
                }
                else => {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
        }
    }
}

use anyhow::{Context, Result};
use futures::{future, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::BroadcastStream;

use crate::server::grpc::api::{Track, TrackChange, TrackStatus};
use futures::stream::select;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use tokio::select as tokio_select;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use zbus::fdo::{DBusProxy, PropertiesProxy};
use zbus::names::{InterfaceName, UniqueName};
use zbus::Connection;

use zvariant::{Array, Dict, OwnedValue, Value};

use super::grpc::api::ChangeEvent;

const FREEDESKTOP_DEST: &str = "org.freedesktop.DBus";
const FREEDESKTOP_PATH: &str = "/org/freedesktop/DBus";

const SPOTIFY_DEST: &str = "org.mpris.MediaPlayer2.spotify";
const SPOTIFY_PATH: &str = "/org/mpris/MediaPlayer2";

pub struct DBusClient {
    events_tx: mpsc::Sender<ChangeEvent>,
    update_requests_tx: broadcast::Sender<()>,
}

impl DBusClient {
    pub fn new(
        events_tx: mpsc::Sender<ChangeEvent>,
        update_requests_tx: broadcast::Sender<()>,
    ) -> Self {
        DBusClient {
            events_tx,
            update_requests_tx,
        }
    }

    /// Listen for song changes, mirroring what you would see playing in Spotify.
    pub async fn listen(&mut self) -> Result<()> {
        info!("Starting to listen on DBUS...");
        let update_requests_rx = self.update_requests_tx.subscribe();
        let (change_tx, mut change_rx) = mpsc::channel::<ChangeEvent>(10);

        self.update_requests_tx.send(())?;

        let mut window_handle =
            tokio::spawn(DBusClient::listen_spotify_window(change_tx.clone()));

        let mut change_handle = tokio::spawn(DBusClient::listen_song_changes(
            change_tx.clone(),
            BroadcastStream::new(update_requests_rx),
        ));

        loop {
            tokio_select! {
                Some(change_event) = change_rx.recv() => {
                    debug!("Forwarding: {:#?}", change_event);
                    self.events_tx.send(change_event).await?;
                }
                join_result = &mut window_handle => {
                    change_handle.abort();
                    return join_result.context("An error occurred while listening for window changes")?;
                },
                join_result = &mut change_handle => {
                    window_handle.abort();
                    return join_result.context("An error occurred while listening for song changes")?;
                },
                else => {
                    warn!("failed to get info");
                    break
                },
            }
        }

        Ok(())
    }

    /// Listen for song changes.
    ///
    /// Song changes will be published to the passed `events` sender.
    /// The `update_requests_rx` receiver that is passed can be used to request
    /// an explicit update. That can be useful for example when Spotify opens,
    /// but hasn't sent a song update yet. You would then request one explicitly.
    async fn listen_song_changes(
        events: mpsc::Sender<ChangeEvent>,
        update_requests_rx: BroadcastStream<()>,
    ) -> Result<()> {
        let connection = Connection::session().await?;
        let props = PropertiesProxy::builder(&connection)
            .destination(SPOTIFY_DEST)?
            .path(SPOTIFY_PATH)?
            .build()
            .await?;

        let props_changed_stream = props
            .receive_properties_changed()
            .await?
            .filter_map(|signal| {
                async move {
                    match signal.args().ok() {
                        Some(args) => {
                            debug!("{args:#?}");
                            let changed_properties =
                                args.changed_properties().clone().to_owned();

                            let playback_status: Option<OwnedValue> =
                                changed_properties
                                    .get("PlaybackStatus")
                                    .map(|a| a.into());
                            let metadata: Option<OwnedValue> =
                                changed_properties
                                    .get("Metadata")
                                    .map(|a| a.into());

                            Some((playback_status, metadata, false))
                        }
                        None => None,
                    }
                }
            });

        let player_interface_name =
            InterfaceName::try_from("org.mpris.MediaPlayer2.Player")?;
        let update_requests_stream = update_requests_rx.then(|_| {
            debug!("Received DBUS update request:");
            async {
                (
                    props
                        .get(player_interface_name.clone(), "PlaybackStatus")
                        .await
                        .ok(),
                    props
                        .get(player_interface_name.clone(), "Metadata")
                        .await
                        .ok(),
                    true,
                )
            }
        });

        let mut merged_stream = Box::pin(select(
            update_requests_stream,
            props_changed_stream,
        ))
        .scan(
            (None, None),
            |(last_playback_value, last_metadata_value),
             (playback_value, metadata_value, is_update_request)| {
                let item = (
                    playback_value.or_else(|| (*last_playback_value).clone()),
                    metadata_value.or_else(|| (*last_metadata_value).clone()),
                    is_update_request,
                );

                *last_playback_value = item.0.clone();
                *last_metadata_value = item.1.clone();

                future::ready(Some(item))
            },
        );

        let mut last_song_change = None;

        while let Some((playback_value, metadata_value, is_update_request)) =
            merged_stream.next().await
        {
            debug!("Received DBUS update:");
            debug!("playback_value: {playback_value:#?}");
            debug!("metadata_value: {metadata_value:#?}");
            debug!("is_update_request: {is_update_request:#?}");

            let status = playback_value
                .map(|value: OwnedValue| -> Value { value.into() })
                .and_then(|value| value.clone().downcast::<String>())
                .and_then(|value| TrackStatus::from_str(value.as_str()).ok())
                .unwrap_or(TrackStatus::Stopped);

            let metadata = metadata_value
                .map(|value: OwnedValue| -> Value { value.into() })
                .and_then(|value| value.clone().downcast::<Dict>())
                .and_then(|value| -> Option<HashMap<String, Value>> {
                    value.try_into().ok()
                });

            let artist = metadata
                .as_ref()
                .and_then(|value| {
                    value
                        .get("xesam:artist")
                        .and_then(|value| value.clone().downcast::<Array>())
                })
                .and_then(|arr| {
                    arr.iter()
                        .map(|value| value.try_into().ok())
                        .collect::<Option<Vec<String>>>()
                        .and_then(|items| items.into_iter().next())
                })
                .filter(|value| !value.is_empty());

            let title = metadata
                .as_ref()
                .and_then(|value| {
                    value
                        .get("xesam:title")
                        .and_then(|value| value.clone().downcast::<String>())
                })
                .filter(|value| !value.is_empty());

            let album = metadata
                .as_ref()
                .and_then(|value| {
                    value
                        .get("xesam:album")
                        .and_then(|value| value.clone().downcast::<String>())
                })
                .filter(|value| !value.is_empty());

            let id = metadata
                .as_ref()
                .and_then(|value| {
                    value
                        .get("mpris:trackid")
                        .and_then(|value| value.clone().downcast::<String>())
                })
                .filter(|value| !value.is_empty())
                .and_then(|value| {
                    // There are 2 track ID formats. One of them being:
                    // "/com/spotify/track/{TRACK_ID}", the other being
                    // similar but with ":" instead of "/" and also
                    // ending with "{TRACK_ID}" so we pick the last.
                    value.split(&['/', ':']).last().map(|raw| raw.to_string())
                });

            let song_change = TrackChange {
                status,
                track: Track {
                    artist,
                    title,
                    album,
                    id,
                },
            };
            if let Some(last) = last_song_change.clone() {
                if !is_update_request && last == song_change {
                    debug!("Skip sending track change as it was the same and not an update request");
                    continue;
                }
            }
            last_song_change = Some(song_change.clone());
            debug!("Sending: {song_change:#?}");
            events.send(ChangeEvent::TrackChange(song_change)).await?;
        }

        Ok(())
    }

    /// Listen for when the Spotify window is opened or closed.
    async fn listen_spotify_window(
        events: mpsc::Sender<ChangeEvent>,
    ) -> Result<()> {
        let connection = Connection::session().await?;

        let dbus = DBusProxy::builder(&connection)
            .destination(FREEDESKTOP_DEST)?
            .path(FREEDESKTOP_PATH)?
            .build()
            .await?;
        let mut name_owner_changed_stream =
            dbus.receive_name_owner_changed().await?;
        while let Some(signal) = name_owner_changed_stream.next().await {
            let args = signal.args()?;

            if args.name().to_owned() != SPOTIFY_DEST {
                continue;
            }

            if signal
                .args()?
                .new_owner()
                .as_ref()
                .unwrap_or(&UniqueName::from_str_unchecked(""))
                == ""
            {
                events.send(ChangeEvent::SpotifyClosed).await?;
            } else {
                events.send(ChangeEvent::SpotifyOpened).await?;
            }
        }

        Ok(())
    }
}

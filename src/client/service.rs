use anyhow::{Context, Result};

use crate::server::grpc::api::{
    spotifatius_client::SpotifatiusClient, ToggleLikedRequest,
    TogglePlayPauseRequest,
};
use crate::shared::consts::ADDRESS;

pub struct Service {}

impl Service {
    pub async fn toggle_liked() -> Result<()> {
        let mut client =
            SpotifatiusClient::connect(format!("http://{ADDRESS}")).await
            .context("Could not connect to monitor instance, make sure there is one running")?;
        let request = tonic::Request::new(ToggleLikedRequest {});
        let response = client.toggle_liked(request).await?;
        println!(
            "{}",
            if response.get_ref().is_liked {
                "Added to library!"
            } else {
                "Removed from library!"
            }
        );
        Ok(())
    }

    pub async fn toggle_play_pause() -> Result<()> {
        let mut client =
            SpotifatiusClient::connect(format!("http://{ADDRESS}")).await
            .context("Could not connect to monitor instance, make sure there is one running")?;
        let request = tonic::Request::new(TogglePlayPauseRequest {});
        client.toggle_play_pause(request).await?;
        Ok(())
    }
}

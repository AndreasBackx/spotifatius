use anyhow::{Context, Result};

use crate::server::grpc::api::{
    spotifatius_client::SpotifatiusClient, ToggleLikedRequest,
    TogglePlayRequest,
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

    pub async fn toggle_play() -> Result<()> {
        println!("MARKER 2");
        let mut client =
            SpotifatiusClient::connect(format!("http://{ADDRESS}")).await
            .context("Could not connect to monitor instance, make sure there is one running")?;
        println!("MARKER 3 {:?}", client);
        let request = tonic::Request::new(TogglePlayRequest {});
        println!("MARKER 4 {:?}", request);
        match client.toggle_play(request).await {
            Ok(resp) => println!("MARKER 5 {:?}", resp),
            Err(e) => println!("e: {:?}", e),
        }
        Ok(())
    }
}

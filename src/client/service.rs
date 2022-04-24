use anyhow::Result;

use crate::server::grpc::api::{
    spotifatius_client::SpotifatiusClient, ToggleSavedRequest,
};
use crate::shared::consts::ADDRESS;

pub struct Service {}

impl Service {
    pub async fn toggle_saved() -> Result<()> {
        let mut client =
            SpotifatiusClient::connect(format!("http://{ADDRESS}")).await?;
        let request = tonic::Request::new(ToggleSavedRequest {});
        let response = client.toggle_saved(request).await?;
        println!(
            "{}",
            if response.get_ref().is_saved {
                "Saved to library!"
            } else {
                "Removed from library!"
            }
        );
        Ok(())
    }
}

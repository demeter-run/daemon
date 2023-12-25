use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    domain::{Domain, Event},
    driven::event_dispatch::EventWrapper,
};

pub async fn run(domain: Arc<Mutex<Domain>>) -> Result<()> {
    let mut subscription = { domain.lock().await.event_dispatch.subscribe() };

    while let Ok(EventWrapper(evt, _)) = subscription.recv().await {
        let mut domain = domain.lock().await;

        match evt {
            Event::ApiKeyRegisteredV1(evt) => domain
                .on_apikey_registered(evt)
                .await
                .context("handling apikey registered event")?,
            Event::NamespaceMintedV1(evt) => {
                warn!(ns = evt.name, "TODO: handle namespace minted");
            }
            Event::ResourceCreatedV1(evt) => {
                warn!(
                    uuid = hex::encode(evt.resource_uuid),
                    "TODO: handle resource created"
                );
            }
        }
    }

    Ok(())
}

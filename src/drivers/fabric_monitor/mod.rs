use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{domain::Domain, driven::event_dispatch::EventWrapper};

pub async fn run(domain: Arc<Mutex<Domain>>) -> Result<()> {
    let mut subscription = { domain.lock().await.event_dispatch.subscribe() };

    while let Ok(EventWrapper(evt, _)) = subscription.recv().await {
        let mut domain = domain.lock().await;
        domain.handle(evt).await?;
    }

    Ok(())
}

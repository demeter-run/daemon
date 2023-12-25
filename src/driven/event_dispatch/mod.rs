use anyhow::Result;
use tokio::sync::broadcast::Receiver;

use crate::domain::Event;

pub type EventReceipt = Vec<u8>;

#[derive(Debug, Clone)]
pub struct EventWrapper(pub Event, pub EventReceipt);

pub struct EventDispatch {
    pub sender: tokio::sync::broadcast::Sender<EventWrapper>,
}

impl EventDispatch {
    pub fn ephemeral(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);

        Self { sender }
    }

    pub fn subscribe(&mut self) -> Receiver<EventWrapper> {
        self.sender.subscribe()
    }

    pub fn submit_event(&mut self, event: impl Into<Event>) -> Result<EventReceipt> {
        let rcpt = uuid::Uuid::new_v4().into_bytes().to_vec();
        let wrapper = EventWrapper(event.into(), rcpt.clone());

        self.sender.send(wrapper);

        Ok(rcpt)
    }
}

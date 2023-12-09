use anyhow::Result;
use std::collections::VecDeque;

use crate::domain::Event;

pub struct EventDispatch {
    pub entries: VecDeque<Event>,
}

impl EventDispatch {
    pub fn ephemeral() -> Self {
        Self {
            entries: Default::default(),
        }
    }

    pub async fn submit_event(&mut self, event: impl Into<Event>) -> Result<()> {
        self.entries.push_back(event.into());

        Ok(())
    }
}

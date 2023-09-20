use anyhow::Result;
use artemis_core::types::Strategy;
use mev_share::{rpc::SendBundleRequest, sse};

pub struct SimpleArbitrageStrategy {}

impl SimpleArbitrageStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Strategy<Event, Action> for SimpleArbitrageStrategy {
    /// Sync the initial state of the strategy if needed, usually by fetching
    /// onchain data.
    async fn sync_state(&mut self) -> Result<()> {
        Ok(())
    }

    /// Process an event, and return an action if needed.
    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    MEVShareEvent(sse::Event),
}

#[derive(Debug, Clone)]
pub enum Action {
    SubmitBundle(SendBundleRequest),
}

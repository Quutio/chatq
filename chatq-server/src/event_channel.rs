use anyhow::anyhow;
use tokio::sync::broadcast::Receiver;
use tonic::async_trait;
use crate::ports::{MessageEvent, MessageEventChannel, MessageEventChannelError, MessageEventChannelResult};

pub struct BroadcastMessageEventChannel {
    tx: tokio::sync::broadcast::Sender<MessageEvent>,
}

impl BroadcastMessageEventChannel {
    pub fn new(tx: tokio::sync::broadcast::Sender<MessageEvent>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl MessageEventChannel for BroadcastMessageEventChannel {
    fn publish(&self, event: MessageEvent) -> MessageEventChannelResult<()> {
        let _ = self.tx.send(event)
            .map_err(|err| MessageEventChannelError::Arbitrary(anyhow!(err).to_string()))?;
        Ok(())
    }

    async fn subscribe(&self) -> MessageEventChannelResult<Receiver<MessageEvent>> {
        Ok(self.tx.subscribe())
    }
}
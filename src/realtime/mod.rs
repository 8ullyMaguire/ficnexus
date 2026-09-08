pub mod ws;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Deserialize, Serialize};

/// A channel name like "topic:123" or "user:456".
pub type Channel = String;

/// Messages broadcast to channel subscribers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealtimeMessage {
    pub channel: Channel,
    pub event: String,
    pub data: serde_json::Value,
}

/// Manages connected clients and their channel subscriptions.
#[derive(Clone)]
pub struct ConnectionManager {
    channels: Arc<RwLock<HashMap<Channel, broadcast::Sender<RealtimeMessage>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to a channel. Returns a receiver that yields messages.
    pub async fn subscribe(&self, channel: &Channel) -> broadcast::Receiver<RealtimeMessage> {
        let mut channels = self.channels.write().await;
        channels
            .entry(channel.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .subscribe()
    }

    /// Publish a message to a channel.
    pub async fn publish(&self, msg: &RealtimeMessage) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(&msg.channel) {
            let _ = tx.send(msg.clone());
        }
    }

    /// List active channels (for diagnostics).
    pub async fn active_channels(&self) -> Vec<Channel> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

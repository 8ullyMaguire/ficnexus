use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::broadcast;

use super::{ConnectionManager, RealtimeMessage};
use crate::routes::auth::verify_token;
use crate::server::AppState;

#[derive(Deserialize)]
pub struct SseQuery {
    pub token: Option<String>,
    pub channel: Option<String>,
}

/// SSE fallback endpoint for clients that can't use WebSocket.
pub async fn sse_handler(
    Query(query): Query<SseQuery>,
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = if let Some(token) = query.token {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
        if let Ok(claims) = verify_token(&token, &secret) {
            Some(claims.sub)
        } else {
            None
        }
    } else {
        None
    };

    let manager = state.rt_manager.clone();
    let channel = query.channel.unwrap_or_else(|| "global".to_string());

    // Create a merged receiver that gets messages from both the channel
    // and the user's personal channel (if authenticated).
    let mut rx1 = manager.subscribe(&channel).await;
    let mut rx2 = if let Some(uid) = user_id {
        let ch = format!("user:{}", uid);
        Some(manager.subscribe(&ch).await)
    } else {
        None
    };

    // Use a broadcast channel to merge both receivers into one stream.
    let (merged_tx, merged_rx) = broadcast::channel::<RealtimeMessage>(256);
    let merged_tx_clone = merged_tx.clone();

    // Spawn a task that forwards from both receivers to the merged channel.
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(msg) = rx1.recv() => {
                    let _ = merged_tx_clone.send(msg);
                }
                msg = async {
                    match &mut rx2 {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Ok(msg) = msg {
                        let _ = merged_tx_clone.send(msg);
                    }
                }
            }
        }
    });

    Sse::new(SseStream {
        rx: merged_rx,
        _tx: merged_tx,
    })
    .keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("ping"),
    )
}

/// A simple SSE stream backed by a broadcast receiver.
struct SseStream {
    rx: broadcast::Receiver<RealtimeMessage>,
    // Keep tx alive so the channel doesn't close.
    _tx: broadcast::Sender<RealtimeMessage>,
}

impl Stream for SseStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.rx.try_recv() {
            Ok(msg) => {
                let event = Event::default()
                    .event(&msg.event)
                    .id(&msg.channel)
                    .data(serde_json::to_string(&msg.data).unwrap_or_default());
                Poll::Ready(Some(Ok(event)))
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                // Register waker for when a message arrives.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Lagged(_)) => {
                // Skip lagged messages, try again next poll.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => Poll::Ready(None),
        }
    }
}

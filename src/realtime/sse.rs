use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
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
        if let Ok(claims) = verify_token(&token, &state.jwt_secret) {
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

    // Replace the old manual Stream impl (which called cx.waker().wake_by_ref()
    // on every Empty try_recv — a 100%-CPU busy-wait per SSE client) with a
    // proper async stream built on unfold. The closure awaits recv() without
    // spinning, so the task yields to the executor between messages.
    let stream = futures::stream::unfold(merged_rx, |mut rx| async {
        match rx.recv().await {
            Ok(msg) => Some((
                Ok(Event::default()
                    .event(&msg.event)
                    .id(&msg.channel)
                    .data(serde_json::to_string(&msg.data).unwrap_or_default())),
                rx,
            )),
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Skip lagged messages — yield a keepalive and continue.
                Some((Ok(Event::default().event("keepalive").data("")), rx))
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("ping"),
    )
}

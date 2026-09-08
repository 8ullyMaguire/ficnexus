use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{Channel, ConnectionManager, RealtimeMessage};
use crate::routes::auth::verify_token;
use crate::server::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// Upgrade an HTTP request to WebSocket.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
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

    ws.on_upgrade(move |socket| handle_socket(socket, state.rt_manager.clone(), user_id))
}

async fn handle_socket(
    socket: WebSocket,
    manager: ConnectionManager,
    user_id: Option<i32>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Shared subscriptions list for both send and recv tasks.
    let subscriptions: Arc<RwLock<Vec<(Channel, tokio::sync::broadcast::Receiver<RealtimeMessage>)>>> =
        Arc::new(RwLock::new(Vec::new()));

    // If authenticated, auto-subscribe to personal channel.
    if let Some(uid) = user_id {
        let channel = format!("user:{}", uid);
        let rx = manager.subscribe(&channel).await;
        subscriptions.write().await.push((channel, rx));
    }

    let welcome = serde_json::json!({
        "event": "connected",
        "user_id": user_id,
    });
    let _ = sender.send(Message::Text(welcome.to_string().into())).await;

    let subs_send = subscriptions.clone();
    let mut send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
        loop {
            interval.tick().await;
            let mut subs = subs_send.write().await;
            for (_channel, rx) in subs.iter_mut() {
                while let Ok(msg) = rx.try_recv() {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            return;
                        }
                    }
                }
            }
        }
    });

    let manager2 = manager.clone();
    let subs_recv = subscriptions.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                        let event = cmd.get("event").and_then(|e| e.as_str()).unwrap_or("");
                        match event {
                            "subscribe" => {
                                if let Some(channel) = cmd.get("channel").and_then(|c| c.as_str()) {
                                    let ch = channel.to_string();
                                    let rx = manager2.subscribe(&ch).await;
                                    subs_recv.write().await.push((ch, rx));
                                }
                            }
                            "unsubscribe" => {
                                if let Some(channel) = cmd.get("channel").and_then(|c| c.as_str()) {
                                    subs_recv.write().await.retain(|(ch, _)| ch != channel);
                                }
                            }
                            "publish" => {
                                if let (Some(channel), Some(data)) = (
                                    cmd.get("channel").and_then(|c| c.as_str()),
                                    cmd.get("data"),
                                ) {
                                    let msg = RealtimeMessage {
                                        channel: channel.to_string(),
                                        event: "message".to_string(),
                                        data: data.clone(),
                                    };
                                    manager2.publish(&msg).await;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::config::Config;
use crate::models::Claims;

/// Event types broadcast over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    #[serde(rename = "note_created")]
    NoteCreated {
        #[serde(rename = "note-id")]
        note_id: String,
        #[serde(rename = "database-id")]
        database_id: String,
        title: String,
        #[serde(rename = "root-nav-id")]
        root_nav_id: String,
    },
    #[serde(rename = "nav_updated")]
    NavUpdated {
        #[serde(rename = "nav-id")]
        nav_id: String,
        #[serde(rename = "note-id")]
        note_id: String,
        #[serde(rename = "database-id")]
        database_id: String,
        content: String,
    },
}

/// Manages WebSocket connections per account
#[derive(Clone)]
pub struct WsBroadcaster {
    /// Map of account_id -> broadcast sender
    channels: Arc<RwLock<HashMap<i64, broadcast::Sender<String>>>>,
}

impl WsBroadcaster {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for an account
    pub async fn get_sender(&self, account_id: i64) -> broadcast::Sender<String> {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&account_id) {
            return sender.clone();
        }
        drop(channels);

        let mut channels = self.channels.write().await;
        // Double-check after acquiring write lock
        if let Some(sender) = channels.get(&account_id) {
            return sender.clone();
        }
        let (tx, _) = broadcast::channel(64);
        channels.insert(account_id, tx.clone());
        tx
    }

    /// Broadcast an event to all WebSocket connections for an account
    pub async fn broadcast(&self, account_id: i64, event: WsEvent) {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&account_id) {
            let msg = serde_json::to_string(&event).unwrap_or_default();
            // Ignore send errors (no active receivers)
            let _ = sender.send(msg);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// WebSocket upgrade handler - authenticates via query param token
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<super::AppState>,
    Query(query): Query<WsQuery>,
) -> Response {
    let broadcaster = app_state.ws_broadcaster.clone();
    // Validate JWT token
    let config = Config::from_env();
    let token_data = jsonwebtoken::decode::<Claims>(
        &query.token,
        &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    );

    match token_data {
        Ok(data) => {
            let account_id = data.claims.id;
            tracing::info!("WebSocket connected for account {}", account_id);
            ws.on_upgrade(move |socket| handle_socket(socket, broadcaster, account_id))
        }
        Err(e) => {
            tracing::warn!("WebSocket auth failed: {}", e);
            // Return upgrade anyway but close immediately with error
            ws.on_upgrade(|mut socket| async move {
                let _ = socket
                    .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: 4001,
                        reason: "Authentication failed".into(),
                    })))
                    .await;
            })
        }
    }
}

async fn handle_socket(mut socket: WebSocket, broadcaster: WsBroadcaster, account_id: i64) {
    // Send a welcome message
    let welcome = json!({
        "type": "connected",
        "message": "WebSocket connected successfully"
    });
    if socket
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    // Subscribe to broadcast channel for this account
    let sender = broadcaster.get_sender(account_id).await;
    let mut receiver = sender.subscribe();

    loop {
        tokio::select! {
            // Forward broadcast events to the WebSocket client
            Ok(msg) = receiver.recv() => {
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            // Handle incoming messages from the client (mainly for ping/pong)
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Ping(data)) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
            else => break,
        }
    }

    tracing::info!("WebSocket disconnected for account {}", account_id);
}

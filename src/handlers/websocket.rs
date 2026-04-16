use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    debug!("WebSocket client connected");

    loop {
        match rx.recv().await {
            Ok(event) => {
                let json = match serde_json::to_string(&event) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Failed to serialize event: {}", e);
                        continue;
                    }
                };

                if socket.send(Message::Text(json.into())).await.is_err() {
                    debug!("WebSocket client disconnected");
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!("Client lagged, missed {} messages", n);
                // Seguimos, no desconectamos por lag
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                debug!("Broadcast channel closed");
                break;
            }
        }
    }
}

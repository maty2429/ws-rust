use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::models::event::Event;
use crate::AppState;

pub async fn emit_handler(
    State(state): State<Arc<AppState>>,
    Json(event): Json<Event>,
) -> StatusCode {
    debug!("Received event: type={} payload={}", event.event_type, event.payload);

    match state.tx.send(event) {
        Ok(receivers) => {
            debug!("Broadcasted to {} receivers", receivers);
            StatusCode::OK
        }
        Err(err) => {
            warn!("Broadcast send failed: {}", err);
            StatusCode::OK
        }
    }
}

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

    let receivers = state.publish(event).await;
    if receivers == 0 {
        warn!("No active WebSocket receivers; snapshot retained if applicable");
    } else {
        debug!("Broadcasted to {} receivers", receivers);
    }

    StatusCode::OK
}

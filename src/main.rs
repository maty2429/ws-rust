use axum::{Router, routing::{get, post}};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod handlers;
mod models;

use handlers::emit::emit_handler;
use handlers::websocket::ws_handler;
use models::event::Event;

pub struct AppState {
    pub tx: broadcast::Sender<Event>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "websocket=info".into()),
        )
        .init();

    let (tx, _) = broadcast::channel::<Event>(100);

    let state = Arc::new(AppState { tx });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/emit", post(emit_handler))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3001";
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

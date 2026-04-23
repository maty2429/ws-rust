use axum::{Router, routing::{get, post}, http::StatusCode};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tonic::transport::Server;

mod handlers;
mod grpc;
mod models;

use handlers::emit::emit_handler;
use handlers::grpc::EventBridgeService;
use handlers::websocket::ws_handler;
use grpc::eventbridge::event_bridge_server::EventBridgeServer;
use models::event::Event;

pub struct AppState {
    pub tx: broadcast::Sender<Event>,
    latest_dashboard_tickets_por_estado: RwLock<Option<Event>>,
}

impl AppState {
    pub async fn publish(&self, event: Event) -> usize {
        self.remember_snapshot(&event).await;
        match self.tx.send(event) {
            Ok(receivers) => receivers,
            Err(_) => 0,
        }
    }

    pub async fn latest_dashboard_tickets_por_estado(&self) -> Option<Event> {
        self.latest_dashboard_tickets_por_estado.read().await.clone()
    }

    async fn remember_snapshot(&self, event: &Event) {
        if event.event_type != "dashboard.tickets_por_estado.updated" {
            return;
        }

        let mut guard = self.latest_dashboard_tickets_por_estado.write().await;
        *guard = Some(event.clone());
    }
}

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3001";
const DEFAULT_GRPC_BIND_ADDR: &str = "0.0.0.0:50051";
const DEFAULT_BROADCAST_CAPACITY: usize = 4096;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "websocket=info".into()),
        )
        .init();

    let broadcast_capacity = std::env::var("WS_BROADCAST_CAPACITY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_BROADCAST_CAPACITY);

    let (tx, _) = broadcast::channel::<Event>(broadcast_capacity);

    let state = Arc::new(AppState {
        tx,
        latest_dashboard_tickets_por_estado: RwLock::new(None),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/emit", post(emit_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(state.clone());

    let addr = std::env::var("WS_BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    let grpc_addr = std::env::var("WS_GRPC_BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_GRPC_BIND_ADDR.to_string());
    info!(
        "Listening on {} and gRPC {} with broadcast capacity {}",
        addr, grpc_addr, broadcast_capacity
    );

    let grpc_state = state.clone();
    let grpc_addr_parsed = grpc_addr.parse().expect("invalid gRPC bind address");
    tokio::spawn(async move {
        let service = EventBridgeService::new(grpc_state);
        if let Err(err) = Server::builder()
            .add_service(EventBridgeServer::new(service))
            .serve(grpc_addr_parsed)
            .await
        {
            error!("gRPC server failed: {}", err);
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> StatusCode {
    StatusCode::OK
}

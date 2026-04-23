use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::debug;

use crate::{
    AppState,
    grpc::eventbridge::{
        PublishEventRequest, PublishEventResponse,
        event_bridge_server::EventBridge,
    },
    models::event::Event,
};

pub struct EventBridgeService {
    state: Arc<AppState>,
}

impl EventBridgeService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl EventBridge for EventBridgeService {
    async fn publish_event(
        &self,
        request: Request<PublishEventRequest>,
    ) -> Result<Response<PublishEventResponse>, Status> {
        let request = request.into_inner();

        let payload = serde_json::from_str(&request.payload_json)
            .map_err(|err| Status::invalid_argument(format!("invalid payload_json: {err}")))?;

        let event = Event {
            event_type: request.event_type,
            payload,
            timestamp: request.timestamp,
        };

        debug!(
            "Received gRPC event: type={} payload={}",
            event.event_type, event.payload
        );

        self.state.publish(event).await;

        Ok(Response::new(PublishEventResponse { accepted: true }))
    }
}

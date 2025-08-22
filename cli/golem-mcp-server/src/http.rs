//! HTTP transport implementation for MCP over HTTP
//!
//! Provides HTTP transport using axum for serving MCP requests over HTTP.

use crate::handlers::MessageHandlerImpl;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::JsonRpcMessage,
    serialization::{MessageBuilder, MessageSerializer},
    transport::{HttpTransportConfig, MessageHandler},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::{debug, error, info};

/// HTTP transport for MCP protocol
#[derive(Debug)]
pub struct HttpTransport {
    /// Transport configuration
    config: HttpTransportConfig,
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    /// Message handler
    message_handler: Arc<MessageHandlerImpl>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if the configuration is invalid.
    pub async fn new(config: HttpTransportConfig) -> McpResult<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Start serving HTTP requests
    ///
    /// # Errors
    ///
    /// Returns `McpError::Transport` if the server cannot start.
    pub async fn serve(self, message_handler: Arc<MessageHandlerImpl>) -> McpResult<()> {
        let state = AppState { message_handler };

        let app = self.create_app(state);
        let addr = format!("{}:{}", self.config.host, self.config.port);

        info!("Starting HTTP transport on {}", addr);

        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            McpError::transport("http".to_owned(), format!("Failed to bind to {addr}: {e}"))
        })?;

        axum::serve(listener, app)
            .await
            .map_err(|e| McpError::transport("http".to_owned(), format!("HTTP server error: {e}")))
    }

    /// Create the axum application
    fn create_app(&self, state: AppState) -> Router {
        let timeout_duration = std::time::Duration::from_millis(self.config.request_timeout_ms);

        Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/health", axum::routing::get(health_check))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()) // TODO: Configure CORS properly
                    .layer(TimeoutLayer::new(timeout_duration))
                    .into_inner(),
            )
            .with_state(state)
    }
}

/// Handle MCP JSON-RPC requests
async fn handle_mcp_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    debug!("Received MCP request");

    // Validate content type
    if let Some(content_type) = headers.get("content-type") {
        if !content_type
            .to_str()
            .unwrap_or("")
            .contains("application/json")
        {
            return Err((
                StatusCode::BAD_REQUEST,
                "Content-Type must be application/json".to_owned(),
            ));
        }
    }

    // Parse JSON-RPC message
    let message = match MessageSerializer::deserialize_message(&body.to_string()) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Failed to parse JSON-RPC message: {}", e);
            let error_response = MessageBuilder::error_response(
                None::<String>,
                &McpError::serialization(
                    "deserialize".to_owned(),
                    format!("Invalid JSON-RPC message: {e}"),
                ),
            );
            let json = serialize_response(&error_response)?;
            return Ok(Json(json));
        }
    };

    // Handle the message
    let response = match state.message_handler.handle_message(message).await {
        Ok(Some(response)) => response,
        Ok(None) => {
            // Notification - no response expected
            return Ok(Json(serde_json::json!({})));
        }
        Err(e) => {
            error!("Error handling message: {}", e);
            MessageBuilder::error_response(None::<String>, &e)
        }
    };

    let json = serialize_response(&response)?;
    Ok(Json(json))
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Serialize response to JSON value
fn serialize_response(
    response: &JsonRpcMessage,
) -> Result<serde_json::Value, (StatusCode, String)> {
    match MessageSerializer::serialize_message(response) {
        Ok(json_str) => match serde_json::from_str(&json_str) {
            Ok(json_value) => Ok(json_value),
            Err(e) => {
                error!("Failed to parse serialized response: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to serialize response".to_owned(),
                ))
            }
        },
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize response".to_owned(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::McpServerConfig;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check() {
        let config = McpServerConfig::default();
        let message_handler = Arc::new(MessageHandlerImpl::new(config).await.expect("Message handler should initialize"));
        let state = AppState { message_handler };

        let transport = HttpTransport::new(HttpTransportConfig::default())
            .await
            .expect("Transport should initialize");
        let app = transport.create_app(state);
        let server = TestServer::new(app).expect("Test server should start");

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "healthy");
    }

    #[tokio::test]
    async fn test_mcp_request_invalid_json() {
        let config = McpServerConfig::default();
        let message_handler = Arc::new(MessageHandlerImpl::new(config).await.expect("Message handler should initialize"));
        let state = AppState { message_handler };

        let transport = HttpTransport::new(HttpTransportConfig::default())
            .await
            .expect("Transport should initialize");
        let app = transport.create_app(state);
        let server = TestServer::new(app).expect("Test server should start");

        let response = server
            .post("/mcp")
            .json(&serde_json::json!({"invalid": "json-rpc"}))
            .await;

        assert_eq!(response.status_code(), 200); // Should return JSON-RPC error, not HTTP error

        let body: serde_json::Value = response.json();
        assert!(body.get("error").is_some());
    }
}

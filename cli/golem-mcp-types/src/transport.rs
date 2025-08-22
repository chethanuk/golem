//! Transport layer abstractions for MCP communications
//!
//! Provides transport-agnostic interfaces for MCP message handling,
//! following the 2025-06-18 specification which standardizes on HTTP transport.

use crate::error::{McpError, McpResult};
use crate::protocol::JsonRpcMessage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport layer abstraction for MCP message handling
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message through the transport
    ///
    /// # Errors
    ///
    /// Returns `McpError::Transport` if the message cannot be sent due to
    /// transport-specific issues (network errors, connection failures, etc.)
    async fn send(&mut self, message: &JsonRpcMessage) -> McpResult<()>;

    /// Receive a message from the transport
    ///
    /// # Errors
    ///
    /// Returns `McpError::Transport` if message reception fails or times out.
    /// Returns `McpError::Serialization` if the received data cannot be parsed.
    async fn receive(&mut self) -> McpResult<JsonRpcMessage>;

    /// Close the transport connection gracefully
    ///
    /// # Errors
    ///
    /// Returns `McpError::Transport` if the connection cannot be closed cleanly.
    async fn close(&mut self) -> McpResult<()>;

    /// Check if the transport connection is still active
    fn is_connected(&self) -> bool;

    /// Get transport-specific metadata
    fn metadata(&self) -> &HashMap<String, String>;
}

/// HTTP transport configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpTransportConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Use TLS/HTTPS
    pub use_tls: bool,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// HTTP headers to include with requests
    pub headers: HashMap<String, String>,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            port: 3000,
            use_tls: false,
            timeout_ms: 30_000,
            request_timeout_ms: 60_000,
            max_request_size: 1_024 * 1_024, // 1MB
            headers: HashMap::new(),
        }
    }
}

impl HttpTransportConfig {
    /// Create a new HTTP transport configuration
    #[must_use]
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            ..Self::default()
        }
    }

    /// Enable TLS for the transport
    #[must_use]
    pub fn with_tls(mut self) -> Self {
        self.use_tls = true;
        self
    }

    /// Set connection timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set request timeout
    #[must_use]
    pub const fn with_request_timeout(mut self, timeout_ms: u64) -> Self {
        self.request_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum request size
    #[must_use]
    pub const fn with_max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = size;
        self
    }

    /// Add an HTTP header
    #[must_use]
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Get the base URL for the transport
    #[must_use]
    pub fn base_url(&self) -> String {
        let scheme = if self.use_tls { "https" } else { "http" };
        format!("{scheme}://{}:{}", self.host, self.port)
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if the configuration is invalid.
    pub fn validate(&self) -> McpResult<()> {
        if self.host.is_empty() {
            return Err(McpError::configuration(
                "host".to_owned(),
                "host cannot be empty".to_owned(),
            ));
        }

        if self.port == 0 {
            return Err(McpError::configuration(
                "port".to_owned(),
                "port must be greater than 0".to_owned(),
            ));
        }

        if self.timeout_ms == 0 {
            return Err(McpError::configuration(
                "timeout_ms".to_owned(),
                "timeout must be greater than 0".to_owned(),
            ));
        }

        if self.request_timeout_ms == 0 {
            return Err(McpError::configuration(
                "request_timeout_ms".to_owned(),
                "request timeout must be greater than 0".to_owned(),
            ));
        }

        if self.max_request_size == 0 {
            return Err(McpError::configuration(
                "max_request_size".to_owned(),
                "max request size must be greater than 0".to_owned(),
            ));
        }

        Ok(())
    }
}

/// Transport factory for creating transport instances
pub struct TransportFactory;

impl TransportFactory {
    /// Validate HTTP transport configuration
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if the configuration is invalid.
    pub fn validate_http_config(config: &HttpTransportConfig) -> McpResult<()> {
        config.validate()
    }
}

/// Message handler trait for processing received messages
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message and return a response if needed
    ///
    /// # Errors
    ///
    /// Returns appropriate `McpError` variants based on the specific handling failure.
    fn handle_message(&self, message: JsonRpcMessage) -> impl std::future::Future<Output = McpResult<Option<JsonRpcMessage>>> + Send;
}

/// Connection state for tracking transport connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is active and ready
    Connected,
    /// Connection is being closed
    Closing,
    /// Connection is closed
    Closed,
    /// Connection failed
    Failed,
}

/// Connection metadata
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Current connection state
    pub state: ConnectionState,
    /// Connection start time
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last activity time
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of messages sent
    pub messages_sent: u64,
    /// Number of messages received
    pub messages_received: u64,
    /// Transport-specific metadata
    pub metadata: HashMap<String, String>,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            state: ConnectionState::Closed,
            connected_at: None,
            last_activity: None,
            messages_sent: 0,
            messages_received: 0,
            metadata: HashMap::new(),
        }
    }
}

impl ConnectionInfo {
    /// Create new connection info
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark connection as connected
    pub fn mark_connected(&mut self) {
        self.state = ConnectionState::Connected;
        let now = chrono::Utc::now();
        self.connected_at = Some(now);
        self.last_activity = Some(now);
    }

    /// Record a sent message
    pub fn record_sent_message(&mut self) {
        self.messages_sent += 1;
        self.last_activity = Some(chrono::Utc::now());
    }

    /// Record a received message
    pub fn record_received_message(&mut self) {
        self.messages_received += 1;
        self.last_activity = Some(chrono::Utc::now());
    }

    /// Mark connection as closed
    pub fn mark_closed(&mut self) {
        self.state = ConnectionState::Closed;
    }

    /// Mark connection as failed
    pub fn mark_failed(&mut self) {
        self.state = ConnectionState::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_config_creation() {
        let config = HttpTransportConfig::new("example.com".to_owned(), 8080);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 8080);
        assert!(!config.use_tls);
    }

    #[test]
    fn test_http_transport_config_with_tls() {
        let config = HttpTransportConfig::new("example.com".to_owned(), 443).with_tls();
        assert!(config.use_tls);
        assert_eq!(config.base_url(), "https://example.com:443");
    }

    #[test]
    fn test_http_transport_config_validation() {
        let valid_config = HttpTransportConfig::new("example.com".to_owned(), 8080);
        assert!(valid_config.validate().is_ok());

        let invalid_config = HttpTransportConfig::new("".to_owned(), 0);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_connection_info_lifecycle() {
        let mut info = ConnectionInfo::new();
        assert_eq!(info.state, ConnectionState::Closed);

        info.mark_connected();
        assert_eq!(info.state, ConnectionState::Connected);
        assert!(info.connected_at.is_some());

        info.record_sent_message();
        assert_eq!(info.messages_sent, 1);

        info.record_received_message();
        assert_eq!(info.messages_received, 1);

        info.mark_closed();
        assert_eq!(info.state, ConnectionState::Closed);
    }
}

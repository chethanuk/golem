//! Core MCP server implementation
//!
//! Provides the main server implementation that coordinates all MCP functionality.

use crate::{
    config::McpServerConfig,
    handlers::MessageHandlerImpl,
    http::HttpTransport,
    session::SessionManager,
};
use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::{McpCapabilities, McpServerInfo},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Main MCP server implementation
pub struct McpServer {
    /// Server configuration
    config: McpServerConfig,
    /// Session manager
    session_manager: Arc<RwLock<SessionManager>>,
    /// Message handler
    message_handler: Arc<MessageHandlerImpl>,
    /// HTTP transport (when using HTTP)
    http_transport: Option<HttpTransport>,
}

impl McpServer {
    /// Create a new MCP server with the given configuration
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if the configuration is invalid.
    /// Returns `McpError::Internal` if server initialization fails.
    pub async fn new(config: McpServerConfig) -> McpResult<Self> {
        // Validate configuration
        config.validate()?;

        info!("Initializing MCP server: {}", config.server_info.name);

        // Create session manager
        let session_manager = Arc::new(RwLock::new(SessionManager::new()));

        // Create message handler
        let message_handler = Arc::new(MessageHandlerImpl::new(config.clone()).await?);

        // Create HTTP transport if configured
        let http_transport = if config.http.port > 0 {
            Some(HttpTransport::new(config.http.clone()).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            session_manager,
            message_handler,
            http_transport,
        })
    }

    /// Start serving MCP requests
    ///
    /// # Errors
    ///
    /// Returns `McpError::Transport` if the server cannot start listening.
    pub async fn serve(self) -> McpResult<()> {
        info!("Starting MCP server on {}", self.config.bind_address());

        if let Some(http_transport) = self.http_transport {
            http_transport.serve(self.message_handler.clone()).await
        } else {
            Err(McpError::configuration(
                "transport".to_owned(),
                "No transport configured".to_owned(),
            ))
        }
    }

    /// Get server information
    #[must_use]
    pub fn server_info(&self) -> &McpServerInfo {
        &self.config.server_info
    }

    /// Get server capabilities
    #[must_use]
    pub fn capabilities(&self) -> McpCapabilities {
        // This would be built based on available tools and resources
        McpCapabilities {
            tools: Some(golem_mcp_types::protocol::ToolsCapability {
                list_changed: true,
            }),
            resources: Some(golem_mcp_types::protocol::ResourcesCapability {
                subscribe: false,
                list_changed: true,
            }),
            prompts: None,
            logging: Some(golem_mcp_types::protocol::LoggingCapability {
                levels: vec![
                    golem_mcp_types::protocol::LogLevel::Debug,
                    golem_mcp_types::protocol::LogLevel::Info,
                    golem_mcp_types::protocol::LogLevel::Warning,
                    golem_mcp_types::protocol::LogLevel::Error,
                ],
            }),
        }
    }

    /// Gracefully shutdown the server
    pub async fn shutdown(&self) -> McpResult<()> {
        info!("Shutting down MCP server");
        
        // Close all active sessions
        {
            let mut session_manager = self.session_manager.write().await;
            session_manager.close_all_sessions().await?;
        }

        info!("MCP server shutdown complete");
        Ok(())
    }
}
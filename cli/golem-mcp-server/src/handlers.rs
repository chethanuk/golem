//! JSON-RPC method handlers for MCP protocol messages
//!
//! Implements all MCP protocol handlers including initialization, tool execution,
//! and resource access.

use crate::config::McpServerConfig;
use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::{
        JsonRpcMessage, JsonRpcRequest, InitializeParams, InitializeResult,
        McpCapabilities, ToolsCapability, ResourcesCapability, LoggingCapability, LogLevel,
        Tool, Resource,
    },
    transport::MessageHandler,
    serialization::MessageBuilder,
};
use std::sync::Arc;
use tracing::{debug, error, instrument};

/// Message handler implementation for MCP protocol
pub struct MessageHandlerImpl {
    /// Server configuration
    config: McpServerConfig,
    /// Tool registry
    _tool_registry: Arc<golem_mcp_tools::ToolRegistry>,
    /// Resource provider registry
    _resource_registry: Arc<golem_mcp_resources::ResourceProviderRegistry>,
}

impl MessageHandlerImpl {
    /// Create a new message handler
    ///
    /// # Errors
    ///
    /// Returns `McpError::Internal` if initialization fails.
    pub async fn new(config: McpServerConfig) -> McpResult<Self> {
        let tool_registry = Arc::new(golem_mcp_tools::ToolRegistry::new());
        let resource_registry = Arc::new(golem_mcp_resources::ResourceProviderRegistry::new());

        Ok(Self {
            config,
            _tool_registry: tool_registry,
            _resource_registry: resource_registry,
        })
    }
}

impl MessageHandler for MessageHandlerImpl {
    #[instrument(skip(self))]
    async fn handle_message(&self, message: JsonRpcMessage) -> McpResult<Option<JsonRpcMessage>> {
        match message {
            JsonRpcMessage::Request(request) => {
                debug!("Handling request: {}", request.method);
                let response = self.handle_request(request).await?;
                Ok(Some(response))
            }
            JsonRpcMessage::Notification(notification) => {
                debug!("Handling notification: {}", notification.method);
                self.handle_notification(notification).await?;
                Ok(None) // Notifications don't expect responses
            }
            JsonRpcMessage::Response(_) | JsonRpcMessage::Error(_) => {
                // Servers don't typically handle responses/errors from clients
                Err(McpError::invalid_request(
                    "Server received unexpected response/error message".to_owned(),
                ))
            }
        }
    }
}

impl MessageHandlerImpl {
    /// Handle a JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,
            _ => Ok(MessageBuilder::error_response(
                Some(request.id),
                &McpError::json_rpc(
                    -32_601, // Method not found
                    format!("Method not found: {}", request.method),
                ),
            )),
        }
    }

    /// Handle a JSON-RPC notification
    async fn handle_notification(
        &self,
        notification: golem_mcp_types::protocol::JsonRpcNotification,
    ) -> McpResult<()> {
        match notification.method.as_str() {
            "initialized" => {
                debug!("Client initialized");
                Ok(())
            }
            "notifications/cancelled" => {
                debug!("Client cancelled operation");
                Ok(())
            }
            _ => {
                debug!("Unknown notification: {}", notification.method);
                Ok(())
            }
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        let params: InitializeParams = if let Some(params) = request.params {
            golem_mcp_types::serialization::MessageSerializer::deserialize(&params)?
        } else {
            return Ok(MessageBuilder::error_response(
                Some(request.id),
                &McpError::invalid_request("Initialize request missing parameters".to_owned()),
            ));
        };

        debug!("Initialize request from client: {}", params.client_info.name);

        // Validate protocol version
        if params.protocol_version != golem_mcp_types::protocol::MCP_VERSION {
            return Ok(MessageBuilder::error_response(
                Some(request.id),
                &McpError::protocol_version_mismatch(
                    golem_mcp_types::protocol::MCP_VERSION.to_owned(),
                    params.protocol_version,
                ),
            ));
        }

        let result = InitializeResult {
            protocol_version: golem_mcp_types::protocol::MCP_VERSION.to_owned(),
            capabilities: McpCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: true,
                }),
                resources: Some(ResourcesCapability {
                    subscribe: false,
                    list_changed: true,
                }),
                prompts: None,
                logging: Some(LoggingCapability {
                    levels: vec![
                        LogLevel::Debug,
                        LogLevel::Info,
                        LogLevel::Warning,
                        LogLevel::Error,
                    ],
                }),
            },
            server_info: self.config.server_info.clone(),
            instructions: Some(
                "Golem MCP Server provides access to Golem CLI functionality via MCP protocol."
                    .to_owned(),
            ),
        };

        MessageBuilder::response_with_result(request.id, &result)
            .map_err(|e| {
                error!("Failed to serialize initialize result: {}", e);
                e
            })
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Listing available tools");
        
        // This would integrate with the tool registry
        let tools: Vec<Tool> = vec![]; // Placeholder - will be populated by tool registry
        
        MessageBuilder::response_with_result(request.id, &tools)
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Executing tool call");
        
        // This would parse the tool call parameters and execute the tool
        let result = serde_json::json!({
            "success": true,
            "result": "Tool execution placeholder"
        });
        
        MessageBuilder::response_with_result(request.id, &result)
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Listing available resources");
        
        // This would integrate with the resource provider registry
        let resources: Vec<Resource> = vec![]; // Placeholder - will be populated by resource provider registry
        
        MessageBuilder::response_with_result(request.id, &resources)
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Reading resource");
        
        // This would read the requested resource
        let content = serde_json::json!({
            "contents": [{
                "uri": "placeholder",
                "mimeType": "text/plain",
                "text": "Resource content placeholder"
            }]
        });
        
        MessageBuilder::response_with_result(request.id, &content)
    }
}
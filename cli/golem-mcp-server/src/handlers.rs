//! JSON-RPC method handlers for MCP protocol messages
//!
//! Implements all MCP protocol handlers including initialization, tool execution,
//! and resource access.

use crate::config::McpServerConfig;
use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::{
        InitializeParams, InitializeResult, JsonRpcMessage, JsonRpcRequest, LogLevel,
        LoggingCapability, McpCapabilities, ResourcesCapability, ToolsCapability,
    },
    serialization::MessageBuilder,
    transport::MessageHandler,
};
use std::sync::Arc;
use tracing::{debug, error, instrument};

/// Message handler implementation for MCP protocol
pub struct MessageHandlerImpl {
    /// Server configuration
    config: McpServerConfig,
    /// Tool registry
    tool_registry: Arc<golem_mcp_tools::ToolRegistry>,
    /// Resource provider registry
    resource_registry: Arc<golem_mcp_resources::ResourceProviderRegistry>,
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
            tool_registry,
            resource_registry,
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

        debug!(
            "Initialize request from client: {}",
            params.client_info.name
        );

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
                tools: Some(ToolsCapability { list_changed: true }),
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

        MessageBuilder::response_with_result(request.id, &result).map_err(|e| {
            error!("Failed to serialize initialize result: {}", e);
            e
        })
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Listing available tools");

        // Get tools from registry
        let tools = self.tool_registry.list_tools();

        MessageBuilder::response_with_result(request.id, &tools)
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Executing tool call");

        // Parse tool call parameters
        let params: serde_json::Value = request.params.unwrap_or_default();
        let tool_name = params["name"].as_str().unwrap_or("unknown");
        let arguments = &params["arguments"];

        debug!("Tool call: {} with args: {}", tool_name, arguments);

        // For now, return a placeholder response
        // TODO: In Phase 4, this will call the rmcp server's tool methods
        let execution_result = serde_json::json!({
            "success": true,
            "result": {
                "tool": tool_name,
                "status": "executed",
                "message": format!("Tool {} would be executed with arguments", tool_name),
                "arguments": arguments
            }
        });

        MessageBuilder::response_with_result(request.id, &execution_result)
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Listing available resources");

        // Get resources from the resource provider registry
        let resources = self.resource_registry.list_resources().await;

        MessageBuilder::response_with_result(request.id, &resources)
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, request: JsonRpcRequest) -> McpResult<JsonRpcMessage> {
        debug!("Reading resource");

        // Parse resource URI from request
        let params: serde_json::Value = request.params.unwrap_or_default();
        let uri = params["uri"].as_str().unwrap_or("unknown");

        debug!("Reading resource: {}", uri);

        // Read the resource via resource provider registry
        let content = match self.resource_registry.read_resource_async(uri).await {
            Ok(resource_content) => resource_content,
            Err(e) => serde_json::json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": format!("Error reading resource: {}", e)
                }]
            })
        };

        MessageBuilder::response_with_result(request.id, &content)
    }
}

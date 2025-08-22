//! MCP protocol message types and structures
//!
//! Implements the complete Model Context Protocol message format based on
//! the 2025-06-18 specification, including JSON-RPC envelope and MCP-specific content.

use crate::error::McpError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP protocol version string
pub const MCP_VERSION: &str = "2025-06-18";

/// JSON-RPC version string
pub const JSON_RPC_VERSION: &str = "2.0";

/// JSON-RPC message envelope for all MCP communications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// Request message
    Request(JsonRpcRequest),
    /// Response message (success)
    Response(JsonRpcResponse),
    /// Error response message
    Error(JsonRpcErrorResponse),
    /// Notification message (no response expected)
    Notification(JsonRpcNotification),
}

/// JSON-RPC request message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request identifier
    pub id: JsonRpcId,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC response message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request identifier this responds to
    pub id: JsonRpcId,
    /// Response result
    pub result: serde_json::Value,
}

/// JSON-RPC error response message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request identifier this responds to
    pub id: Option<JsonRpcId>,
    /// Error details
    pub error: JsonRpcError,
}

/// JSON-RPC notification message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC error object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC identifier type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// String identifier
    String(String),
    /// Numeric identifier
    Number(i64),
}

/// MCP server information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server implementation details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<McpImplementation>,
}

/// MCP client information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
    /// Client implementation details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<McpImplementation>,
}

/// Implementation details for MCP server or client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpImplementation {
    /// Implementation name (e.g., "golem-mcp-server")
    pub name: String,
    /// Implementation version
    pub version: String,
    /// Additional implementation metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// MCP protocol version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpVersion {
    /// Protocol version string
    pub version: String,
}

/// MCP server capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpCapabilities {
    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Prompts capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Logging capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
}

/// Tools capability specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether tools can be listed
    #[serde(default = "default_true")]
    pub list_changed: bool,
}

/// Resources capability specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether resources support subscription to changes
    #[serde(default = "default_false")]
    pub subscribe: bool,
    /// Whether the server supports resource list changes
    #[serde(default = "default_true")]
    pub list_changed: bool,
}

/// Prompts capability specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether prompts can be listed
    #[serde(default = "default_true")]
    pub list_changed: bool,
}

/// Logging capability specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingCapability {
    /// Supported logging levels
    pub levels: Vec<LogLevel>,
}

/// MCP logging levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Notice level
    Notice,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
    /// Alert level
    Alert,
    /// Emergency level
    Emergency,
}

/// Initialize request parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: McpCapabilities,
    /// Client information
    #[serde(rename = "clientInfo")]
    pub client_info: McpClientInfo,
}

/// Initialize response result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: McpCapabilities,
    /// Server information
    #[serde(rename = "serverInfo")]
    pub server_info: McpServerInfo,
    /// Additional server instructions or information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Tool definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool input schema
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    /// Additional tool metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Resource definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Resource MIME type
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Additional resource metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    #[must_use]
    pub fn new(id: JsonRpcId, method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            method,
            params,
        }
    }
}

impl JsonRpcResponse {
    /// Create a new JSON-RPC response
    #[must_use]
    pub fn new(id: JsonRpcId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            result,
        }
    }
}

impl JsonRpcErrorResponse {
    /// Create a new JSON-RPC error response
    #[must_use]
    pub fn new(id: Option<JsonRpcId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            error,
        }
    }

    /// Create an error response from McpError
    #[must_use]
    pub fn from_mcp_error(id: Option<JsonRpcId>, error: &McpError) -> Self {
        let json_error = JsonRpcError {
            code: error.code(),
            message: error.to_string(),
            data: None, // Could be expanded to include error-specific data
        };
        Self::new(id, json_error)
    }
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    #[must_use]
    pub fn new(method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            method,
            params,
        }
    }
}

impl From<String> for JsonRpcId {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for JsonRpcId {
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl From<i64> for JsonRpcId {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl std::fmt::Display for JsonRpcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}

/// Default true value for capability fields
const fn default_true() -> bool {
    true
}

/// Default false value for capability fields
const fn default_false() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_creation() {
        let request = JsonRpcRequest::new(
            JsonRpcId::from("test-id"),
            "initialize".to_owned(),
            Some(serde_json::json!({"test": "value"})),
        );

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, JsonRpcId::String("test-id".to_owned()));
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_json_rpc_response_creation() {
        let response = JsonRpcResponse::new(
            JsonRpcId::from(42),
            serde_json::json!({"result": "success"}),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, JsonRpcId::Number(42));
    }

    #[test]
    fn test_error_response_from_mcp_error() {
        let mcp_error = McpError::authorization("invalid token".to_owned());
        let error_response =
            JsonRpcErrorResponse::from_mcp_error(Some(JsonRpcId::from("test-id")), &mcp_error);

        assert_eq!(error_response.error.code, -32_601);
        assert!(error_response
            .error
            .message
            .contains("Authorization failed"));
    }

    #[test]
    fn test_mcp_capabilities_serialization() {
        let capabilities = McpCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: true,
            }),
            prompts: None,
            logging: Some(LoggingCapability {
                levels: vec![LogLevel::Error, LogLevel::Warning, LogLevel::Info],
            }),
        };

        let json = serde_json::to_string(&capabilities).expect("should serialize");
        let deserialized: McpCapabilities =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(capabilities, deserialized);
    }
}

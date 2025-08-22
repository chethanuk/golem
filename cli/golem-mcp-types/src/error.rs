//! Error types for MCP operations
//!
//! Provides a comprehensive error hierarchy using structured enums and `thiserror`
//! for context preservation and proper error propagation throughout the MCP stack.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;

/// Comprehensive error types for MCP operations
///
/// This enum provides structured error handling covering all aspects of MCP
/// server operations, from protocol violations to resource access failures.
/// Each variant includes rich context for debugging and client communication.
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum McpError {
    /// JSON-RPC protocol errors
    #[error("JSON-RPC protocol error: {message}")]
    JsonRpc {
        /// Error code as defined in JSON-RPC specification
        code: i32,
        /// Human-readable error message
        message: String,
        /// Additional error data
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },

    /// MCP protocol version mismatch
    #[error("MCP protocol version mismatch: server supports {server_version}, client requested {client_version}")]
    ProtocolVersionMismatch {
        /// Version supported by the server
        server_version: String,
        /// Version requested by the client
        client_version: String,
    },

    /// Invalid request format or content
    #[error("Invalid request: {reason}")]
    InvalidRequest {
        /// Reason for request invalidity
        reason: String,
        /// The invalid request data for debugging
        #[serde(skip_serializing_if = "Option::is_none")]
        request_data: Option<serde_json::Value>,
    },

    /// Authentication and authorization errors
    #[error("Authorization failed: {reason}")]
    Authorization {
        /// Specific authorization failure reason
        reason: String,
        /// Required permissions or scope
        #[serde(skip_serializing_if = "Option::is_none")]
        required_permissions: Option<Vec<String>>,
    },

    /// Resource access errors
    #[error("Resource access error: {resource_type}:{resource_id} - {reason}")]
    ResourceAccess {
        /// Type of resource (e.g., "manifest", "component")
        resource_type: String,
        /// Unique identifier for the resource
        resource_id: String,
        /// Reason for access failure
        reason: String,
    },

    /// Tool execution errors
    #[error("Tool execution error: {tool_name} - {reason}")]
    ToolExecution {
        /// Name of the tool that failed
        tool_name: String,
        /// Reason for execution failure
        reason: String,
        /// Exit code if available
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        /// Standard error output
        #[serde(skip_serializing_if = "Option::is_none")]
        stderr: Option<String>,
    },

    /// Transport layer errors
    #[error("Transport error: {transport_type} - {reason}")]
    Transport {
        /// Type of transport (e.g., "http", "stdio")
        transport_type: String,
        /// Reason for transport failure
        reason: String,
        /// HTTP status code if applicable
        #[serde(skip_serializing_if = "Option::is_none")]
        status_code: Option<u16>,
    },

    /// Serialization and deserialization errors
    #[error("Serialization error: {operation} - {reason}")]
    Serialization {
        /// Operation that failed (serialize/deserialize)
        operation: String,
        /// Reason for serialization failure
        reason: String,
        /// Type being serialized/deserialized
        #[serde(skip_serializing_if = "Option::is_none")]
        type_name: Option<String>,
    },

    /// Configuration errors
    #[error("Configuration error: {key} - {reason}")]
    Configuration {
        /// Configuration key that caused the error
        key: String,
        /// Reason for configuration error
        reason: String,
        /// Expected value format or type
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_format: Option<String>,
    },

    /// Internal server errors
    #[error("Internal server error: {reason}")]
    Internal {
        /// Reason for internal error
        reason: String,
        /// Additional context for debugging
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<serde_json::Value>,
    },
}

impl McpError {
    /// Create a JSON-RPC error with standard error codes
    #[must_use]
    pub const fn json_rpc(code: i32, message: String) -> Self {
        Self::JsonRpc {
            code,
            message,
            data: None,
        }
    }

    /// Create a JSON-RPC error with additional data
    #[must_use]
    pub const fn json_rpc_with_data(code: i32, message: String, data: serde_json::Value) -> Self {
        Self::JsonRpc {
            code,
            message,
            data: Some(data),
        }
    }

    /// Create a protocol version mismatch error
    #[must_use]
    pub fn protocol_version_mismatch(server_version: String, client_version: String) -> Self {
        Self::ProtocolVersionMismatch {
            server_version,
            client_version,
        }
    }

    /// Create an invalid request error
    #[must_use]
    pub fn invalid_request(reason: String) -> Self {
        Self::InvalidRequest {
            reason,
            request_data: None,
        }
    }

    /// Create an invalid request error with request data
    #[must_use]
    pub fn invalid_request_with_data(reason: String, request_data: serde_json::Value) -> Self {
        Self::InvalidRequest {
            reason,
            request_data: Some(request_data),
        }
    }

    /// Create an authorization error
    #[must_use]
    pub fn authorization(reason: String) -> Self {
        Self::Authorization {
            reason,
            required_permissions: None,
        }
    }

    /// Create a resource access error
    #[must_use]
    pub fn resource_access(resource_type: String, resource_id: String, reason: String) -> Self {
        Self::ResourceAccess {
            resource_type,
            resource_id,
            reason,
        }
    }

    /// Create a tool execution error
    #[must_use]
    pub fn tool_execution(tool_name: String, reason: String) -> Self {
        Self::ToolExecution {
            tool_name,
            reason,
            exit_code: None,
            stderr: None,
        }
    }

    /// Create a transport error
    #[must_use]
    pub fn transport(transport_type: String, reason: String) -> Self {
        Self::Transport {
            transport_type,
            reason,
            status_code: None,
        }
    }

    /// Create a serialization error
    #[must_use]
    pub fn serialization(operation: String, reason: String) -> Self {
        Self::Serialization {
            operation,
            reason,
            type_name: None,
        }
    }

    /// Create a configuration error
    #[must_use]
    pub fn configuration(key: String, reason: String) -> Self {
        Self::Configuration {
            key,
            reason,
            expected_format: None,
        }
    }

    /// Create an internal server error
    #[must_use]
    pub fn internal(reason: String) -> Self {
        Self::Internal {
            reason,
            context: None,
        }
    }

    /// Get the error code for JSON-RPC responses
    #[must_use]
    pub const fn code(&self) -> i32 {
        match self {
            Self::JsonRpc { code, .. } => *code,
            Self::ProtocolVersionMismatch { .. } => -32_000,
            Self::InvalidRequest { .. } => -32_600,
            Self::Authorization { .. } => -32_601,
            Self::ResourceAccess { .. } => -32_602,
            Self::ToolExecution { .. } => -32_603,
            Self::Transport { .. } => -32_604,
            Self::Serialization { .. } => -32_605,
            Self::Configuration { .. } => -32_606,
            Self::Internal { .. } => -32_603,
        }
    }

    /// Check if the error is retryable
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Transport { .. } | Self::Internal { .. } => true,
            Self::JsonRpc { code, .. } => *code >= -32_099 && *code <= -32_000,
            _ => false,
        }
    }
}

/// Standard JSON-RPC error codes
pub mod json_rpc_codes {
    /// Parse error - Invalid JSON was received by the server
    pub const PARSE_ERROR: i32 = -32_700;
    /// Invalid Request - The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32_600;
    /// Method not found - The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32_601;
    /// Invalid params - Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32_602;
    /// Internal error - Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32_603;
    /// Server error range start
    pub const SERVER_ERROR_START: i32 = -32_099;
    /// Server error range end
    pub const SERVER_ERROR_END: i32 = -32_000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(McpError::invalid_request("test".to_owned()).code(), -32_600);
        assert_eq!(McpError::authorization("test".to_owned()).code(), -32_601);
        assert_eq!(
            McpError::resource_access(
                "manifest".to_owned(),
                "test".to_owned(),
                "not found".to_owned()
            )
            .code(),
            -32_602
        );
    }

    #[test]
    fn test_retryable_errors() {
        assert!(McpError::transport("http".to_owned(), "timeout".to_owned()).is_retryable());
        assert!(McpError::internal("database connection".to_owned()).is_retryable());
        assert!(!McpError::authorization("invalid token".to_owned()).is_retryable());
    }

    #[test]
    fn test_error_serialization() {
        let error = McpError::tool_execution("golem".to_owned(), "command failed".to_owned());
        let json = serde_json::to_string(&error).expect("should serialize");
        let deserialized: McpError = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(error, deserialized);
    }
}

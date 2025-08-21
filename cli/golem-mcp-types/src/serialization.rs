//! JSON-RPC and MCP message serialization utilities
//!
//! Provides robust serialization and deserialization for MCP protocol messages,
//! with comprehensive error handling and validation.

use crate::error::{McpError, McpResult};
use crate::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcErrorResponse, JsonRpcNotification};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// JSON-RPC message serializer/deserializer
pub struct MessageSerializer;

impl MessageSerializer {
    /// Serialize a message to JSON string
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the message cannot be serialized to JSON.
    pub fn serialize_message(message: &JsonRpcMessage) -> McpResult<String> {
        serde_json::to_string(message).map_err(|e| {
            McpError::serialization(
                "serialize".to_owned(),
                format!("Failed to serialize JSON-RPC message: {e}"),
            )
        })
    }

    /// Serialize a message to pretty-printed JSON string
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the message cannot be serialized to JSON.
    pub fn serialize_message_pretty(message: &JsonRpcMessage) -> McpResult<String> {
        serde_json::to_string_pretty(message).map_err(|e| {
            McpError::serialization(
                "serialize_pretty".to_owned(),
                format!("Failed to serialize JSON-RPC message: {e}"),
            )
        })
    }

    /// Deserialize a JSON string to a message
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the JSON cannot be parsed into a valid message.
    pub fn deserialize_message(json: &str) -> McpResult<JsonRpcMessage> {
        serde_json::from_str(json).map_err(|e| {
            McpError::serialization(
                "deserialize".to_owned(),
                format!("Failed to deserialize JSON-RPC message: {e}"),
            )
        })
    }

    /// Serialize any serializable value to JSON
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the value cannot be serialized.
    pub fn serialize<T: Serialize>(value: &T) -> McpResult<serde_json::Value> {
        serde_json::to_value(value).map_err(|e| {
            McpError::serialization(
                "serialize_value".to_owned(),
                format!("Failed to serialize value: {e}"),
            )
        })
    }

    /// Deserialize a JSON value to a specific type
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the value cannot be deserialized to the target type.
    pub fn deserialize<T: DeserializeOwned>(value: &serde_json::Value) -> McpResult<T> {
        serde_json::from_value(value.clone()).map_err(|e| {
            McpError::serialization(
                "deserialize_value".to_owned(),
                format!("Failed to deserialize value to {}: {e}", std::any::type_name::<T>()),
            )
        })
    }

    /// Validate that a JSON string represents a valid JSON-RPC message
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the JSON is invalid or doesn't conform to JSON-RPC structure.
    pub fn validate_json_rpc(json: &str) -> McpResult<()> {
        // First, ensure it's valid JSON
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
            McpError::serialization(
                "validate_json".to_owned(),
                format!("Invalid JSON: {e}"),
            )
        })?;

        // Check that it's an object
        let obj = value.as_object().ok_or_else(|| {
            McpError::serialization(
                "validate_structure".to_owned(),
                "JSON-RPC message must be an object".to_owned(),
            )
        })?;

        // Check for required jsonrpc field
        match obj.get("jsonrpc") {
            Some(serde_json::Value::String(version)) if version == "2.0" => {}
            Some(_) => {
                return Err(McpError::serialization(
                    "validate_version".to_owned(),
                    "jsonrpc field must be '2.0'".to_owned(),
                ));
            }
            None => {
                return Err(McpError::serialization(
                    "validate_version".to_owned(),
                    "Missing required 'jsonrpc' field".to_owned(),
                ));
            }
        }

        // Validate message type-specific requirements
        if obj.contains_key("method") {
            // Request or notification
            if !obj.contains_key("id") {
                // Notification - method is required, id is absent
                if !obj.contains_key("method") {
                    return Err(McpError::serialization(
                        "validate_notification".to_owned(),
                        "Notification must have 'method' field".to_owned(),
                    ));
                }
            } else {
                // Request - method and id are required
                if !obj.contains_key("method") {
                    return Err(McpError::serialization(
                        "validate_request".to_owned(),
                        "Request must have 'method' field".to_owned(),
                    ));
                }
            }
        } else if obj.contains_key("result") || obj.contains_key("error") {
            // Response - must have id
            if !obj.contains_key("id") {
                return Err(McpError::serialization(
                    "validate_response".to_owned(),
                    "Response must have 'id' field".to_owned(),
                ));
            }
            
            // Must have either result or error, but not both
            let has_result = obj.contains_key("result");
            let has_error = obj.contains_key("error");
            
            if has_result && has_error {
                return Err(McpError::serialization(
                    "validate_response".to_owned(),
                    "Response cannot have both 'result' and 'error' fields".to_owned(),
                ));
            }
            
            if !has_result && !has_error {
                return Err(McpError::serialization(
                    "validate_response".to_owned(),
                    "Response must have either 'result' or 'error' field".to_owned(),
                ));
            }
        } else {
            return Err(McpError::serialization(
                "validate_message_type".to_owned(),
                "Message must be a request, response, or notification".to_owned(),
            ));
        }

        Ok(())
    }
}

/// Message builder for constructing JSON-RPC messages
pub struct MessageBuilder;

impl MessageBuilder {
    /// Build a request message
    #[must_use]
    pub fn request(
        id: impl Into<crate::protocol::JsonRpcId>,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcMessage {
        JsonRpcMessage::Request(JsonRpcRequest::new(id.into(), method.into(), params))
    }

    /// Build a success response message
    #[must_use]
    pub fn response(
        id: impl Into<crate::protocol::JsonRpcId>,
        result: serde_json::Value,
    ) -> JsonRpcMessage {
        JsonRpcMessage::Response(JsonRpcResponse::new(id.into(), result))
    }

    /// Build an error response message
    #[must_use]
    pub fn error_response(
        id: Option<impl Into<crate::protocol::JsonRpcId>>,
        error: &McpError,
    ) -> JsonRpcMessage {
        JsonRpcMessage::Error(JsonRpcErrorResponse::from_mcp_error(
            id.map(Into::into),
            error,
        ))
    }

    /// Build a notification message
    #[must_use]
    pub fn notification(
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcMessage {
        JsonRpcMessage::Notification(JsonRpcNotification::new(method.into(), params))
    }

    /// Build a request with serialized parameters
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the parameters cannot be serialized.
    pub fn request_with_params<P: Serialize>(
        id: impl Into<crate::protocol::JsonRpcId>,
        method: impl Into<String>,
        params: &P,
    ) -> McpResult<JsonRpcMessage> {
        let params_value = MessageSerializer::serialize(params)?;
        Ok(Self::request(id, method, Some(params_value)))
    }

    /// Build a response with serialized result
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the result cannot be serialized.
    pub fn response_with_result<R: Serialize>(
        id: impl Into<crate::protocol::JsonRpcId>,
        result: &R,
    ) -> McpResult<JsonRpcMessage> {
        let result_value = MessageSerializer::serialize(result)?;
        Ok(Self::response(id, result_value))
    }

    /// Build a notification with serialized parameters
    ///
    /// # Errors
    ///
    /// Returns `McpError::Serialization` if the parameters cannot be serialized.
    pub fn notification_with_params<P: Serialize>(
        method: impl Into<String>,
        params: &P,
    ) -> McpResult<JsonRpcMessage> {
        let params_value = MessageSerializer::serialize(params)?;
        Ok(Self::notification(method, Some(params_value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcId, InitializeParams, McpCapabilities, McpClientInfo, McpImplementation};
    use std::collections::HashMap;

    #[test]
    fn test_message_serialization_roundtrip() {
        let original_message = JsonRpcMessage::Request(JsonRpcRequest::new(
            JsonRpcId::from("test-id"),
            "initialize".to_owned(),
            Some(serde_json::json!({"test": "value"})),
        ));

        let json = MessageSerializer::serialize_message(&original_message)
            .expect("should serialize");
        let deserialized = MessageSerializer::deserialize_message(&json)
            .expect("should deserialize");

        assert_eq!(original_message, deserialized);
    }

    #[test]
    fn test_json_rpc_validation_valid_request() {
        let json = r#"{"jsonrpc":"2.0","id":"1","method":"test","params":{}}"#;
        assert!(MessageSerializer::validate_json_rpc(json).is_ok());
    }

    #[test]
    fn test_json_rpc_validation_invalid_version() {
        let json = r#"{"jsonrpc":"1.0","id":"1","method":"test"}"#;
        assert!(MessageSerializer::validate_json_rpc(json).is_err());
    }

    #[test]
    fn test_json_rpc_validation_missing_method() {
        let json = r#"{"jsonrpc":"2.0","id":"1","params":{}}"#;
        assert!(MessageSerializer::validate_json_rpc(json).is_err());
    }

    #[test]
    fn test_json_rpc_validation_response_with_both_result_and_error() {
        let json = r#"{"jsonrpc":"2.0","id":"1","result":"ok","error":{"code":-1,"message":"fail"}}"#;
        assert!(MessageSerializer::validate_json_rpc(json).is_err());
    }

    #[test]
    fn test_message_builder_request() {
        let message = MessageBuilder::request("test-id", "initialize", None);
        
        if let JsonRpcMessage::Request(req) = message {
            assert_eq!(req.id, JsonRpcId::String("test-id".to_owned()));
            assert_eq!(req.method, "initialize");
            assert!(req.params.is_none());
        } else {
            panic!("Expected request message");
        }
    }

    #[test]
    fn test_message_builder_request_with_params() {
        let params = InitializeParams {
            protocol_version: "2025-06-18".to_owned(),
            capabilities: McpCapabilities {
                tools: None,
                resources: None,
                prompts: None,
                logging: None,
            },
            client_info: McpClientInfo {
                name: "test-client".to_owned(),
                version: "1.0.0".to_owned(),
                implementation: Some(McpImplementation {
                    name: "test".to_owned(),
                    version: "1.0.0".to_owned(),
                    metadata: HashMap::new(),
                }),
            },
        };

        let message = MessageBuilder::request_with_params("test-id", "initialize", &params)
            .expect("should build request");

        if let JsonRpcMessage::Request(req) = message {
            assert_eq!(req.method, "initialize");
            assert!(req.params.is_some());
        } else {
            panic!("Expected request message");
        }
    }

    #[test]
    fn test_message_builder_error_response() {
        let error = McpError::authorization("invalid token".to_owned());
        let message = MessageBuilder::error_response(Some("test-id"), &error);

        if let JsonRpcMessage::Error(err_resp) = message {
            assert_eq!(err_resp.id, Some(JsonRpcId::String("test-id".to_owned())));
            assert_eq!(err_resp.error.code, -32_601);
        } else {
            panic!("Expected error response message");
        }
    }
}
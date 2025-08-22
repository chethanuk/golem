//! Property-based tests for MCP types using proptest
//!
//! These tests verify invariants and properties that should hold across
//! a wide range of generated inputs, following Rust Staff Engineer best practices
//! for comprehensive validation of type safety and protocol compliance.

use golem_mcp_types::{
    error::McpError,
    protocol::{
        JsonRpcId, JsonRpcMessage, JsonRpcRequest, MCP_VERSION,
    },
    serialization::{MessageBuilder, MessageSerializer},
};
use proptest::prelude::*;
use serde_json::Value;

/// Generate arbitrary strings for testing
fn arb_string() -> impl Strategy<Value = String> {
    ".*".prop_map(|s| s.chars().take(1000).collect())
}

/// Generate arbitrary JSON-RPC IDs
fn arb_jsonrpc_id() -> impl Strategy<Value = JsonRpcId> {
    prop_oneof![
        arb_string().prop_map(JsonRpcId::String),
        any::<i64>().prop_map(JsonRpcId::Number),
    ]
}

/// Generate arbitrary JSON values (limited depth to prevent stack overflow)
fn arb_json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|i| Value::Number(serde_json::Number::from(i))),
        arb_string().prop_map(Value::String),
    ];
    leaf.prop_recursive(
        4, // max depth
        32, // max size
        4, // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                prop::collection::hash_map(arb_string(), inner, 0..4)
                    .prop_map(|map| Value::Object(map.into_iter().collect())),
            ]
        },
    )
}

/// Generate arbitrary JSON-RPC requests
fn arb_jsonrpc_request() -> impl Strategy<Value = JsonRpcRequest> {
    (arb_jsonrpc_id(), arb_string(), prop::option::of(arb_json_value()))
        .prop_map(|(id, method, params)| JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id,
            method,
            params,
        })
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Test JSON-RPC ID serialization roundtrip property
    #[test]
    fn prop_jsonrpc_id_serialization_roundtrip() {
        proptest!(|(id in arb_jsonrpc_id())| {
            let json_str = serde_json::to_string(&id).unwrap();
            let deserialized: JsonRpcId = serde_json::from_str(&json_str).unwrap();
            prop_assert_eq!(id, deserialized);
        });
    }

    /// Test JSON-RPC request serialization roundtrip property
    #[test]
    fn prop_jsonrpc_request_roundtrip() {
        proptest!(|(request in arb_jsonrpc_request())| {
            let json_str = serde_json::to_string(&request).unwrap();
            let deserialized: JsonRpcRequest = serde_json::from_str(&json_str).unwrap();
            
            prop_assert_eq!(request.jsonrpc, deserialized.jsonrpc);
            prop_assert_eq!(request.id, deserialized.id);
            prop_assert_eq!(request.method, deserialized.method);
            
            // Handle the JSON-RPC specification behavior where Some(Null) becomes None
            let normalized_expected_params = match &request.params {
                Some(Value::Null) => None,
                other => other.clone(),
            };
            prop_assert_eq!(normalized_expected_params, deserialized.params);
        });
    }

    /// Test error codes consistency property
    #[test]
    fn prop_error_codes_consistency() {
        proptest!(|(
            reason in arb_string(),
            tool_name in arb_string(),
            resource_type in arb_string(),
            resource_id in arb_string(),
            operation in arb_string(),
            key in arb_string(),
            transport_type in arb_string(),
            server_version in arb_string(),
            client_version in arb_string(),
        )| {
            // Test that error codes are consistent across error variants
            prop_assert_eq!(McpError::invalid_request(reason.clone()).code(), -32_600);
            prop_assert_eq!(McpError::authorization(reason.clone()).code(), -32_601);
            prop_assert_eq!(McpError::resource_access(resource_type.clone(), resource_id.clone(), reason.clone()).code(), -32_602);
            prop_assert_eq!(McpError::tool_execution(tool_name.clone(), reason.clone()).code(), -32_603);
            prop_assert_eq!(McpError::transport(transport_type.clone(), reason.clone()).code(), -32_604);
            prop_assert_eq!(McpError::serialization(operation.clone(), reason.clone()).code(), -32_605);
            prop_assert_eq!(McpError::configuration(key.clone(), reason.clone()).code(), -32_606);
            prop_assert_eq!(McpError::internal(reason.clone()).code(), -32_603);
            prop_assert_eq!(McpError::protocol_version_mismatch(server_version.clone(), client_version.clone()).code(), -32_000);
        });
    }

    /// Test error serialization preserves codes property
    #[test]
    fn prop_error_serialization_preserves_codes() {
        proptest!(|(
            code in any::<i32>(),
            message in arb_string(),
        )| {
            let error = McpError::json_rpc(code, message.clone());
            let json_str = serde_json::to_string(&error).unwrap();
            let deserialized: McpError = serde_json::from_str(&json_str).unwrap();
            
            prop_assert_eq!(error.code(), deserialized.code());
            prop_assert_eq!(code, deserialized.code());
        });
    }

    /// Test message builder request validity property
    #[test]
    fn prop_message_builder_request_validity() {
        proptest!(|(
            id_str in arb_string(),
            method in arb_string(),
            params in prop::option::of(arb_json_value()),
        )| {
            let message = MessageBuilder::request(id_str.clone(), method.clone(), params.clone());
            
            if let JsonRpcMessage::Request(req) = message {
                prop_assert_eq!(req.jsonrpc, "2.0");
                prop_assert_eq!(req.id, JsonRpcId::String(id_str));
                prop_assert_eq!(req.method, method);
                prop_assert_eq!(req.params, params);
            } else {
                panic!("Expected Request message");
            }
        });
    }

    /// Test message serialization produces valid JSON property
    #[test]
    fn prop_message_serialization_produces_valid_json() {
        proptest!(|(request in arb_jsonrpc_request())| {
            let message = JsonRpcMessage::Request(request);
            let json_str = MessageSerializer::serialize_message(&message).unwrap();
            
            // Should parse as valid JSON
            let _: Value = serde_json::from_str(&json_str).unwrap();
            
            // Should contain required JSON-RPC fields
            let parsed: Value = serde_json::from_str(&json_str).unwrap();
            prop_assert_eq!(&parsed["jsonrpc"], "2.0");
            prop_assert!(parsed.get("method").is_some());
            prop_assert!(parsed.get("id").is_some());
        });
    }

    /// Test error retryability consistency property
    #[test]
    fn prop_error_retryability_consistency() {
        proptest!(|(
            reason in arb_string(),
            transport_type in arb_string(),
            code in any::<i32>(),
        )| {
            // Transport and internal errors should always be retryable
            prop_assert!(McpError::transport(transport_type.clone(), reason.clone()).is_retryable());
            prop_assert!(McpError::internal(reason.clone()).is_retryable());
            
            // Client errors should never be retryable
            prop_assert!(!McpError::invalid_request(reason.clone()).is_retryable());
            prop_assert!(!McpError::authorization(reason.clone()).is_retryable());
            
            // JSON-RPC server errors in range should be retryable
            if code >= -32_099 && code <= -32_000 {
                prop_assert!(McpError::json_rpc(code, reason.clone()).is_retryable());
            } else {
                prop_assert!(!McpError::json_rpc(code, reason.clone()).is_retryable());
            }
        });
    }

    /// Test large message handling property
    #[test]
    fn prop_large_message_handling() {
        proptest!(|(
            size in 100usize..1000,
            base_string in arb_string(),
        )| {
            let large_string = base_string.repeat(size / base_string.len().max(1));
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_owned(),
                id: JsonRpcId::String("large-test".to_owned()),
                method: "large_method".to_owned(),
                params: Some(serde_json::json!({"data": large_string})),
            };
            
            let message = JsonRpcMessage::Request(request);
            let json_str = MessageSerializer::serialize_message(&message).unwrap();
            let deserialized = MessageSerializer::deserialize_message(&json_str).unwrap();
            
            prop_assert_eq!(message, deserialized);
        });
    }

    /// Test protocol version format property
    #[test]
    fn test_protocol_version_format() {
        assert_eq!(MCP_VERSION, "2025-06-18");
        
        let parts: Vec<&str> = MCP_VERSION.split('-').collect();
        assert_eq!(parts.len(), 3);
        
        // Should be parseable as date components
        let year: u16 = parts[0].parse().unwrap();
        let month: u8 = parts[1].parse().unwrap();
        let day: u8 = parts[2].parse().unwrap();
        
        assert!(year >= 2024 && year <= 2030);
        assert!(month >= 1 && month <= 12);
        assert!(day >= 1 && day <= 31);
    }

    /// Test specific edge cases that should always hold
    #[test]
    fn test_empty_strings_handling() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: JsonRpcId::String("".to_owned()),
            method: "".to_owned(),
            params: None,
        };
        
        let json_str = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(request, deserialized);
    }

    /// Test numeric ID edge cases
    #[test]
    fn test_numeric_id_edge_cases() {
        for id_value in [i64::MIN, -1, 0, 1, i64::MAX] {
            let id = JsonRpcId::Number(id_value);
            let json_str = serde_json::to_string(&id).unwrap();
            let deserialized: JsonRpcId = serde_json::from_str(&json_str).unwrap();
            assert_eq!(id, deserialized);
        }
    }

    /// Test null params handling
    #[test]
    fn test_null_params_handling() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: JsonRpcId::String("test".to_owned()),
            method: "test_method".to_owned(),
            params: Some(serde_json::Value::Null),
        };
        
        let json_str = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json_str).unwrap();
        
        // JSON-RPC spec: Some(Null) should be treated as None after serialization/deserialization
        let expected_request = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: JsonRpcId::String("test".to_owned()),
            method: "test_method".to_owned(),
            params: None,
        };
        assert_eq!(expected_request, deserialized);
    }
}